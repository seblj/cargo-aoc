use std::path::{Path, PathBuf};

use clap::ArgMatches;
use tokio::fs;

use crate::{
    error::AocError,
    util::{file::*, get_day},
};

async fn create_file(path: &Path) -> Result<(), AocError> {
    let folder = path.join(".bench");

    let file = fs::read_to_string(path.join("src").join("main.rs")).await?;
    let file = file.replace("fn main()", "fn not_main()");

    let tests = r#"
use criterion::{black_box, criterion_group, criterion_main, Criterion};
fn from_elem(c: &mut Criterion)
{
    let input = read_input("XXX");

    c.bench_function("task_one", |b| b.iter(|| task_one(black_box(&input))));
    c.bench_function("task_two", |b| b.iter(|| task_two(black_box(&input))));
}

criterion_group!(benches, from_elem);
criterion_main!(benches);
"#;

    let remove_errors = "#![allow(dead_code)]";

    let input = path.join("input");
    let tests = tests.replace("XXX", &input.display().to_string());
    let tests = tests.replace("read_input(\"", "read_input(r\"");

    let file = format!("{}\n{}\n{}", remove_errors, file, tests);

    fs::write(folder.join("benches").join("my_benchmark.rs"), file).await?;

    Ok(())
}

async fn create_bench_foler(path: &Path) -> Result<(), AocError> {
    let folder = path.join(".bench");
    fs::create_dir(&folder).await?;

    let template = format!(
        "{}/template/Cargo.toml.benchmark",
        env!("CARGO_MANIFEST_DIR")
    );

    fs::copy(template, folder.join("Cargo.toml")).await?;
    fs::create_dir(folder.join("benches")).await?;

    Ok(())
}

// Wanted to use tokio here, but got some issues related to recursive async
// function. https://rust-lang.github.io/async-book/07_workarounds/04_recursion.html
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), AocError> {
    use std::fs;
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub async fn bench(matches: &ArgMatches) -> Result<(), AocError> {
    let day = get_day(matches)?;
    let root_folder = get_root_path()?;
    let day_path = day_path(root_folder, day).await?;

    if !day_path.join(".bench").exists() {
        create_bench_foler(&day_path).await?;
    }
    create_file(&day_path).await?;

    tokio::process::Command::new("cargo")
        .arg("bench")
        .current_dir(day_path.join(".bench"))
        .spawn()?
        .wait()
        .await?;

    if let Some(output) = matches.get_one::<String>("output").map(PathBuf::from) {
        if !output.is_dir() {
            return Err(AocError::ArgError("Path must be a folder".into()));
        }
        println!("Copying into {}", output.display());
        copy_dir_all(day_path.join(".bench/target/criterion"), output)?;
    }

    Ok(())
}
