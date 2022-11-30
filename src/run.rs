use chrono::prelude::*;
use clap::ArgMatches;

use crate::util::file::*;

fn get_day(matches: &ArgMatches) -> Result<u32, std::num::ParseIntError>
{
    if let Some(day) = matches.get_one::<String>("day")
    {
        day.parse()
    }
    else
    {
        Ok(Utc::now().day())
    }
}

fn get_year(matches: &ArgMatches) -> Result<i32, std::num::ParseIntError>
{
    if let Some(year) = matches.get_one::<String>("year")
    {
        if year.chars().count() == 2
        {
            format!("20{}", year).parse()
        }
        else
        {
            year.parse()
        }
    }
    else
    {
        Ok(Utc::now().year())
    }
}

pub async fn run(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>>
{
    let day = get_day(matches)?;

    if day < 1 || day > 25
    {
        return Err(Box::<_>::from("Day must be between 1 and 25"));
    }

    let cwd = std::env::current_dir()?;
    let path = cargo_path(&cwd).await.unwrap_or(cwd);
    let dir = day_path(path, day).await?;

    if !dir.join("input").exists()
    {
        let year = get_year(matches)?;
        let current_year = Utc::now().year();
        let current_month = Utc::now().month();
        if year < 2015 || year > current_year || (year == current_year && current_month < 12)
        {
            return Err(Box::<_>::from(format!("No advent of code for {}", year)));
        }
        download_input_file(day, year, &dir).await?;
    }

    let res = tokio::process::Command::new("cargo")
        .args(["run", "--bin", format!("day_{:02}", day).as_str(), "--color", "always"])
        .current_dir(dir)
        .output()
        .await?;

    // Print the (potential) errors FIRST so that if we got an answer
    // it is at the bottom
    let err = std::str::from_utf8(&res.stderr)?;
    if !err.is_empty()
    {
        println!("{}", err);
    }

    let out = std::str::from_utf8(&res.stdout)?.trim_end();
    println!("{}", out);

    Ok(())
}
