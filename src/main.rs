use clap::Command;
mod setup;

fn main()
{
    let mut cmd = Command::new("cargo-aoc")
        .author("Sebastian, seblyng98@gmail.com")
        .author("Sivert, sivert-joh@hotmail.com")
        .subcommand(clap::command!("setup"));

    let help = cmd.render_help();
    let matches = cmd.get_matches();
    match matches.subcommand()
    {
        Some(("setup", matches)) => setup::setup(),
        _ =>
        {
            println!("{}", help);
        },
    }
}
