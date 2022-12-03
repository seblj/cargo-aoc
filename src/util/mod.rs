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
