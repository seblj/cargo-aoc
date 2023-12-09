use std::{
    path::PathBuf,
    process::{Command, Output},
};

use chrono::Datelike;
use clap::ArgMatches;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::task::JoinSet;

use crate::{
    error::AocError,
    util::{file::*, get_day_title_and_answers, get_time_symbol},
};

// Helper function to:
// 1. iterate over a collection
// 2. spawn a scoped thread for each item
// 3. map each item T to a new type U
// 4. collect the items into some R
fn thread_exec<T, U, I, F, R>(iter: I, f: F) -> R
where
    F: Fn(T) -> U + Send + Clone + Copy,
    R: FromIterator<U>,
    U: Send,
    T: Send,
    I: IntoIterator<Item = T>,
{
    // Collecting the JoinHandles are very important to actually spawn the threads.
    // Removing the collect results in sequential execution
    #[allow(clippy::needless_collect)]
    std::thread::scope(|s| {
        iter.into_iter()
            .map(|v| s.spawn(move || f(v)))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<R>()
    })
}

fn get_progressbar(len: u64) -> ProgressBar
{
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {msg}... {bar:40.cyan/blue} {pos:>7}/{len:7}",
    )
    .unwrap()
    .progress_chars("##-");

    ProgressBar::new(len).with_style(sty)
}

fn parse_get_times(output: Output) -> Result<(usize, Option<usize>), AocError>
{
    let unit = get_time_symbol();
    let parse = |line: &str| -> Result<usize, AocError> {
        let start = line.find('(').ok_or(AocError::ParseStdout)?;
        let stop = line.find(&format!("{unit})")).ok_or(AocError::ParseStdout)?;
        Ok(line[start + 1..stop].parse().unwrap())
    };
    let text = std::str::from_utf8(&output.stdout).unwrap();
    let mut iter = text.split('\n');
    let p1 = parse(iter.next().unwrap())?;
    let p2 = iter.next().and_then(|n| parse(n).ok());

    Ok((p1, p2))
}

fn parse_get_answers(output: Output) -> (Option<String>, Option<String>)
{
    let text = std::str::from_utf8(&output.stdout).unwrap();
    let strip = strip_ansi_escapes::strip(text);
    let text = std::str::from_utf8(&strip).unwrap();

    let parse = |line: &str| line.split_ascii_whitespace().rev().next().map(|s| s.to_string());
    let mut iter = text.split('\n');
    (iter.next().and_then(parse), iter.next().and_then(parse))
}
fn print_info(
    days: Vec<(usize, (usize, Option<usize>))>,
    not_done: Vec<usize>,
    number_of_runs: usize,
)
{
    let unit = get_time_symbol();
    let red_text = |s: usize| format!("\x1b[0;33;31m{}\x1b[0m", s);
    let gold_text = |s: &str| format!("\x1b[0;33;10m{}\x1b[0m:", s);
    let silver_text = |s: &str| format!("\x1b[0;34;34m{}\x1b[0m:", s);

    if !not_done.is_empty()
    {
        let mut not_done = not_done;
        not_done.sort_unstable();
        let mut s = String::new();
        let mut first = true;
        for day in not_done
        {
            if !first
            {
                s.push_str(", ");
            }
            s.push_str(&red_text(day));
            first = false;
        }
        println!("Days not completed: {}", s);
    }
    println!("STATS:");
    println!("Number of runs: {}:\n", number_of_runs);

    let print_info = |text: String, vec: Vec<(usize, usize)>| {
        println!("{}", text);

        let mut data: Vec<_> = vec.iter().map(|(_, time)| *time).collect();
        data.sort_unstable();

        let median = data[data.len() / 2];

        let total = vec.iter().map(|(_, time)| time).sum::<usize>();
        let avg = total / vec.len();

        let (highest_day, highest_time) = vec.iter().max_by_key(|k| k.1).unwrap();

        println!("\t Total time:  \t{}{unit}", total);
        println!("\t Average time:\t{}{unit}", avg);
        println!("\t Median time: \t{}{unit}", median);
        println!("\t Highest time:\t{}{unit}, day: {}", highest_time, highest_day);
        println!();
    };

    let silver = days.iter().map(|(day, (p1, _))| (*day, *p1)).collect::<Vec<_>>();
    let gold = days
        .iter()
        .filter_map(|(day, (_, p2))| p2.map(|p2| (*day, p2)))
        .collect::<Vec<_>>();

    let total = gold.iter().chain(silver.iter()).map(|(_, time)| time).sum::<usize>();

    print_info(silver_text("Silver"), silver);
    print_info(gold_text("Gold"), gold);
    let unit = get_time_symbol();
    println!("\nTOTAL TIME: {}{unit}", total);
}

fn get_year(matches: &ArgMatches) -> Result<usize, AocError>
{
    Ok(matches.get_one::<String>("year").ok_or(AocError::ArgMatches)?.parse()?)
}

