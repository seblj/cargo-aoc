# What is this?

`cargo-aoc`is a cargo subcommand `cargo aoc` for setting up and running advent of code days. :santa:

## Features :star2:

- Automatically download input files
- Generate AOC rust project structure
- Automatically submit answer

## Installations

Currently, this is only distributed on github. You can install it with

```
cargo install --path .
```

## Crate/Project Requirements

In order to use this tool your crate/project need to have

- Have a [binary](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries) for each day on the form `day_xx` (e.g, `day_01`, `day_23`)
- Have a `.env` file containing the variable `AOC_TOKEN=<your token>`. Tokens can we found by inspecting a network request on the advent of code site (while logged in) and grabbing the cookie session number.

The `setup` subcommand can be used to generate a valid project structure; however, you still need to get your session number.

```
Usage: cargo-aoc [COMMAND]

Commands:
  setup   Setup folder structure and asks for session token for automatic input download
  clippy  Run cargo clippy on the specified day
  run     Runs the given day [aliases: r]
  token   Get or set the session token used to communicate with the AOC servers
  tally   Tallies the  performance of each day and displays information about the performance
  bench   Run benchmarks for the specified day
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help information
```
