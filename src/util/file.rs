use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use reqwest::{
    header::{COOKIE, USER_AGENT},
    StatusCode,
};

pub async fn day_path<P: AsRef<Path>>(
    root: P,
    day: u32,
) -> Result<std::path::PathBuf, std::io::Error>
{
    use std::{collections::VecDeque, io::*};
    let dir_name = format!("day_{:02}", day);
    let dir_name = OsStr::new(&dir_name);
    let ignore = [OsStr::new("target"), OsStr::new(".git")];

    let mut vec = VecDeque::new();
    vec.push_back(root.as_ref().as_os_str().to_os_string());

    while let Some(path) = vec.pop_front()
    {
        let mut stream = tokio::fs::read_dir(&path).await?;
        while let Ok(Some(entry)) = stream.next_entry().await
        {
            let file_name = entry.file_name();
            if ignore.contains(&file_name.as_os_str())
            {
                continue;
            }

            if file_name == dir_name
            {
                let mut buff: PathBuf = path.into();
                buff.push(dir_name);
                return Ok(buff);
            }

            let file_type = entry.file_type().await?;
            if file_type.is_dir()
            {
                let mut path = Path::new(&path).to_path_buf();
                path.push(entry.file_name());
                let name = path.as_os_str().to_os_string();

                vec.push_back(name);
            }
        }
    }
    let err_text = format!("could not find folder for day_{}", day);
    Err(Error::new(ErrorKind::NotFound, err_text))
}

pub async fn cargo_path<P: AsRef<Path>>(path: P) -> Result<std::path::PathBuf, std::io::Error>
{
    use std::{collections::VecDeque, io::*};

    let mut vec = VecDeque::new();
    vec.push_back(path.as_ref().as_os_str().to_os_string());

    let not_found = || Error::new(ErrorKind::NotFound, "could not find Cargo.toml file");

    while let Some(path) = vec.pop_front()
    {
        let mut stream = tokio::fs::read_dir(&path).await?;
        while let Ok(Some(entry)) = stream.next_entry().await
        {
            if entry.file_name() == OsStr::new("Cargo.toml")
            {
                return Ok(path.into());
            }
        }
        // add parent
        let path = Path::new(&path);
        let path = path.parent().ok_or_else(not_found)?.as_os_str().to_os_string();
        vec.push_back(path);
    }

    Err(not_found())
}

pub async fn download_input_file(
    day: u32,
    year: i32,
    dir: &Path,
) -> Result<(), Box<dyn std::error::Error>>
{
    let token = dotenv::var("AOC_TOKEN")?;
    let url = format!("https://adventofcode.com/{}/day/{}/input", year, day);
    let res = reqwest::Client::new()
        .get(url)
        .header(COOKIE, format!("session={}", token))
        .header(USER_AGENT, "https://github.com/seblj/cargo-aoc by seblyng98@gmail.com")
        .send()
        .await?;

    if res.status() != StatusCode::OK
    {
        return Err(Box::<_>::from(format!(
            "Couldn't download input for year: {} and day: {}",
            year, day
        )));
    }

    let bytes = res.bytes().await?;
    tokio::fs::write(dir.join("input"), bytes).await?;
    Ok(())
}
