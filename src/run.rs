use chrono::prelude::*;
use clap::ArgMatches;

use crate::util::file::*;

pub fn get_day(matches: &ArgMatches) -> Result<u32, std::num::ParseIntError>
{
    matches.get_one::<String>("day").unwrap().parse()
}

pub fn get_year(matches: &ArgMatches) -> Result<i32, std::num::ParseIntError>
{
    let year = matches.get_one::<String>("year").unwrap();
    if year.chars().count() == 2
    {
        format!("20{}", year).parse()
    }
    else
    {
        year.parse()
    }
}


async fn get_input_file(matches: &ArgMatches) -> Result<String, Box<dyn std::error::Error>>
{
    if matches.get_flag("test")
    {
        Ok("test".to_owned())
    }
    else
    {
        Ok("input".to_owned())
    }
}

pub async fn run(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>>
{
    let day = get_day(matches)?;

    if !(1..=25).contains(&day)
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

    let input = get_input_file(matches).await?;
    let flags = matches.get_one::<String>("compiler-flags").unwrap();

    let res = tokio::process::Command::new("cargo")
        .args(["run", "--color", "always", "--bin", format!("day_{:02}", day).as_str(), &input])
        .env("RUSTFLAGS", flags)
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
