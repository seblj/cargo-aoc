use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use chrono::Datelike;
use reqwest::StatusCode;

use super::request::AocRequest;
use crate::error::AocError;

pub fn get_day_from_path() -> Result<Option<u32>, AocError> {
    let get_day = |s: &str| -> Option<u32> {
        let mut num = "".to_string();

        for ch in s.chars() {
            if ch.is_ascii_digit() {
                if ch == '0' && num.is_empty() {
                    continue;
                }
                num.push(ch);
            }
        }
        let num = num.parse::<u32>().unwrap();

        (1..=25).contains(&num).then_some(num)
    };

    let mut cwd = std::env::current_dir()?;

    loop {
        let name = cwd.file_name();
        let name = name
            .ok_or(AocError::InvalidRunDay)?
            .to_str()
            .ok_or(AocError::InvalidRunDay)?;

        if let Some(day) = get_day(&name) {
            return Ok(Some(day));
        }
        if !cwd.pop() {
            return Ok(None);
        }
    }
}

pub fn get_root_path() -> Result<std::path::PathBuf, AocError> {
    let mut cwd = std::env::current_dir()?;

    loop {
        let name = cwd
            .file_name()
            .ok_or_else(|| std::io::Error::last_os_error())?;

        let Ok(year): Result<i32, _> = name.to_str().unwrap().parse() else {
            if !cwd.pop() {
                return Err(AocError::InvalidYear);
            }
            continue;
        };

        let current_year = chrono::Utc::now().year();

        if (2015..=current_year).contains(&year) {
            return Ok(cwd);
        }
        if !cwd.pop() {
            return Err(AocError::InvalidYear);
        }
    }
}

pub async fn day_path<P: AsRef<Path>>(root: P, day: u32) -> Result<std::path::PathBuf, AocError> {
    use std::{collections::VecDeque, io::*};
    let dir_name = format!("day_{:02}", day);
    let dir_name = OsStr::new(&dir_name);
    let ignore = [OsStr::new("target"), OsStr::new(".git")];

    let mut vec = VecDeque::new();
    vec.push_back(root.as_ref().as_os_str().to_os_string());

    while let Some(path) = vec.pop_front() {
        let mut stream = tokio::fs::read_dir(&path).await?;
        while let Ok(Some(entry)) = stream.next_entry().await {
            let file_name = entry.file_name();
            if ignore.contains(&file_name.as_os_str()) {
                continue;
            }

            if file_name == dir_name {
                let mut buff: PathBuf = path.into();
                buff.push(dir_name);
                return Ok(buff);
            }

            let file_type = entry.file_type().await?;
            if file_type.is_dir() {
                let mut path = Path::new(&path).to_path_buf();
                path.push(entry.file_name());
                let name = path.as_os_str().to_os_string();

                vec.push_back(name);
            }
        }
    }
    let err_text = format!("could not find folder for day_{}", day);
    Err(Error::new(ErrorKind::NotFound, err_text).into())
}

pub async fn download_input_file(day: u32, year: i32, dir: &Path) -> Result<(), AocError> {
    let url = format!("https://adventofcode.com/{}/day/{}/input", year, day);
    let res = AocRequest::new().get(url).await?;

    if res.status() != StatusCode::OK {
        return Err(AocError::DownloadError(format!(
            "Couldn't download input for year: {} and day: {}",
            year, day
        )));
    }

    let bytes = res.bytes().await?;
    tokio::fs::write(dir.join("input"), bytes).await?;
    Ok(())
}
