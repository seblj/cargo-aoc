use std::collections::HashMap;

use clap::ArgMatches;
use sanitize_html::rules::predefined::DEFAULT;

use super::{parse_get_answers, request::AocRequest, Task};
use crate::error::AocError;

pub fn get_submit_task(matches: &ArgMatches) -> Option<Result<Task, AocError>> {
    let task = matches.get_one::<String>("submit")?.parse::<u8>();
    if let Err(e) = task {
        return Some(Err(e.into()));
    }
    match task.unwrap() {
        1 => Some(Ok(Task::One)),
        2 => Some(Ok(Task::Two)),
        _ => Some(Err(AocError::InvalidSubmitTask)),
    }
}

fn parse_and_sanitize_output(output: &str) -> Option<String> {
    let start = output.find("<article><p>")?;
    let end = output.find("</p></article>")?;
    let body = &output[start..end];
    sanitize_html::sanitize_str(&DEFAULT, body).ok()
}

pub async fn submit(output: &str, task: Task, day: u32, year: i32) -> Result<String, AocError> {
    let (p1, p2) = parse_get_answers(output);
    let answer = if task == Task::One { p1 } else { p2 }.ok_or(AocError::ParseStdout)?;
    let url = format!("https://adventofcode.com/{}/day/{}/answer", year, day);

    let mut form = HashMap::new();
    form.insert("level", if task == Task::One { 1 } else { 2 }.to_string());
    form.insert("answer", answer);
    let res = AocRequest::new().post(&url, &form).await?;

    let text = &res.text().await?;
    parse_and_sanitize_output(text).ok_or(AocError::SanitizeHtml)
}
