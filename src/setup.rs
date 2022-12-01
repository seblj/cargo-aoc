use clap::ArgMatches;

use crate::{error::AocError, util::get_year};

async fn setup_template_project(year: i32) -> Result<(), AocError>
{
    tokio::process::Command::new("cargo")
        .args(["new", &format!("year_{}", year)])
        .output()
        .await?;

    let template = format!("{}/template/template.rs", env!("CARGO_MANIFEST_DIR"));
    for i in 1..=25
    {
        let dir = format!("year_{year}/src/bin/day_{:0>2}", i);
        tokio::fs::create_dir_all(&dir).await?;
        tokio::fs::copy(&template, format!("{dir}/main.rs")).await?;
    }
    tokio::fs::remove_file(format!("year_{year}/src/main.rs")).await?;
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

pub async fn setup(args: &ArgMatches) -> Result<(), AocError>
{
    let year = get_year(args)?;

    setup_template_project(year).await?;
    get_session_token().await?;
    Ok(())
}
