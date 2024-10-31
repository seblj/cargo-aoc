use crate::{
    error::AocError,
    util::{get_day_title_and_answers, parse_get_answers, Task},
};

fn assert_print_equal(expected: &str, actual: &str, task: Task) {
    if expected == actual {
        println!("Task {}: \x1b[0;32mok\x1b[0m", task);
    } else {
        println!(
            "Task {}: \x1b[0;31mFAILED\x1b[0m
    expected: `{}`,
    actual: `{}`",
            task, expected, actual
        )
    }
}

fn assert_print_fail(s: &str, task: Task) {
    println!(
        "Task {}: \x1b[0;31mFAILED\x1b[0m
    `{}`",
        task, s
    )
}

pub async fn assert_answer(out: &str, day: u32, year: i32) -> Result<(), AocError> {
    let info = get_day_title_and_answers(day, year as u32).await?;
    let (p1, p2) = parse_get_answers(out);

    match (p1, p2, info.part1_answer, info.part2_answer) {
        (Some(p1), Some(p2), Some(a1), Some(a2)) => {
            assert_print_equal(&a1, &p1, Task::One);
            assert_print_equal(&a2, &p2, Task::Two);
        }
        (Some(p1), None, Some(a1), Some(a2)) => {
            assert_print_equal(&a1, &p1, Task::One);
            assert_print_fail(
                &format!("Couldn't verify answer against the correct one: {}", a2),
                Task::Two,
            );
        }
        (None, Some(p2), Some(a1), Some(a2)) => {
            assert_print_fail(
                &format!("Couldn't verify answer against the correct one: {}", a1),
                Task::One,
            );
            assert_print_equal(&a2, &p2, Task::Two);
        }
        (Some(p1), _, Some(a1), None) if day == 25 => {
            assert_print_equal(&a1, &p1, Task::One);
        }
        (Some(p1), _, Some(a1), None) => {
            assert_print_equal(&a1, &p1, Task::One);
            assert_print_fail("Have you completed it?", Task::Two);
        }
        (None, Some(_), Some(a1), None) => {
            assert_print_fail(
                &format!("Couldn't verify answer against the correct one: {}", a1),
                Task::One,
            );
            assert_print_fail("Coulnd't find the submitted answer", Task::Two);
        }
        (None, None, _, _) => {
            assert_print_fail("Have you completed it?", Task::One);
            assert_print_fail("Have you completed it?", Task::Two);
        }
        // Assumes that it is impossible to get answer for part 2 if we don't get answer for part 1
        (_, _, None, _) => {
            assert_print_fail("Coulnd't find the submitted answer", Task::One);
            assert_print_fail("Coulnd't find the submitted answer", Task::Two);
        }
    }

    Ok(())
}
