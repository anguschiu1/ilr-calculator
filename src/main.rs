use chrono::{Duration, NaiveDate};
use std::cmp::{max, min};
use std::io::{self, Write};

/// Prompts the user for a date and parses it.
///
/// This function will repeatedly ask the user for input until a valid date
/// in the "YYYY-MM-DD" format is entered. If the user enters an empty line,
/// it returns `None`, which is used as a signal to stop input.
///
/// # Arguments
///
/// * `prompt` - The message to display to the user.
///
/// # Returns
///
/// An `Option<NaiveDate>` which is `Some(date)` on a valid parse, or `None`
/// if the user provides empty input.
fn get_date_from_user(prompt: &str) -> Option<NaiveDate> {
    loop {
        print!("{}", prompt);
        // We need to flush stdout to ensure the prompt is displayed before `read_line`.
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let trimmed_input = input.trim();

        if trimmed_input.is_empty() {
            return None;
        }

        match NaiveDate::parse_from_str(trimmed_input, "%Y-%m-%d") {
            Ok(date) => return Some(date),
            Err(_) => {
                println!("Invalid date format. Please use YYYY-MM-DD and try again.");
            }
        }
    }
}

fn main() {
    println!("--- Absence Calculator for ILR ---");
    println!("Please enter all dates in YYYY-MM-DD format.");

    // // 1. Get the ILR start date from the user.
    // let ilr_start_date = match get_date_from_user("Enter the ILR start date: ") {
    //     Some(date) => date,
    //     None => {
    //         println!("ILR start date is required. Exiting.");
    //         return;
    //     }
    // };

    // 2. Get multiple absence date ranges from the user.
    let mut absence_periods = Vec::new();
    println!(
        "\nEnter absence periods. To finish, press Enter on an empty line for the start date."
    );

    let mut counter = 1;
    loop {
        println!("\n--- Absence Period #{} ---", counter);
        let absence_start = match get_date_from_user("Enter absence start date: ") {
            Some(date) => date,
            None => break, // User is done entering dates.
        };

        let absence_end = loop {
            if let Some(date) = get_date_from_user("Enter absence end date:   ") {
                if date >= absence_start {
                    break date;
                } else {
                    println!("End date must be on or after the start date. Please try again.");
                }
            } else {
                println!("Absence end date is required for a period. Please try again.");
            }
        };

        absence_periods.push((absence_start, absence_end));
        counter += 1;
    }

    // 3 & 4. Calculate and print the total number of absence days within the 365 days prior to each absence end date.
    println!("\n--- Absence Calculation Results (365-day rolling window) ---");

    for (absence_start, absence_end) in absence_periods.iter() {
        let calculation_end = *absence_end;
        let calculation_start = calculation_end - Duration::days(365);

        let total_absence_days: i64 = absence_periods
            .iter()
            .map(|(other_start, other_end)| {
                let overlap_start = max(*other_start, calculation_start);
                let overlap_end = min(*other_end, calculation_end);

                if overlap_start <= overlap_end {
                    (overlap_end - overlap_start).num_days() + 1
                } else {
                    0
                }
            })
            .sum();

        println!(
            "\nAbsence Period: {} to {}",
            absence_start.format("%Y-%m-%d"),
            absence_end.format("%Y-%m-%d")
        );
        println!(
            "  365-day Calculation Window: {} to {}",
            calculation_start.format("%Y-%m-%d"),
            calculation_end.format("%Y-%m-%d")
        );
        println!("  Total Absence Days within Window: {}", total_absence_days);
    }
}
