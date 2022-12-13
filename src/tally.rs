use std::{path::PathBuf, process::Output};

use clap::ArgMatches;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use tokio::task::JoinSet;

use crate::{error::AocError, util::file::*};

const TOO_MANY_OPEN_FILES: i32 = 24;

async fn days() -> Result<(Vec<(PathBuf, u32)>, Vec<u32>), AocError>
{
    let path = cargo_path().await?;

    let mut set = JoinSet::new();
    for day in 1..=25
    {
        let path = path.clone();
        set.spawn(async move {
            let path = day_path(path, day).await;
            (path, day)
        });
    }

    let mut have = Vec::new();
    let mut dont_have = Vec::new();
    while let Some(Ok(res)) = set.join_next().await
    {
        // The day path exists
        match res
        {
            (Ok(path), day) => have.push((path, day)),
            (_, day) => dont_have.push(day),
        }
    }


    Ok((have, dont_have))
}

async fn build_day(day: u32) -> Result<(), AocError>
{
    let bin = format!("day_{:02}", day);
    let res = tokio::process::Command::new("cargo")
        .args(["build", "--release", "--bin", &bin])
        .output()
        .await?;

    if !res.status.success()
    {
        Err(AocError::BuildError(bin.to_string()))
    }
    else
    {
        Ok(())
    }
}

async fn build_days(cargo_folder: PathBuf, days: &[(PathBuf, u32)]) -> Result<Vec<u32>, AocError>
{
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {msg}... {bar:40.cyan/blue} {pos:>7}/{len:7}",
    )
    .unwrap()
    .progress_chars("##-");

    let progress = ProgressBar::new(days.len() as u64);
    progress.set_style(sty);
    progress.set_message("compiling");

    days.into_par_iter().try_for_each(|(_path, day)| {
        let bin = format!("day_{:02}", day);
        let res = std::process::Command::new("cargo")
            .args(["build", "--release", "--bin", &bin])
            .output()?;

        progress.inc(1);
        if !res.status.success()
        {
            Err(AocError::BuildError(bin.to_string()))
        }
        else
        {
            Ok(())
        }
    })?;

    progress.reset();
    progress.set_message("verifying days");

    Ok(days
        .into_par_iter()
        .flat_map(|(pb, day)| {
            let pb = pb.clone();
            let day = *day;
            let bin = format!("day_{:02}", day);
            let mut target = cargo_folder.clone();
            target.push("target/release");
            target.push(&bin);

            let res = std::process::Command::new(target).current_dir(&pb).output();
            progress.inc(1);
            res.map_err(|_| day).err()
        })
        .collect())
}

fn parse_get_times(output: Output) -> Result<(usize, Option<usize>), AocError>
{
    let parse = |line: &str| -> Result<usize, AocError> {
        let start = line.find('(').ok_or_else(|| AocError::ParseStdout)?;
        let stop = line.find("ms)").ok_or_else(|| AocError::ParseStdout)?;
        Ok(line[start + 1..stop].parse().unwrap())
    };
    let text = std::str::from_utf8(&output.stdout).unwrap();
    let mut iter = text.split('\n');
    let p1 = parse(iter.next().unwrap())?;
    let p2 = iter.next().and_then(|n| parse(n).ok());

    Ok((p1, p2))
}

fn run_day(
    cargo_folder: PathBuf,
    day: (PathBuf, u32),
    number_of_runs: usize,
    progress: ProgressBar,
) -> Result<(usize, Option<usize>), AocError>
{
    let bin = format!("day_{:02}", day.1);
    let mut target = cargo_folder;
    target.push("target/release");
    target.push(&bin);
    let mut vec = Vec::with_capacity(number_of_runs);

    for _ in 0..number_of_runs
    {
        let dir = day.0.clone();
        let target = target.clone();
        let res = std::process::Command::new(target).current_dir(dir).output()?;
        if !res.status.success()
        {
            return Err(AocError::RunError(format!("Error running day {}", day.1)));
        }
        progress.inc(1);
        vec.push(parse_get_times(res)?);
    }

    let len = vec.len();
    let (p1, p2): (usize, Option<usize>) =
        vec.into_iter().fold((0, Option::<usize>::None), |(p1, p2), (a, b)| {
            (p1 + a, match (p2, b)
            {
                (Some(a), Some(b)) => Some(a + b),
                (None, Some(b)) => Some(b),
                _ => None,
            })
        });

    Ok((p1 / len, p2.map(|val| val / len)))
}

