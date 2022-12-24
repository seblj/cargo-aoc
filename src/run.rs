use std::io::{BufRead, BufReader};

use chrono::prelude::*;
use clap::ArgMatches;
use duct::cmd;

use crate::{
    error::AocError,
    util::{
        file::{cargo_path, day_path, download_input_file},
        get_day, get_year,
        submit::{self, get_submit_day},
    },
};

fn get_input_file(matches: &ArgMatches) -> &str
{
    if matches.get_flag("test")
    {
        "test"
    }
    else
    {
        "input"
    }
}

pub async fn run(matches: &ArgMatches) -> Result<(), AocError>
{
    let day = get_day(matches)?;
    let path = cargo_path().await.unwrap_or(std::env::current_dir()?);
    let dir = day_path(path, day).await?;

    if !dir.join("input").exists()
    {
        let year = get_year(matches)?;
        let current_year = Utc::now().year();
        let current_month = Utc::now().month();
        if year < 2015 || year > current_year || (year == current_year && current_month < 12)
        {
            return Err(AocError::InvalidYear);
        }
        download_input_file(day, year, &dir).await?;
    }

    let input = get_input_file(matches);
    let flags = matches.get_one::<String>("compiler-flags").ok_or(AocError::ArgMatches)?;

    let reader = cmd!(
        "cargo",
        "run",
        "--color",
        "always",
        "--bin",
        format!("day_{:02}", day).as_str(),
        input
    )
    .dir(dir)
    .env("RUSTFLAGS", flags)
    .stderr_to_stdout()
    .reader()?;

    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    let mut out = String::new();
    while let Some(Ok(line)) = lines.next()
    {
        println!("{}", line);
        if line.contains("ms)\tTask")
        {
            out.push_str(&line);
            out.push('\n');
        }
    }

    // Only try to submit if the submit flag is passed
    if let Some(submit) = get_submit_day(matches)
    {
        let year = get_year(matches)?;
        match submit
        {
            Ok(task) => match submit::submit(&out, &task, day, year).await
            {
                Ok(output) => println!("Task {}: {}", task, output),
                Err(e) => println!("Error submitting task {}: {}", task, e),
            },
            Err(e) => println!("Error: {}", e),
        }
    }
    Ok(())
}
