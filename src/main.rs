use chrono::Datelike;
use clap::{builder::OsStr, Arg, Command};
mod run;
mod setup;
mod token;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    dotenv::dotenv().ok();
    let mut cmd = Command::new("cargo-aoc")
        .author("Sebastian, seblyng98@gmail.com")
        .author("Sivert, sivert-joh@hotmail.com")
        .arg(Arg::new("dummy").hide(true))
        .subcommand(
            clap::command!("setup").arg(
                Arg::new("year")
                    .short('y')
                    .default_value(OsStr::from(chrono::Utc::now().year().to_string())),
            ),
        )
        .subcommand(clap::command!("run").args([
            Arg::new("day").short('d').required(false),
            Arg::new("year").short('y').required(false),
        ]))
        .subcommand(
            Command::new("token")
                .about("Get or set the session token used to communicate with the AOC servers")
                .arg_required_else_help(true)
                .args([
                    Arg::new("set")
                        .short('s')
                        .long("set")
                        .exclusive(true)
                        .help("Set the session token"),
                    Arg::new("get")
                        .short('g')
                        .long("get")
                        .exclusive(true)
                        .num_args(0)
                        .help("Print the current session token, if any"),
                ]),
        );

    let help = cmd.render_help();
    let matches = cmd.get_matches();
    match matches.subcommand()
    {
        Some(("setup", matches)) =>
        {
            setup::setup(matches).await.expect("Couldn't setup project properly")
        },
        Some(("run", matches)) => run::run(matches).await?,
        Some(("token", matches)) => token::token(matches).await,
        _ =>
        {
            println!("{}", help);
        },
    }
    Ok(())
}