async fn run_days(
    days: Vec<(PathBuf, u32)>,
    number_of_runs: usize,
) -> Result<Vec<(u32, (usize, Option<usize>))>, AocError>
{
    let cargo_folder = cargo_path().await?;
    let multi = MultiProgress::new();
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {msg}... {bar:40.cyan/blue} {pos:>7}/{len:7}",
    )
    .unwrap()
    .progress_chars("##-");

    // Sort it to get the progress bars in increasing order
    let mut days = days;
    days.sort_unstable_by_key(|k| k.1);

    let v = std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(days.len());
        for day in days
        {
            let sty = sty.clone();
            let cargo_folder = cargo_folder.clone();
            let progress = multi.add(ProgressBar::new(number_of_runs as u64));
            let d = day.1;
            progress.set_style(sty);
            progress.set_message(format!("Running day {}", d));

            handles.push(s.spawn(move || {
                let res =
                    run_day(cargo_folder, day, number_of_runs, progress).expect("Running day");
                (d, res)
            }));
        }
        handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<_>>()
    });

    Ok(v)
}

fn print_info(days: Vec<(u32, (usize, Option<usize>))>, not_done: Vec<u32>, number_of_runs: usize)
{
    let red_text = |s: u32| format!("\x1b[0;33;31m{}\x1b[0m", s);
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

    let print_info = |text: String, vec: Vec<(u32, usize)>| {
        println!("{}", text);

        let mut _vec: Vec<_> = vec.iter().map(|(_, time)| *time).collect();
        _vec.sort_unstable();

        let median = _vec[_vec.len() / 2];

        let total = vec.iter().map(|(_, time)| time).sum::<usize>();
        let avg = total / vec.len();

        let (highest_day, highest_time) = vec.iter().max_by_key(|k| k.1).unwrap();

        println!("\t Total time:  \t{}ms", total);
        println!("\t Average time:\t{}ms", avg);
        println!("\t Median time: \t{}ms", median);
        println!("\t Highest time:\t{}ms, day: {}", highest_time, highest_day);
        println!();
    };

    let silver = days.iter().cloned().map(|(day, (p1, _))| (day, p1)).collect::<Vec<_>>();
    let gold = days
        .iter()
        .cloned()
        .filter_map(|(day, (_, p2))| p2.map(|p2| (day, p2)))
        .collect::<Vec<_>>();

    let total = gold.iter().chain(silver.iter()).map(|(_, time)| time).sum::<usize>();

    print_info(silver_text("Silver"), silver);
    print_info(gold_text("Gold"), gold);
    println!("\nTOTAL TIME: {}ms", total);
}

pub async fn tally(matches: &ArgMatches) -> Result<(), AocError>
{
    let number_of_runs: usize =
        matches.get_one::<String>("runs").ok_or(AocError::ArgMatches)?.parse()?;
    let cargo_folder = cargo_path().await?;

    let (mut have, mut dont) = days().await?;
    let unimplementeds = build_days(cargo_folder, &have).await?;

    have.retain(|elem| !unimplementeds.contains(&elem.1));
    for unimpl in unimplementeds
    {
        dont.push(unimpl);
    }

    let mut res = run_days(have, number_of_runs).await?;
    res.sort_unstable_by_key(|v| v.0);

    print_info(res, dont, number_of_runs);


    Ok(())
}
