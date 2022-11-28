use chrono::Datelike;
use clap::{builder::OsStr, Arg, Command};
mod setup;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let mut cmd = Command::new("cargo-aoc")
        .author("Sebastian, seblyng98@gmail.com")
        .author("Sivert, sivert-joh@hotmail.com")
        .subcommand(
            clap::command!("setup").arg(
                Arg::new("year")
                    .short('y')
                    .default_value(OsStr::from(chrono::Utc::now().year().to_string())),
            ),
        );

    let help = cmd.render_help();
    let matches = cmd.get_matches();
    match matches.subcommand()
    {
        Some(("setup", matches)) =>
        {
            setup::setup(matches).await.expect("Couldn't setup project properly")
        },
        _ =>
        {
            println!("{}", help);
        },
    }
    Ok(())
}
