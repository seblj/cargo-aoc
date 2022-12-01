use std::fmt::Display;

use clap::ArgMatches;

pub mod file;
pub mod submit;

#[derive(Debug)]
pub enum ParseArgError
{
    ParseError,
    TypeError,
    Invalid(String),
}

impl Display for ParseArgError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            ParseArgError::TypeError => write!(f, "Incorrect type found"),
            ParseArgError::ParseError =>
            {
                write!(f, "Couldn't parse input. Check that you are using the correct type")
            },
            ParseArgError::Invalid(s) => write!(f, "{}", s),
        }
    }
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
