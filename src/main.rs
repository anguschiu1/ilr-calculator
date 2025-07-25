use chrono::{Duration, NaiveDate};
use serde::Deserialize;
use std::cmp::{max, min};
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};

/// Represents a single absence period from the JSON input.
#[derive(Deserialize)]
struct AbsencePeriod {
    start_date: NaiveDate,
    end_date: NaiveDate,
}

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

/// Gathers absence periods by prompting the user interactively.
fn get_absences_from_interactive() -> Vec<(NaiveDate, NaiveDate)> {
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
    absence_periods
}

/// Reads absence periods from a JSON file.
/// The JSON file should be an array of objects, each with "start_date" and "end_date".
/// e.g., `[{"start_date": "YYYY-MM-DD", "end_date": "YYYY-MM-DD"}]`
fn get_absences_from_file(path: &str) -> Result<Vec<(NaiveDate, NaiveDate)>, Box<dyn Error>> {
    let data = fs::read_to_string(path)?;
    let parsed_periods: Vec<AbsencePeriod> = serde_json::from_str(&data)?;

    // Validate dates and convert to the tuple format used by the rest of the program.
    let mut absence_periods = Vec::new();
    for period in parsed_periods {
        if period.end_date < period.start_date {
            // Using eprintln! to write to standard error for error messages.
            eprintln!(
                "Warning: Invalid period in JSON file. End date {} is before start date {}. Skipping.",
                period.end_date, period.start_date
            );
            continue;
        }
        absence_periods.push((period.start_date, period.end_date));
    }
    Ok(absence_periods)
}

/// Calculates and prints the results for the given absence periods.
fn calculate_and_print_results(absence_periods: &[(NaiveDate, NaiveDate)]) {
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
            "  Number of Days in absence: {} days",
            (*absence_end - *absence_start).num_days() + 1
        );
        println!(
            "  365-day calculation window: {} to {}",
            calculation_start.format("%Y-%m-%d"),
            calculation_end.format("%Y-%m-%d")
        );
        println!(
            "  Total absence days within this window: {}",
            total_absence_days
        );
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let absence_periods = if args.len() > 1 {
        // File input mode
        let file_path = &args[1];
        println!("--- Reading absences from {} ---", file_path);
        match get_absences_from_file(file_path) {
            Ok(periods) => periods,
            Err(e) => {
                eprintln!(
                    "Error: Failed to process file '{}'. Reason: {}",
                    file_path, e
                );
                return;
            }
        }
    } else {
        // Interactive mode
        println!("--- Absence Calculator (Interactive Mode) ---");
        println!("Usage: Pass a JSON file path as an argument, or enter dates interactively.");
        println!("Please enter all dates in YYYY-MM-DD format.");
        get_absences_from_interactive()
    };

    if absence_periods.is_empty() {
        println!("\nNo absence periods to process. Exiting.");
        return;
    }

    calculate_and_print_results(&absence_periods);
}
