use std::collections::HashMap;

use clap::ArgMatches;
use sanitize_html::rules::predefined::DEFAULT;

use super::request::AocRequest;
use crate::error::AocError;

pub fn get_submit_day(matches: &ArgMatches) -> Option<Result<Task, AocError>>
{
    let day = matches.get_one::<String>("submit")?;
    let Ok(day) = day.parse::<u8>()
    else
    {
        return Some(Err(AocError::ParseIntError));
    };

    match day
    {
        1 => Some(Ok(Task::One)),
        2 => Some(Ok(Task::Two)),
        _ => Some(Err(AocError::InvalidSubmitDay)),
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
    let start = out.split(&format!("Task {}: ", task)).nth(1)?;
    let encoded_answer = start.split_once('\n').unwrap_or((start, "")).0;
    let answer = strip_ansi_escapes::strip(encoded_answer);
    String::from_utf8(answer).ok()
}

fn parse_and_sanitize_output(output: &str) -> Option<String>
{
    let start = output.find("<article><p>")?;
    let end = output.find("</p></article>")?;
    let body = &output[start..end];
    sanitize_html::sanitize_str(&DEFAULT, body).ok()
}

pub async fn submit(output: &str, task: &Task, day: u32, year: i32) -> Result<String, AocError>
{
    let answer = get_answer(output, task).ok_or(AocError::ParseStdout)?;
    let url = format!("https://adventofcode.com/{}/day/{}/answer", year, day);

    let mut form = HashMap::new();
    form.insert("level", if task == &Task::One { 1 } else { 2 }.to_string());
    form.insert("answer", answer);
    let res = AocRequest::new().post(&url, &form).await?;

    let text = &res.text().await?;
    let parsed_output = parse_and_sanitize_output(text).ok_or(AocError::SanitizeHtml)?;

    Ok(parsed_output)
}
