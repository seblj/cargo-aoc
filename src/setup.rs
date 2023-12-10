use std::path::Path;

use clap::ArgMatches;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::error::AocError;

async fn setup_template_project(year: i32) -> Result<(), AocError>
{
    let res = tokio::process::Command::new("cargo")
        .args(["new", &format!("year_{}", year)])
        .output()
        .await?;

    if !res.status.success()
    {
        return Err(AocError::SetupExists);
    }

    let template_dir = format!("{}/template", env!("CARGO_MANIFEST_DIR"));
    let bins = tokio::fs::read(Path::new(&template_dir).join("Cargo.toml.template")).await?;

    OpenOptions::new()
        .append(true)
        .open(format!("year_{}/Cargo.toml", year))
        .await?
        .write_all(&bins)
        .await?;

    for i in 1..=25
    {
        let dir = format!("year_{year}/day_{:0>2}", i);
        tokio::fs::create_dir_all(&dir).await?;
        tokio::fs::copy(Path::new(&template_dir).join("template.rs"), format!("{dir}/main.rs"))
            .await?;
    }
    tokio::fs::remove_dir_all(format!("year_{year}/src")).await?;
    Ok(())
}

async fn get_session_token() -> Result<(), AocError>
{
    if dotenv::var("AOC_TOKEN").is_err()
    {
        println!("Paste session token here for automatic download of input files");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if !input.is_empty()
        {
            let env_file = std::env::current_dir()?.join(".env");
            tokio::fs::write(env_file, format!("AOC_TOKEN={input}"))
                .await
                .expect("Couldn't write to file");
        }
    }
    Ok(())
}

fn get_year(matches: &ArgMatches) -> Result<i32, AocError>
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

pub async fn setup(args: &ArgMatches) -> Result<(), AocError>
{
    let year = get_year(args)?;

    setup_template_project(year).await?;
    get_session_token().await?;
    Ok(())
}
