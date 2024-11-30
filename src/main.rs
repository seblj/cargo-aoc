use chrono::Datelike;
use clap::{builder::OsStr, Arg, Command};
use error::AocError;
mod assert;
#[cfg(feature = "bench")]
mod bench;
mod clippy;
mod error;
mod run;
mod setup;
#[cfg(feature = "tally")]
mod tally;
mod test;
mod token;
mod util;

#[tokio::main]
async fn main() -> Result<(), AocError> {
    dotenv::dotenv().ok();
    let mut cmd = Command::new("cargo-aoc")
        .author("Sebastian, sebastian@lyngjohansen.com")
        .author("Sivert, sivert-joh@hotmail.com")
        .arg(Arg::new("dummy").hide(true))
        .subcommand(
            clap::command!("setup")
                .arg(
                    Arg::new("year")
                        .short('y')
                        .default_value(OsStr::from(chrono::Utc::now().year().to_string()))
                        .help("Year to setup folder structure for"),
                )
                .about(
                    "Setup folder structure and asks for session token for automatic input \
                     download",
                ),
        )
        .subcommand(
            clap::command!("clippy")
                .disable_version_flag(true)
                .about("Run cargo clippy on the specified day")
                .args([
                    Arg::new("day")
                        .short('d')
                        .required(false)
                        .default_value(OsStr::from(chrono::Utc::now().day().to_string()))
                        .help("Day to check"),
                    Arg::new("fix")
                        .long("fix")
                        .required(false)
                        .action(clap::ArgAction::SetTrue)
                        .help("Fixes the issues clippy warns about"),
                ]),
        )
        .subcommand(
            clap::command!("run")
                .visible_alias("r")
                .args([
                    Arg::new("day")
                        .short('d')
                        .required(chrono::Utc::now().day() > 25)
                        .default_value(OsStr::from(chrono::Utc::now().day().to_string()))
                        .help("Day to run"),
                    Arg::new("test")
                        .short('t')
                        .long("test")
                        .required(false)
                        .action(clap::ArgAction::SetTrue)
                        .help("Run the day with the \"test\" file"),
                    Arg::new("assert")
                        .short('a')
                        .long("assert")
                        .required(false)
                        .action(clap::ArgAction::SetTrue)
                        .help("Asserts that the answers are still correct after submitting"),
                    Arg::new("compiler-flags")
                        .short('C')
                        .long("compiler-flags")
                        .required(false)
                        .default_value(std::env::var("RUSTFLAGS").unwrap_or_default())
                        .allow_hyphen_values(true)
                        .help("Flags to send to rustc"),
                    #[cfg(feature = "submit")]
                    Arg::new("submit")
                        .short('S')
                        .long("submit")
                        .required(false)
                        .help("Submit answer")
                        .conflicts_with("test"),
                ])
                .about("Runs the given day"),
        )
        .subcommand(
            clap::command!("test").args([Arg::new("day")
                .short('d')
                .required(false)
                .default_value(OsStr::from(chrono::Utc::now().day().to_string()))
                .help("Day to run tests for")]),
        )
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

    #[cfg(feature = "tally")]
    {
        cmd = cmd.subcommand(
            Command::new("tally")
                .about(
                    "Tallies the  performance of each day and displays information about the \
                        performance",
                )
                .arg(
                    Arg::new("runs")
                        .long("num-runs")
                        .help("Number of runs")
                        .default_value("10"),
                ),
        );
    }

    #[cfg(feature = "bench")]
    {
        cmd = cmd.subcommand(
            Command::new("bench")
                .about("Run benchmarks for the specified day")
                .args([
                    Arg::new("day")
                        .help("The day to benchmark")
                        .short('d')
                        .default_value(chrono::Utc::now().day().to_string()),
                    Arg::new("output")
                        .help("Output location")
                        .short('o')
                        .long("output")
                        .required(false),
                ]),
        );
    }

    let help = cmd.render_help();
    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("setup", matches)) => setup::setup(matches)
            .await
            .expect("Couldn't setup project properly"),
        Some(("run", matches)) => run::run(matches).await?,
        Some(("test", matches)) => test::test(matches).await?,
        Some(("token", matches)) => token::token(matches).await?,
        Some(("clippy", matches)) => clippy::clippy(matches).await?,

        #[cfg(feature = "bench")]
        Some(("bench", matches)) => bench::bench(matches).await?,

        #[cfg(feature = "tally")]
        Some(("tally", matches)) => tally::tally(matches).await?,
        _ => {
            println!("{}", help);
        }
    }
    Ok(())
}
