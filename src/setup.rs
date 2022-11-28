use clap::ArgMatches;

async fn setup_template_project(year: i32) -> Result<(), Box<dyn std::error::Error>>
{
    tokio::process::Command::new("cargo")
        .args(&["new", &format!("year_{}", year)])
        .output()
        .await?;

    let template = format!("{}/template/template.rs", env!("CARGO_MANIFEST_DIR"));
    for i in 1..=25
    {
        let dir = format!("year_{year}/src/bin/day_{:0>2}", i);
        tokio::fs::create_dir_all(&dir).await?;
        tokio::fs::copy(&template, format!("{dir}/main.rs")).await?;
    }
    Ok(())
}

async fn get_session_token() -> Result<(), Box<dyn std::error::Error>>
{
    dotenv::dotenv().ok();
    if dotenv::var("AOC_TOKEN").is_err()
    {
        println!("Paste session token here for automatic download of input files");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if !input.is_empty()
        {
            tokio::fs::write(
                format!("{}/.env", env!("CARGO_MANIFEST_DIR")),
                format!("AOC_TOKEN={input}"),
            )
            .await
            .expect("Couldn't write to file");
        }
    }
    Ok(())
}

pub async fn setup(args: &ArgMatches) -> Result<(), Box<dyn std::error::Error>>
{
    let year = args
        .get_one::<String>("year")
        .expect("No year specified")
        .parse::<i32>()
        .expect("Invalid year");

    setup_template_project(year).await?;
    get_session_token().await?;
    Ok(())
}
