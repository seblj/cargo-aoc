use chrono::Datelike;
use clap::{builder::OsStr, Arg, Command};
mod run;
mod setup;
mod util;

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
        )
        .subcommand(clap::command!("run").args([Arg::new("day").short('d').required(false)]));

    let help = cmd.render_help();
    let matches = cmd.get_matches();
    match matches.subcommand()
    {
        Some(("setup", matches)) =>
        {
            setup::setup(matches).await.expect("Couldn't setup project properly")
        },
        Some(("run", matches)) => run::run(matches).await?,
        _ =>
        {
            println!("{}", help);
        },
    }
    Ok(())
}
