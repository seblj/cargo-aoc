use clap::ArgMatches;

pub mod file;

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
