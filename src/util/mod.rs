use clap::ArgMatches;

use crate::error::AocError;

pub mod file;
pub mod request;
pub mod submit;

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
