use std::io::{BufRead, BufReader};

use clap::ArgMatches;
use duct::cmd;

use crate::{error::AocError, util::get_day};

pub async fn test(matches: &ArgMatches) -> Result<(), AocError> {
    let day = get_day(matches)?;
    let day = format!("day_{:02}", day);

    let reader = cmd!("cargo", "test", "--color", "always", "--", "--color", "always")
        .dir(day)
        .stderr_to_stdout()
        .reader()?;

    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    while let Some(Ok(line)) = lines.next() {
        println!("{}", line);
    }
    Ok(())
}
