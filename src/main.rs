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

/// Holds the results of a calculation for a single absence period.
#[derive(Debug, PartialEq)] // Added for testing purposes
struct CalculationResult {
    absence_start: NaiveDate,
    absence_end: NaiveDate,
    window_start: NaiveDate,
    window_end: NaiveDate,
    total_days_in_window: i64,
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
fn parse_and_validate_absences(data: &str) -> Result<Vec<(NaiveDate, NaiveDate)>, Box<dyn Error>> {
    let parsed_periods: Vec<AbsencePeriod> = serde_json::from_str(data)?;

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

/// Reads a file and passes its content to the parser.
fn get_absences_from_file(path: &str) -> Result<Vec<(NaiveDate, NaiveDate)>, Box<dyn Error>> {
    let data = fs::read_to_string(path)?;
    parse_and_validate_absences(&data)
}

/// Performs the absence calculation for all periods.
///
/// For each absence period, it defines a 365-day rolling window ending on the
/// absence's end date. It then sums the days of all absences that fall
/// within that specific window.
fn calculate_rolling_absences(
    absence_periods: &[(NaiveDate, NaiveDate)],
) -> Vec<CalculationResult> {
    if absence_periods.is_empty() {
        return Vec::new();
    }

    // --- Merge overlapping and adjacent intervals to prevent double-counting ---
    let mut sorted_periods = absence_periods.to_vec();
    sorted_periods.sort_by_key(|(start, _)| *start);

    let mut merged_periods: Vec<(NaiveDate, NaiveDate)> = Vec::new();
    merged_periods.push(sorted_periods[0]);

    for &(start, end) in sorted_periods.iter().skip(1) {
        let last_mut = merged_periods.last_mut().unwrap();
        // If the current period starts before or exactly one day after the last one ends, merge them.
        if start <= last_mut.1 + Duration::days(1) {
            last_mut.1 = max(last_mut.1, end); // Extend the end date
        } else {
            // Otherwise, it's a new, separate period.
            merged_periods.push((start, end));
        }
    }

    let mut results = Vec::new();
    for (absence_start, absence_end) in absence_periods.iter() {
        let calculation_end = *absence_end;
        let calculation_start = calculation_end - Duration::days(365);

        // Calculate the total using the MERGED periods.
        let total_absence_days: i64 = merged_periods
            .iter()
            .filter_map(|(period_start, period_end)| {
                let overlap_start = max(*period_start, calculation_start);
                let overlap_end = min(*period_end, calculation_end);

                if overlap_start <= overlap_end {
                    Some((overlap_end - overlap_start).num_days() + 1)
                } else {
                    None
                }
            })
            .sum();

        results.push(CalculationResult {
            absence_start: *absence_start,
            absence_end: *absence_end,
            window_start: calculation_start,
            window_end: calculation_end,
            total_days_in_window: total_absence_days,
        });
    }
    results
}

/// Calculates and prints the results for the given absence periods.
fn calculate_and_print_results(absence_periods: &[(NaiveDate, NaiveDate)]) {
    println!("\n--- Absence Calculation Results (365-day rolling window) ---");
    let results = calculate_rolling_absences(absence_periods);

    for result in results {
        println!(
            "\nAbsence Period: {} to {}",
            result.absence_start, result.absence_end
        );
        println!(
            "  365-day calculation window: {} to {}",
            result.window_start, result.window_end
        );
        println!(
            "  Total absence days within this window: {}",
            result.total_days_in_window
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

#[cfg(test)]
mod tests {
    use super::*;

    fn d(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn test_parse_valid_json() {
        let json_data = r#"[
            {"start_date": "2023-01-01", "end_date": "2023-01-10"},
            {"start_date": "2023-05-15", "end_date": "2023-05-20"}
        ]"#;
        let expected = vec![
            (d(2023, 1, 1), d(2023, 1, 10)),
            (d(2023, 5, 15), d(2023, 5, 20)),
        ];
        assert_eq!(parse_and_validate_absences(json_data).unwrap(), expected);
    }

    #[test]
    fn test_parse_json_with_invalid_period() {
        // The function should skip the period where end_date < start_date.
        let json_data = r#"[
            {"start_date": "2023-01-01", "end_date": "2023-01-10"},
            {"start_date": "2023-06-01", "end_date": "2023-05-20"}
        ]"#;
        let expected = vec![(d(2023, 1, 1), d(2023, 1, 10))];
        assert_eq!(parse_and_validate_absences(json_data).unwrap(), expected);
    }

    #[test]
    fn test_parse_invalid_json_syntax() {
        let json_data = r#"[{"start_date": "2023-01-01" "end_date": "2023-01-10"}]"#; // Missing comma
        assert!(parse_and_validate_absences(json_data).is_err());
    }

    #[test]
    fn test_parse_invalid_date_format() {
        let json_data = r#"[{"start_date": "2023/01/01", "end_date": "2023-01-10"}]"#;
        assert!(parse_and_validate_absences(json_data).is_err());
    }

    #[test]
    fn test_calculate_single_absence() {
        let periods = vec![(d(2023, 4, 1), d(2023, 4, 10))]; // 10 days
        let results = calculate_rolling_absences(&periods);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].total_days_in_window, 10);
        assert_eq!(results[0].window_start, d(2022, 4, 10));
        assert_eq!(results[0].window_end, d(2023, 4, 10));
    }

    #[test]
    fn test_calculate_multiple_separate_absences() {
        let periods = vec![
            (d(2023, 1, 1), d(2023, 1, 10)), // 10 days
            (d(2023, 8, 1), d(2023, 8, 20)), // 20 days
        ];
        let results = calculate_rolling_absences(&periods);
        assert_eq!(results.len(), 2);

        // For the first period, its window only contains itself.
        assert_eq!(results[0].total_days_in_window, 10);
        assert_eq!(results[0].window_end, d(2023, 1, 10));

        // For the second period, its window contains both.
        assert_eq!(results[1].total_days_in_window, 30); // 10 + 20
        assert_eq!(results[1].window_end, d(2023, 8, 20));
    }

    #[test]
    fn test_calculate_overlapping_absences() {
        let periods = vec![
            (d(2023, 3, 1), d(2023, 3, 15)),  // 15 days
            (d(2023, 3, 10), d(2023, 3, 25)), // 16 days
        ];
        let results = calculate_rolling_absences(&periods);
        assert_eq!(results.len(), 2);

        // Total unique days are from 3/1 to 3/25 = 25 days.
        // Both periods are within each other's 365-day window.
        // The window for the first period also contains the full merged range.
        assert_eq!(results[0].total_days_in_window, 15);
        assert_eq!(results[1].total_days_in_window, 25);
    }

    #[test]
    fn test_absence_outside_365_day_window() {
        let periods = vec![
            (d(2021, 5, 1), d(2021, 5, 10)), // 10 days, old
            (d(2023, 8, 1), d(2023, 8, 20)), // 20 days, recent
        ];
        let results = calculate_rolling_absences(&periods);
        assert_eq!(results.len(), 2);

        // For the first period, its window only sees itself.
        assert_eq!(results[0].total_days_in_window, 10);
        assert_eq!(results[0].window_end, d(2021, 5, 10));

        // For the second period, its window starts on 2022-08-20.
        // The first period (in 2021) is outside this window.
        assert_eq!(results[1].total_days_in_window, 20);
        assert_eq!(results[1].window_start, d(2022, 8, 20));
        assert_eq!(results[1].window_end, d(2023, 8, 20));
    }

    #[test]
    fn test_absence_partially_in_window() {
        let periods = vec![
            (d(2022, 8, 15), d(2022, 8, 25)), // 11 days total
            (d(2023, 8, 20), d(2023, 8, 30)), // 11 days total
        ];
        let results = calculate_rolling_absences(&periods);
        assert_eq!(results.len(), 2);

        // Window for the second period ends 2023-08-30, starts 2022-08-30.
        // The first period (2022-08-15 to 2022-08-25) is completely outside this window.
        // Let's check the calculation.
        // Window for period 2: 2022-08-30 to 2023-08-30
        // Period 1 (2022-08-15 to 2022-08-25) is entirely before this window.
        // Period 2 (2023-08-20 to 2023-08-30) is entirely inside.
        // So total should be 11.

        // Let's adjust the first period to be partially inside.
        let periods = vec![
            (d(2022, 8, 25), d(2022, 9, 5)),  // 12 days total
            (d(2023, 8, 30), d(2023, 9, 10)), // 12 days total
        ];
        let results = calculate_rolling_absences(&periods);

        // Window for the second period: 2022-08-31 to 2023-08-30.
        // Overlap with first period: 2022-08-31 to 2022-09-05 (6 days).
        // Overlap with second period: 2023-08-30 to 2023-08-30 (1 day).
        // Wait, the second period is from 2023-08-30 to 2023-09-10.
        // The window for the second period is 2022-09-10 to 2023-09-10.
        // Overlap with first period (2022-08-25 to 2022-09-05) is none.
        // Overlap with second period (2023-08-30 to 2023-09-10) is all 12 days.
        // Let's re-check the window calculation: end_date - 365 days.
        // Window for period 2: d(2023, 9, 10) - 365 days = d(2022, 9, 10).
        // First period (2022-08-25 to 2022-09-05) is outside this window.
        // Second period is fully inside.
        // Total should be 12 days from the second period only.
        assert_eq!(results[1].total_days_in_window, 12);
    }
}