fn get_number_of_runs(matches: &ArgMatches) -> Result<usize, AocError>
{
    Ok(matches.get_one::<String>("runs").ok_or(AocError::ArgMatches)?.parse()?)
}

fn get_possible_days(year: usize) -> Result<Vec<usize>, AocError>
{
    let now = chrono::Utc::now();

    if year as i32 == now.year()
    {
        if now.month() < 12
        {
            Err(AocError::InvalidMonth)
        }
        else
        {
            Ok((1..=now.day() as usize).collect())
        }
    }
    else
    {
        Ok((1..=25).collect())
    }
}

#[derive(Debug, Default)]
struct TableInfo
{
    title: String,
    ans1:  Option<String>,
    ans2:  Option<String>,

    correct1: bool,
    correct2: bool,
}

#[derive(Debug, Default)]
struct Time(usize, Option<usize>);

#[derive(Debug)]
struct BuildRes
{
    day:  usize,
    path: PathBuf,
    info: TableInfo,
    time: Time,
}

impl BuildRes
{
    fn new(day: usize, path: PathBuf) -> Self
    {
        Self {
            day,
            path,
            info: Default::default(),
            time: Default::default(),
        }
    }
}

fn build_day(day: usize, path: PathBuf, progress: &ProgressBar) -> Option<usize>
{
    let bin = format!("day_{:02}", day);
    let res = Command::new("cargo")
        .args(["build", "--release", "--bin", &bin])
        .current_dir(path)
        .output()
        .ok()?;

    progress.inc(1);
    if res.status.success()
    {
        Some(day)
    }
    else
    {
        None // Build errors are considered 'not have'
    }
}

async fn verify_day(
    day: usize,
    path: PathBuf,
    year: usize,
    table_display: bool,
    progress: &ProgressBar,
) -> Option<BuildRes>
{
    let day_path = day_path(path.clone(), day as u32)
        .await
        .expect(&format!("day {day} is build, but could not find the path"));

    let bin = format!("day_{:02}", day);
    let mut target = path;
    target.push("target/release");
    target.push(&bin);
    let progress = progress.clone();

    let res = Command::new(target).current_dir(&day_path).output().ok()?;

    let (_t1, _t2) = parse_get_answers(res);
    if _t1.is_none() && _t2.is_none()
    {
        return None;
    }

    let mut br = BuildRes::new(day, day_path);
    let res = if table_display
    {
        get_day_title_and_answers(day as u32, year as u32).await.ok().map(
            |(title, task1, task2)| {
                br.info.title = title;

                br.info.correct1 = _t1 == task1;
                br.info.correct2 = _t2 == task2;

                br.info.ans1 = task1;
                br.info.ans2 = task2;

                br
            },
        )
    }
    else
    {
        Some(br)
    };
    progress.inc(1);
    res
}

async fn compile_and_verify_days(
    days: Vec<usize>,
    cargo_folder: PathBuf,
    year: usize,
    table_display: bool,
) -> Vec<BuildRes>
{
    let progress = get_progressbar(days.len() as u64);
    progress.set_message("compiling");

    let res: Vec<Option<usize>> = thread_exec(&days, |day| {
        let day = *day;
        let path = cargo_folder.clone();
        build_day(day, path, &progress)
    });

    progress.reset();
    progress.set_message("verifying");

    let days: Vec<Option<BuildRes>> = thread_exec(res.into_iter().flatten(), |day| {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(verify_day(day, cargo_folder.clone(), year, table_display, &progress))
    });

    days.into_iter().flatten().collect()
}

fn run_day(
    cargo_folder: PathBuf,
    day_folder: PathBuf,
    day: usize,
    number_of_runs: usize,
    progress: ProgressBar,
) -> Result<(usize, Option<usize>), AocError>
{
    let bin = format!("day_{:02}", day);
    let mut target = cargo_folder;
    target.push("target/release");
    target.push(&bin);
    let mut vec = Vec::with_capacity(number_of_runs);

    for _ in 0..number_of_runs
    {
        let res = Command::new(&target).current_dir(&day_folder).envs(std::env::vars()).output()?;

        progress.inc(1);
        vec.push(parse_get_times(res)?);
    }

    let len = vec.len();
    let (p1, p2): (usize, Option<usize>) =
        vec.into_iter().fold((0, Option::<usize>::None), |(p1, p2), (a, b)| {
            (p1 + a, p2.zip(b).map(|(p2, b)| p2 + b).or(b))
        });

    Ok((p1 / len, p2.map(|val| val / len)))
}

