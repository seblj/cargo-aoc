use crate::{
    error::AocError,
    util::{get_day_title_and_answers, parse_get_answers, Task},
};

pub fn assert_print(expected: &str, actual: &str, task: Task)
{
    if expected == actual
    {
        println!("Task {}: \x1b[0;32mok\x1b[0m", task);
    }
    else
    {
        println!(
            "Task {}: \x1b[0;31mFAILED\x1b[0m
    expected: `{}`,
    actual: `{}`",
            task, expected, actual
        )
    }
}

pub async fn assert_answer(out: &str, day: u32, year: i32) -> Result<(), AocError>
{
    let info = get_day_title_and_answers(day, year as u32).await?;
    let (p1, p2) = parse_get_answers(out);

    match (p1, p2, info.part1_answer, info.part2_answer)
    {
        (Some(p1), Some(p2), Some(a1), Some(a2)) =>
        {
            assert_print(&a1, &p1, Task::One);
            assert_print(&a2, &p2, Task::Two);
        },
        (Some(p1), None, Some(a1), Some(_)) =>
        {
            assert_print(&a1, &p1, Task::One);
            println!("Couldn't verify answer for part 2");
        },
        (Some(p1), _, Some(a1), None) =>
        {
            assert_print(&a1, &p1, Task::One);
            println!("You haven't completed part 2");
        },
        (None, Some(p2), Some(_), Some(a2)) =>
        {
            println!("Couldn't verify answer for part 1");
            assert_print(&a2, &p2, Task::Two);
        },
        (None, None, ..) => println!("You haven't completed the day yet..."),

        // Assumes that it is impossible for it to _not_ find the submitted answer for part 1, but
        // then find the submitted answer for part 2
        (_, _, None, _) => println!("Coulnd't find the submitted answers..."),

        (None, Some(_), Some(_), None) => println!(
            "Only found answer for part 2, but could only find submitted answer for part 1"
        ),
    }

    Ok(())
}
