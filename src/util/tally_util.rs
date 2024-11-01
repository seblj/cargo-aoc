use std::{path::PathBuf, process::Output};

use chrono::Datelike;
use clap::ArgMatches;

use crate::error::AocError;

use super::get_time_symbol;

#[derive(Debug, Default)]
pub struct TableInfo {
    pub title: String,
    pub ans1: Option<String>,
    pub ans2: Option<String>,

    pub correct1: bool,
    pub correct2: bool,
}

#[derive(Debug, Default)]
pub struct Time(pub usize, pub Option<usize>);

#[derive(Debug)]
pub struct BuildRes {
    pub day: usize,
    pub path: PathBuf,
    pub info: TableInfo,
    pub time: Time,
}

impl BuildRes {
    pub fn new(day: usize, path: PathBuf) -> Self {
        Self {
            day,
            path,
            info: Default::default(),
            time: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorTypes {
    CompilerError(String),
    RuntimeError(String),
    InputDownloadError,
    NotImplementd,
}
impl ToString for ErrorTypes {
    fn to_string(&self) -> String {
        match self {
            ErrorTypes::CompilerError(s) => s.to_owned(),
            ErrorTypes::RuntimeError(s) => s.to_owned(),
            ErrorTypes::NotImplementd => String::from("UNIMPL"),
            ErrorTypes::InputDownloadError => String::from("INPUT DOWNLOAD ERROR"),
        }
    }
}
#[derive(Debug)]
pub struct Error {
    pub day: usize,
    pub title: String,
    pub r#type: ErrorTypes,
}

pub fn extract_comiler_error(stderr: String) -> String {
    let pos = stderr.find(": ").unwrap();
    let mut split = stderr[pos + 2..].split('\n');
    split.next().unwrap().to_owned()
}

pub fn extract_runtime_error(stderr: Vec<u8>) -> String {
    let s = String::from_utf8(stderr).unwrap();
    let mut split = s.split('\n');
    split.nth(1).unwrap().to_string()
}

pub fn get_number_of_runs(matches: &ArgMatches) -> Result<usize, AocError> {
    Ok(matches
        .get_one::<String>("runs")
        .ok_or(AocError::ArgMatches)?
        .parse()?)
}

pub fn get_possible_days(year: usize) -> Result<Vec<usize>, AocError> {
    let now = chrono::Utc::now();

    if year as i32 == now.year() {
        if now.month() < 12 {
            Err(AocError::InvalidMonth)
        } else {
            Ok((1..=now.day() as usize).collect())
        }
    } else {
        Ok((1..=25).collect())
    }
}

pub fn parse_get_times(output: Output) -> Result<(usize, Option<usize>), AocError> {
    let unit = get_time_symbol();
    let parse = |line: &str| -> Result<usize, AocError> {
        let start = line.find('(').ok_or(AocError::ParseStdout)?;
        let stop = line
            .find(&format!("{unit})"))
            .ok_or(AocError::ParseStdout)?;
        Ok(line[start + 1..stop].parse().unwrap())
    };
    let text = std::str::from_utf8(&output.stdout).unwrap();
    let mut iter = text.split('\n');
    let p1 = parse(iter.next().unwrap())?;
    let p2 = iter.next().and_then(|n| parse(n).ok());

    Ok((p1, p2))
}

pub fn parse_get_answers(output: Output) -> (Option<String>, Option<String>) {
    let text = std::str::from_utf8(&output.stdout).unwrap();
    let strip = strip_ansi_escapes::strip(text);
    let text = std::str::from_utf8(&strip).unwrap();

    let parse = |line: &str| {
        line.split_ascii_whitespace()
            .next_back()
            .map(|s| s.to_string())
    };
    let mut iter = text.split('\n');
    (iter.next().and_then(parse), iter.next().and_then(parse))
}

pub fn get_target(path_buf: PathBuf, day: usize) -> PathBuf {
    let bin = format!("day_{:02}", day);
    let mut path_buf = path_buf;
    path_buf.push("target/release");
    path_buf.push(&bin);
    path_buf
}