fn run_days(
    days: Vec<BuildRes>,
    cargo_folder: PathBuf,
    number_of_runs: usize,
) -> Result<Vec<BuildRes>, AocError>
{
    let multi = MultiProgress::new();

    // Sort it to get the progress bars in increasing order
    let mut days = days;
    days.sort_unstable_by_key(|k| k.day);
    let days = days
        .into_iter()
        .map(|br| {
            let progress = multi.add(get_progressbar(number_of_runs as u64));
            progress.set_message(format!("Running day {}", br.day));
            (br, progress)
        })
        .collect::<Vec<_>>();

    Ok(thread_exec(days, |(br, progress)| {
        let cargo_folder = cargo_folder.clone();

        let (p1, p2) = run_day(cargo_folder, br.path.clone(), br.day, number_of_runs, progress)
            .expect(&format!("error running day {}", br.day));
        let mut br = br;
        br.time = Time(p1, p2);

        br
    }))
}

fn display_table(matches: &ArgMatches) -> bool
{
    matches.get_flag("table")
}

fn format_duration(duration: usize) -> String
{
    let unit = get_time_symbol();
    format!("{}{}", duration, unit)
}

fn print_table(days: Vec<BuildRes>)
{
    let max_name_len = days.iter().map(|br| br.info.title.len()).max().unwrap();
    let max_part1_len = days.iter().map(|br| br.info.ans1.as_ref().unwrap().len()).max().unwrap();
    let max_part2_len = days.iter().map(|br| br.info.ans2.as_ref().unwrap().len()).max().unwrap();

    let max_part1_time_len = days.iter().map(|br| format_duration(br.time.0).len()).max().unwrap();
    let max_part2_time_len = days
        .iter()
        .map(|br| br.time.1.map(|t| format_duration(t)).unwrap_or("NA".to_string()).len())
        .max()
        .unwrap();


    let part1_header_len = max_part1_len + 8 + max_part1_time_len;
    let part2_header_len = max_part2_len + 8 + max_part2_time_len;

    let max_total_len = max_name_len + part1_header_len + part2_header_len + 3;

    println!("╔{}╗", "═".repeat(max_total_len + 5));
    println!("║ {:^max_total_len$}  ║", "🦀 Advent of Code 2023 🦀");
    println!(
        "╠{}╦{}╦{}╣",
        "═".repeat(max_name_len + 2),
        "═".repeat(part1_header_len + 2),
        "═".repeat(part2_header_len + 2),
    );
    println!(
        "║ {:max_name_len$} ║ {:part1_header_len$} ║ {:part2_header_len$} ║",
        "Day", "Part 1", "Part 2"
    );
    println!(
        "╠{}╬{}╦{}╦{}╬{}╦{}╦{}╣",
        "═".repeat(max_name_len + 2),
        "═".repeat(max_part1_len + 2),
        "═".repeat(max_part1_time_len + 2),
        "═".repeat(4),
        "═".repeat(max_part2_len + 2),
        "═".repeat(max_part2_time_len + 2),
        "═".repeat(4),
    );

    for day in days
    {
        let part1_symbol = if day.info.correct1 { "✅" } else { "❌" };
        let part2_symbol = if day.info.correct2 { "✅" } else { "❌" };

        println!(
            "║ {:max_name_len$} ║ {:max_part1_len$} ║ {:max_part1_time_len$} ║ {} ║ \
             {:max_part2_len$} ║ {:max_part2_time_len$} ║ {} ║ ",
            day.info.title,
            day.info.ans1.unwrap(),
            format_duration(day.time.0),
            part1_symbol,
            day.info.ans2.unwrap(),
            day.time.1.map(|t| format_duration(t)).unwrap_or("NA".to_string()),
            part2_symbol,
        );
    }
    println!(
        "╚{}╩{}╩{}╩{}╩{}╩{}╩{}╝",
        "═".repeat(max_name_len + 2),
        "═".repeat(max_part1_len + 2),
        "═".repeat(max_part1_time_len + 2),
        "═".repeat(4),
        "═".repeat(max_part2_len + 2),
        "═".repeat(max_part2_time_len + 2),
        "═".repeat(4),
    );
}


pub async fn tally(matches: &ArgMatches) -> Result<(), AocError>
{
    let number_of_runs = get_number_of_runs(&matches)?;
    let display_table = display_table(&matches);

    let cargo_folder = cargo_path().await?;
    let year = get_year(&matches)?;
    let possible_days = get_possible_days(year)?;
    let days =
        compile_and_verify_days(possible_days.clone(), cargo_folder.clone(), year, display_table)
            .await;

    let days = run_days(days, cargo_folder, number_of_runs)?;

    let dont_have = possible_days
        .into_iter()
        .filter(|day| !days.iter().any(|br| br.day == *day))
        .collect();


    if display_table
    {
        print_table(days);
    }
    else
    {
        let have = days.into_iter().map(|br| (br.day, (br.time.0, br.time.1))).collect();
        print_info(have, dont_have, number_of_runs);
    }


    Ok(())
}
