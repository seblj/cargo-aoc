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

pub async fn run(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>>
{
    let day = get_day(matches)?;

    if day < 1 || day > 25
    {
        let err = Box::<dyn std::error::Error>::from("Day must be between 1 and 25");
        return Err(err);
    }


    let cwd = std::env::current_dir()?;
    let path = cargo_path(&cwd).await.unwrap_or(cwd);
    let dir = day_path(path, day).await?;
    let dir = dir.to_str().unwrap();

    // check here that 'input' actually exists
    // ...

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
        println!("error {}", err);
    }

    let out = std::str::from_utf8(&res.stdout)?.trim_end();
    println!("{}", out);

    Ok(())
}
