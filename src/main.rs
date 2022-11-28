use clap::{Command, Parser};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args
{
    #[arg(short, long)]
    day: Option<u8>,
}

fn main()
{
    let cmd = Command::new("cargo-aoc")
        .author("Sebastian, seblyng98@gmail.com")
        .author("Sivert, sivert-joh@hotmail.com");
}
