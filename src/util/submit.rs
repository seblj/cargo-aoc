use std::collections::HashMap;

use clap::ArgMatches;
use reqwest::header::{COOKIE, USER_AGENT};
use sanitize_html::rules::predefined::DEFAULT;

use super::ParseArgError;

pub fn get_submit_day(matches: &ArgMatches) -> Result<Task, ParseArgError>
{
    let day = matches.get_one::<String>("submit").ok_or_else(|| ParseArgError::TypeError)?;

    match day.parse::<u8>().map_err(|_| ParseArgError::ParseError)
    {
        Ok(day) => match day
        {
            1 => Ok(Task::One),
            2 => Ok(Task::Two),
            _ =>
            {
                Err(ParseArgError::Invalid("Only allowed to pass in 1 or 2 for submit".to_string()))
            },
        },
        Err(_) => Err(ParseArgError::ParseError),
    }
}

impl std::fmt::Display for SubmitError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            SubmitError::TokenError => write!(f, "Could not find AOC_TOKEN"),
            SubmitError::HttpRequestError => write!(f, "Error on submitting answer"),
            SubmitError::SanitizeHtmlError => write!(f, "Error on sanitizing answer"),
            SubmitError::ParseStdoutError => write!(f, "Error on getting answer from task"),
            SubmitError::HttpTextError => write!(f, "Error on getting response text"),
        }
    }
}

#[derive(Eq, PartialEq)]
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

fn get_answer(out: &str, task: &Task) -> Option<String>
{
    let start = out.split(&format!("Task {}: ", task.to_string())).nth(1)?;
    let encoded_answer = start.split_once('\n').unwrap_or((start, "")).0;
    let answer = strip_ansi_escapes::strip(encoded_answer).ok()?;
    String::from_utf8(answer).ok()
}

fn parse_and_sanitize_output(output: &str) -> Option<String>
{
    let start = output.find("<article><p>")?;
    let end = output.find("</p></article>")?;
    let body = &output[start..end];
    sanitize_html::sanitize_str(&DEFAULT, &body).ok()
}

pub enum SubmitError
{
    TokenError,
    HttpRequestError,
    SanitizeHtmlError,
    ParseStdoutError,
    HttpTextError,
}

pub async fn submit(output: &str, task: &Task, day: u32, year: i32) -> Result<String, SubmitError>
{
    let answer = get_answer(output, &task).ok_or_else(|| SubmitError::ParseStdoutError)?;
    let token = dotenv::var("AOC_TOKEN").map_err(|_| SubmitError::TokenError)?;
    let client = reqwest::Client::new();
    let url = format!("https://adventofcode.com/{}/day/{}/answer", year, day);

    let mut params = HashMap::new();
    params.insert("level", if task == &Task::One { 1 } else { 2 }.to_string());
    params.insert("answer", answer);
    let res = client
        .post(&url)
        .form(&params)
        .header(COOKIE, format!("session={}", token))
        .header(USER_AGENT, "https://github.com/seblj/cargo-aoc by seblyng98@gmail.com")
        .send()
        .await
        .map_err(|_| SubmitError::HttpRequestError)?;
    let text = &res.text().await.map_err(|_| SubmitError::HttpTextError)?.to_string();
    let parsed_output =
        parse_and_sanitize_output(text).ok_or_else(|| SubmitError::SanitizeHtmlError)?;

    Ok(parsed_output)
}
