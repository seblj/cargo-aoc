use std::{path::PathBuf, process::Command};

use clap::ArgMatches;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::runtime::Runtime;

use crate::{
    error::AocError,
    util::{file::*, get_day_title_and_answers, get_time_symbol},
};

use crate::util::tally_util::*;

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

fn get_progressbar(len: u64) -> ProgressBar {
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {msg}... {bar:40.cyan/blue} {pos:>7}/{len:7}",
    )
    .unwrap()
    .progress_chars("##-");

    ProgressBar::new(len).with_style(sty)
}

fn print_info(
    days: Vec<(usize, (usize, Option<usize>))>,
    not_done: Vec<usize>,
    number_of_runs: usize,
) {
    let unit = get_time_symbol();
    let red_text = |s: usize| format!("\x1b[0;33;31m{}\x1b[0m", s);
    let gold_text = |s: &str| format!("\x1b[0;33;10m{}\x1b[0m:", s);
    let silver_text = |s: &str| format!("\x1b[0;34;34m{}\x1b[0m:", s);

    if !not_done.is_empty() {
        let mut not_done = not_done;
        not_done.sort_unstable();
        let mut s = String::new();
        let mut first = true;
        for day in not_done {
            if !first {
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
        println!(
            "\t Highest time:\t{}{unit}, day: {}",
            highest_time, highest_day
        );
        println!();
    };

    let silver = days
        .iter()
        .map(|(day, (p1, _))| (*day, *p1))
        .collect::<Vec<_>>();
    let gold = days
        .iter()
        .filter_map(|(day, (_, p2))| p2.map(|p2| (*day, p2)))
        .collect::<Vec<_>>();

    let total = gold
        .iter()
        .chain(silver.iter())
        .map(|(_, time)| time)
        .sum::<usize>();

    print_info(silver_text("Silver"), silver);
    print_info(gold_text("Gold"), gold);
    let unit = get_time_symbol();
    println!("\nTOTAL TIME: {}{unit}", total);
}

fn build_day(
    day: usize,
    path: PathBuf,
    progress: &ProgressBar,
    year: usize,
) -> Result<usize, Error> {
    let mut day_path = path.clone();
    day_path.push(format!("day_{:02}", day));

    if !day_path.exists() {
        let runtime = Runtime::new().unwrap();
        let info = runtime
            .block_on(get_day_title_and_answers(day as u32, year as u32))
            .expect("Could not get day title and answer");
        return Err(Error {
            title: info.title,
            day,
            r#type: ErrorTypes::NotImplementd,
        });
    }

    let res = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(day_path)
        .output()
        .ok()
        .unwrap();

    progress.inc(1);
    if res.status.success() {
        Ok(day)
    } else {
        let details = extract_comiler_error(String::from_utf8(res.stderr).unwrap());
        let runtime = Runtime::new().unwrap();
        let info = runtime
            .block_on(get_day_title_and_answers(day as u32, year as u32))
            .expect("Could not get day title and answer");
        Err(Error {
            title: info.title,
            day,
            r#type: ErrorTypes::CompilerError(details),
        })
    }
}

async fn verify_day(
    day: usize,
    path: PathBuf,
    year: usize,
    progress: &ProgressBar,
) -> Result<BuildRes, Error> {
    let day_path = day_path(path.clone(), day as u32)
        .await
        .unwrap_or_else(|_| panic!("day {day} is build, but could not find the path"));

    let info = get_day_title_and_answers(day as u32, year as u32)
        .await
        .expect("Could not get day title and answer");

    let mut input = day_path.clone();
    input.push("input");
    if !input.exists() {
        download_input_file(day as u32, year as i32, &day_path)
            .await
            .map_err(|_| Error {
                title: info.title.clone(),
                day,
                r#type: ErrorTypes::InputDownloadError,
            })?;
    }

    let target = get_target(path, day);
    let progress = progress.clone();

    let res = Command::new(target)
        .current_dir(&day_path)
        .output()
        .ok()
        .unwrap();

    if !res.status.success() {
        let details = extract_runtime_error(res.stderr);
        if details == "not implemented" {
            return Err(Error {
                title: info.title.clone(),
                day,
                r#type: ErrorTypes::NotImplementd,
            });
        }

        return Err(Error {
            title: info.title.clone(),
            day,
            r#type: ErrorTypes::RuntimeError(details),
        });
    }

    let (_t1, _t2) = parse_get_answers(res);
    if _t1.is_none() && _t2.is_none() {
        return Err(Error {
            title: info.title.clone(),
            day,
            r#type: ErrorTypes::NotImplementd,
        });
    }

    let mut res = BuildRes::new(day, day_path);
    res.info.title = info.title;

    res.info.correct1 = _t1 == info.part1_answer;
    res.info.correct2 = _t2 == info.part2_answer;

    res.info.ans1 = info.part1_answer;
    res.info.ans2 = info.part2_answer;

    progress.inc(1);

    Ok(res)
}

async fn compile_and_verify_days(
    days: Vec<usize>,
    cargo_folder: PathBuf,
    year: usize,
) -> Result<Vec<Result<BuildRes, Error>>, AocError> {
    let possible_days = filter_days_based_on_folder(&days, &cargo_folder)?;

    let progress = get_progressbar(possible_days.len() as u64);
    progress.set_message("compiling");

    let res: Vec<_> = thread_exec(&days, |day| {
        build_day(*day, cargo_folder.clone(), &progress, year)
    });

    progress.reset();
    progress.set_message("verifying");

    let days: Vec<_> = thread_exec(res, |day| {
        day.and_then(|day| {
            let runtime = Runtime::new().unwrap();
            runtime.block_on(verify_day(day, cargo_folder.clone(), year, &progress))
        })
    });

    Ok(days)
}

fn run_day(
    cargo_folder: PathBuf,
    day_folder: PathBuf,
    day: usize,
    number_of_runs: usize,
    progress: ProgressBar,
) -> Result<(usize, Option<usize>), AocError> {
    let target = get_target(cargo_folder, day);
    let mut vec = Vec::with_capacity(number_of_runs);

    for _ in 0..number_of_runs {
        let res = Command::new(&target)
            .current_dir(&day_folder)
            .envs(std::env::vars())
            .output()?;

        progress.inc(1);
        vec.push(parse_get_times(res)?);
    }

    let len = vec.len();
    let (p1, p2): (usize, Option<usize>) = vec
        .into_iter()
        .fold((0, Option::<usize>::None), |(p1, p2), (a, b)| {
            (p1 + a, p2.zip(b).map(|(p2, b)| p2 + b).or(b))
        });

    Ok((p1 / len, p2.map(|val| val / len)))
}

fn run_days(
    days: Vec<Result<BuildRes, Error>>,
    cargo_folder: PathBuf,
    number_of_runs: usize,
) -> Result<Vec<Result<BuildRes, Error>>, AocError> {
    let multi = MultiProgress::new();

    // Sort it to get the progress bars in increasing order
    let mut days = days;
    days.sort_unstable_by_key(|k| match k {
        Ok(k) => k.day,
        Err(e) => e.day,
    });
    let days = days
        .into_iter()
        .map(|br| {
            br.map(|br| {
                let progress = multi.add(get_progressbar(number_of_runs as u64));
                progress.set_message(format!("Running day {}", br.day));
                (br, progress)
            })
        })
        .collect::<Vec<_>>();

    Ok(thread_exec(days, |res| {
        res.map(|(mut br, progress)| {
            let (p1, p2) = run_day(
                cargo_folder.clone(),
                br.path.clone(),
                br.day,
                number_of_runs,
                progress,
            )
            .unwrap_or_else(|_| panic!("error running day {}", br.day));
            br.time = Time(p1, p2);
            br
        })
    }))
}

fn format_duration(duration: usize) -> String {
    let unit = get_time_symbol();
    format!("{}{}", duration, unit)
}

fn print_table(days: Vec<Result<BuildRes, Error>>, year: usize) {
    /*let max_name_len = days
        .iter()
        .map(|res| match res {
            Ok(br) => br.info.title.len(),
            Err(err) => err.title.len(),
        })
        .max()
        .unwrap_or(5);
    let max_part1_len = days
        .iter()
        .flatten()
        .map(|br| br.info.ans1.as_ref().unwrap_or(&"NA".to_string()).len())
        .max()
        .unwrap_or(5);
    let max_part2_len = days
        .iter()
        .flatten()
        .map(|br| br.info.ans2.as_ref().unwrap_or(&"NA".to_string()).len())
        .max()
        .unwrap_or(5);

    let max_part1_time_len = days
        .iter()
        .flatten()
        .map(|br| format_duration(br.time.0).len())
        .max()
        .unwrap_or(5);
    let max_part2_time_len = days
        .iter()
        .flatten()
        .map(|br| {
            br.time
                .1
                .map(format_duration)
                .unwrap_or("NA".to_string())
                .len()
        })
        .max()
        .unwrap_or(5);

    let day_header_len = max_name_len + 5;
    let part1_header_len = max_part1_len + 8 + max_part1_time_len;
    let part2_header_len = max_part2_len + 8 + max_part2_time_len;

    let max_total_len = day_header_len + part1_header_len + part2_header_len + 5;
    let title_length = max_total_len - 2;

    println!("╔{}╗", "═".repeat(max_total_len + 3));
    println!(
        "║ {:^title_length$}  ║",
        format!("🦀 Advent of Code {year} 🦀")
    );
    println!(
        "╠{}╦{}╦{}╣",
        "═".repeat(day_header_len + 2),
        "═".repeat(part1_header_len + 2),
        "═".repeat(part2_header_len + 2),
    );
    println!(
        "║ {:day_header_len$} ║ {:part1_header_len$} ║ {:part2_header_len$} ║",
        "Day", "Part 1", "Part 2"
    );
    println!(
        "╠{}╦{}╬{}╦{}╦{}╬{}╦{}╦{}╣",
        "═".repeat(4),
        "═".repeat(max_name_len + 2),
        "═".repeat(max_part1_len + 2),
        "═".repeat(max_part1_time_len + 2),
        "═".repeat(4),
        "═".repeat(max_part2_len + 2),
        "═".repeat(max_part2_time_len + 2),
        "═".repeat(4),
    );

    for day in days {
        match day {
            Ok(day) => {
                let part1_symbol = if day.info.correct1 { "✅" } else { "❌" };
                let part2_symbol = if day.info.correct2 { "✅" } else { "❌" };

                println!(
                    "║ {:>2} ║ {:max_name_len$} ║ {:max_part1_len$} ║ {:max_part1_time_len$} ║ {} ║ \
                     {:max_part2_len$} ║ {:max_part2_time_len$} ║ {} ║ ",
                    day.day,
                    day.info.title,
                    day.info.ans1.unwrap_or("NA".to_string()),
                    format_duration(day.time.0),
                    part1_symbol,
                    day.info.ans2.unwrap_or("NA".to_string()),
                    day.time.1.map(format_duration).unwrap_or("NA".to_string()),
                    part2_symbol,
                );
            }
            Err(e) => {
                let available_space = max_total_len - day_header_len - 2;
                let mut s = e.r#type.to_string();
                s.truncate(available_space);
                println!(
                    "║ {:>2} ║ {:max_name_len$} ║ {:available_space$} ║",
                    e.day, e.title, s
                );
            }
        }
    }
    println!(
        "╚{}╩{}╩{}╩{}╩{}╩{}╩{}╩{}╝",
        "═".repeat(4),
        "═".repeat(max_name_len + 2),
        "═".repeat(max_part1_len + 2),
        "═".repeat(max_part1_time_len + 2),
        "═".repeat(4),
        "═".repeat(max_part2_len + 2),
        "═".repeat(max_part2_time_len + 2),
        "═".repeat(4),
    );*/

    Table::default()
        .set_header(format!("🦀 Advent of Code {year} 🦀"))
        .add_row(vec!["Day", "Part 1", "Part 2"])
        .display();
}

pub async fn tally(matches: &ArgMatches) -> Result<(), AocError> {
    let number_of_runs = get_number_of_runs(matches)?;

    let root_folder = get_root_path()?;
    let year = root_folder
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let possible_days = get_possible_days(year)?;
    let days = compile_and_verify_days(possible_days, root_folder.clone(), year).await?;
    let mut days = run_days(days, root_folder, number_of_runs)?;
    let mut dont_have = Vec::new();

    days.retain(|elem| {
        if matches!(elem, Err(e) if e.r#type == ErrorTypes::NotImplementd) {
            dont_have.push(elem.as_ref().unwrap_err().day);
            false
        } else {
            true
        }
    });

    let have = days
        .iter()
        .flatten()
        .map(|br| (br.day, (br.time.0, br.time.1)))
        .collect();

    print_table(days, year);
    print_info(have, dont_have, number_of_runs);

    Ok(())
}

#[derive(Default)]
struct Table {
    header: String,
    rows: Vec<Vec<String>>,
}

impl Table {
    fn set_header(mut self, header: String) -> Self {
        self.header = header;
        self
    }
    fn add_row<S: AsRef<str>>(mut self, row: Vec<S>) -> Self {
        self.rows
            .push(row.into_iter().map(|s| s.as_ref().to_owned()).collect());
        self
    }
}

impl Table {
    const SPACES_LEN: usize = 2;
    const BOARDER_LEN: usize = 2;
    fn max_width(&self) -> usize {
        let max = self.header.len().max(
            self.rows
                .iter()
                .flatten()
                .map(|s| s.len())
                .max()
                .unwrap_or(0),
        );

        max
    }

    fn display_bottom_connect_with_row(&self, idx: usize) {
        let row = &self.rows[idx];

        let mut vec = row
            .iter()
            .map(|s| s.len() + Self::SPACES_LEN)
            .collect::<Vec<_>>();

        let total = vec.iter().sum::<usize>();

        // Pad out the rest (if any), starting from the back
        let remaining = self.table_width() - Self::BOARDER_LEN - total;
        for (_, i) in (0..remaining).zip((0..vec.len()).rev().cycle()) {
            vec[i] += 1;
        }

        let line = vec
            .into_iter()
            .map(|len| "═".repeat(len))
            .collect::<Vec<_>>()
            .join("╦");

        println!("╠{line}╣",);
    }

    fn display_row(&self, idx: usize) {
        let row = &self.rows[idx];
        let mut vec = row
            .iter()
            .map(|s| s.len() + Self::SPACES_LEN)
            .collect::<Vec<_>>();

        let total = vec.iter().sum::<usize>();

        // Pad out the rest (if any), starting from the back
        let remaining = self.table_width() - Self::BOARDER_LEN - total;
        for (_, i) in (0..remaining).zip((0..vec.len()).rev().cycle()) {
            vec[i] += 1;
        }

        let line = vec
            .into_iter()
            .zip(row.iter())
            .map(|(len, s)| format!("{:^len$}", s))
            .collect::<Vec<_>>()
            .join("║");

        println!("║{line}║",);
    }

    fn table_width(&self) -> usize {
        self.max_width() + Self::SPACES_LEN + Self::BOARDER_LEN
    }

    fn display_top(&self) {
        let s = "═".repeat(self.table_width());
        println!("╔{s}╗");
    }

    fn display_header(&self) {
        let max = self.table_width() - Self::SPACES_LEN;
        println!("║{:^max$}║", self.header);
    }

    fn display(&self) {
        self.display_top();
        self.display_header();
        self.display_bottom_connect_with_row(0);
        self.display_row(0);
    }
}
