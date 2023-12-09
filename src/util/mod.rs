use std::path::PathBuf;

use clap::ArgMatches;

use self::{
    file::{cargo_path, day_path},
    request::AocRequest,
};
use crate::error::AocError;

pub mod file;
pub mod request;
#[cfg(feature = "submit")] pub mod submit;

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Task
{
    One,
    Two,
}

impl std::fmt::Display for Task
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Task::One => write!(f, "one"),
            Task::Two => write!(f, "two"),
        }
    }
}


pub fn get_year(matches: &ArgMatches) -> Result<i32, AocError>
{
    let year = matches.get_one::<String>("year").ok_or(AocError::ArgMatches)?;
    if year.chars().count() == 2
    {
        Ok(format!("20{}", year).parse()?)
    }
    else
    {
        Ok(year.parse()?)
    }
}

pub fn get_day(matches: &ArgMatches) -> Result<u32, AocError>
{
    let day = matches.get_one::<String>("day").ok_or(AocError::ArgMatches)?.parse::<u32>()?;
    if !(1..=25).contains(&day)
    {
        Err(AocError::InvalidRunDay)
    }
    else
    {
        Ok(day)
    }
}

pub fn get_time_symbol() -> String
{
    let sym = std::env::var("TASKUNIT").unwrap_or("ms".to_owned());
    if sym == "us"
    {
        "μs".to_owned()
    }
    else
    {
        sym
    }
}

#[derive(Debug)]
pub struct AocInfo
{
    pub day:          u32,
    pub year:         u32,
    pub title:        String,
    pub part1_answer: Option<String>,
    pub part2_answer: Option<String>,
}

pub async fn get_day_title_and_answers(day: u32, year: u32) -> Result<AocInfo, AocError>
{
    if let Ok(cache) = read_cache_answers(day, year).await
    {
        return Ok(cache);
    }

    let url = format!("https://adventofcode.com/{}/day/{}", year, day);

    let res = AocRequest::new().get(&url).await?;

    let text = res.text().await?;

    let h2 = "<h2>--- ";
    let idx1 = text.find(h2).unwrap() + h2.len();
    let idx2 = text[idx1..].find(" ---</h2>").unwrap();
    let (_, title) = text[idx1..idx1 + idx2].split_once(": ").unwrap();

    let search = "Your puzzle answer was <code>";
    let mut iter = text.lines().filter(|&line| line.contains(search)).map(|line| {
        let code_end = "</code>";
        let idx = line.find(search).unwrap() + search.len();
        let end = line[idx..].find(code_end).unwrap();

        line[idx..idx + end].to_owned()
    });
    let a1 = iter.next();
    let a2 = iter.next();

    let info = AocInfo {
        day,
        year,
        title: title.to_owned(),
        part1_answer: a1,
        part2_answer: a2,
    };

    // Ignore possible errors during cache write
    let _ = write_cache_answers(day, &info).await;

    Ok(info)
}

pub fn parse_get_answers(output: &str) -> (Option<String>, Option<String>)
{
    let strip = strip_ansi_escapes::strip(output);
    let text = std::str::from_utf8(&strip).unwrap();

    let parse = |line: &str| line.split_ascii_whitespace().next_back().map(|s| s.to_string());
    let mut iter = text.split('\n');
    (iter.next().and_then(parse), iter.next().and_then(parse))
}

async fn get_cache_path(day: u32) -> Result<PathBuf, AocError>
{
    // Tries to read it from the cache before making a request
    let path = cargo_path().await?;
    Ok(day_path(path, day).await?.join(".answers"))
}

pub async fn write_cache_answers(day: u32, info: &AocInfo) -> Result<(), AocError>
{
    let path = get_cache_path(day).await?;
    if let (Some(a1), Some(a2)) = (&info.part1_answer, &info.part2_answer)
    {
        tokio::fs::write(path, format!("{}\n{}\n{}", info.title, a1, a2)).await?;
    }

    Ok(())
}

pub async fn read_cache_answers(day: u32, year: u32) -> Result<AocInfo, AocError>
{
    let path = get_cache_path(day).await?;
    let res = tokio::fs::read_to_string(path).await?;
    let lines = res.lines().collect::<Vec<_>>();
    Ok(AocInfo {
        day,
        year,
        title: lines[0].to_owned(),
        part1_answer: Some(lines[1].to_owned()),
        part2_answer: Some(lines[2].to_owned()),
    })
}
