use clap::ArgMatches;

use crate::{error::AocError, util::get_day};

pub async fn clippy(matches: &ArgMatches) -> Result<(), AocError> {
    let day = get_day(matches)?;

    // Creating a vec like this is a bit weird? Seems like there should be
    // a better solution for this.
    let mut args = vec!["clippy", "--color", "always"];
    if matches.get_flag("fix") {
        // fix complains about unstaged files without the last two flags
        args.extend(["--fix", "--allow-dirty", "--allow-staged"]);
    }
    let day = format!("day_{:02}", day);
    args.extend(["--bin", &day]);

    let res = tokio::process::Command::new("cargo")
        .args(args)
        .output()
        .await?;

    let err = std::str::from_utf8(&res.stderr)?.trim_end();
    println!("{}", err);

    let out = std::str::from_utf8(&res.stdout)?.trim_end();
    println!("{}", out);

    Ok(())
}
