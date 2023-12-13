use std::io::{BufRead, BufReader};

use clap::ArgMatches;
use duct::cmd;

use crate::{error::AocError, util::get_day};

pub async fn test(matches: &ArgMatches) -> Result<(), AocError>
{
    let day = get_day(matches)?;

    let reader = cmd!(
        "cargo",
        "test",
        "--color",
        "always",
        "--bin",
        format!("day_{:02}", day).as_str(),
        "--",
        "--color",
        "always"
    )
    .stderr_to_stdout()
    .reader()?;

    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    while let Some(Ok(line)) = lines.next()
    {
        println!("{}", line);
    }
    Ok(())
}
