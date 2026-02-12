use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
#[allow(unused_imports)]
pub use wasm::*;

/// Represents a single absence period from JSON input or API.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct AbsencePeriod {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

/// Holds the results of a calculation for a single absence period.
#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct CalculationResult {
    pub absence_start: NaiveDate,
    pub absence_end: NaiveDate,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub total_days_in_window: i64,
}

/// Validates absence periods and converts them to a consistent format.
///
/// # Arguments
///
/// * `periods` - A slice of absence periods to validate
///
/// # Returns
///
/// A `Result` containing a vector of validated periods, or an error message.
pub fn validate_absence_periods(
    periods: &[AbsencePeriod],
) -> Result<Vec<(NaiveDate, NaiveDate)>, String> {
    let mut validated_periods = Vec::new();
    for period in periods {
        if period.end_date < period.start_date {
            return Err(format!(
                "Invalid period: end date {} is before start date {}",
                period.end_date, period.start_date
            ));
        }
        validated_periods.push((period.start_date, period.end_date));
    }
    Ok(validated_periods)
}

/// Merges overlapping and adjacent absence periods to prevent double-counting.
///
/// Two periods are merged if they overlap or are adjacent (within 1 day of each other).
///
/// # Arguments
///
/// * `periods` - A slice of absence periods as tuples (start_date, end_date)
///
/// # Returns
///
/// A vector of merged periods.
pub fn merge_absence_periods(periods: &[(NaiveDate, NaiveDate)]) -> Vec<(NaiveDate, NaiveDate)> {
    if periods.is_empty() {
        return Vec::new();
    }

    let mut sorted_periods = periods.to_vec();
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

    merged_periods
}

/// Calculates the overlap between a period and a window, returning the number of days.
///
/// # Arguments
///
/// * `period_start` - Start date of the absence period
/// * `period_end` - End date of the absence period
/// * `window_start` - Start date of the calculation window
/// * `window_end` - End date of the calculation window
///
/// # Returns
///
/// The number of days of overlap (inclusive), or 0 if there's no overlap.
pub fn calculate_overlap_days(
    period_start: NaiveDate,
    period_end: NaiveDate,
    window_start: NaiveDate,
    window_end: NaiveDate,
) -> i64 {
    let overlap_start = max(period_start, window_start);
    let overlap_end = min(period_end, window_end);

    if overlap_start <= overlap_end {
        (overlap_end - overlap_start).num_days() + 1
    } else {
        0
    }
}

/// Performs the absence calculation for all periods.
///
/// For each absence period, it defines a 365-day rolling window ending on the
/// absence's end date. It then sums the days of all merged absences that fall
/// within that specific window.
///
/// # Arguments
///
/// * `absence_periods` - A slice of absence periods as tuples (start_date, end_date)
///
/// # Returns
///
/// A vector of calculation results, one for each input period.
pub fn calculate_rolling_absences(
    absence_periods: &[(NaiveDate, NaiveDate)],
) -> Vec<CalculationResult> {
    if absence_periods.is_empty() {
        return Vec::new();
    }

    // Merge overlapping and adjacent intervals to prevent double-counting
    let merged_periods = merge_absence_periods(absence_periods);

    let mut results = Vec::new();
    for (absence_start, absence_end) in absence_periods.iter() {
        let calculation_end = *absence_end;
        let calculation_start = calculation_end - Duration::days(365);

        // Calculate the total using the MERGED periods.
        let total_absence_days: i64 = merged_periods
            .iter()
            .map(|(period_start, period_end)| {
                calculate_overlap_days(
                    *period_start,
                    *period_end,
                    calculation_start,
                    calculation_end,
                )
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

/// Parses JSON string into absence periods and calculates rolling absences.
///
/// This is a convenience function that combines parsing, validation, and calculation.
///
/// # Arguments
///
/// * `json_input` - JSON string containing an array of absence periods
///
/// # Returns
///
/// A `Result` containing a vector of calculation results, or an error message.
pub fn calculate_from_json(json_input: &str) -> Result<Vec<CalculationResult>, String> {
    let periods: Vec<AbsencePeriod> =
        serde_json::from_str(json_input).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let validated_periods = validate_absence_periods(&periods)?;
    Ok(calculate_rolling_absences(&validated_periods))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn test_validate_absence_periods_valid() {
        let periods = vec![
            AbsencePeriod {
                start_date: d(2023, 1, 1),
                end_date: d(2023, 1, 10),
            },
            AbsencePeriod {
                start_date: d(2023, 5, 15),
                end_date: d(2023, 5, 20),
            },
        ];
        let result = validate_absence_periods(&periods).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], (d(2023, 1, 1), d(2023, 1, 10)));
        assert_eq!(result[1], (d(2023, 5, 15), d(2023, 5, 20)));
    }

    #[test]
    fn test_validate_absence_periods_invalid() {
        let periods = vec![AbsencePeriod {
            start_date: d(2023, 6, 1),
            end_date: d(2023, 5, 20),
        }];
        assert!(validate_absence_periods(&periods).is_err());
    }

    #[test]
    fn test_merge_absence_periods_no_overlap() {
        let periods = vec![
            (d(2023, 1, 1), d(2023, 1, 10)),
            (d(2023, 5, 15), d(2023, 5, 20)),
        ];
        let merged = merge_absence_periods(&periods);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], (d(2023, 1, 1), d(2023, 1, 10)));
        assert_eq!(merged[1], (d(2023, 5, 15), d(2023, 5, 20)));
    }

    #[test]
    fn test_merge_absence_periods_overlapping() {
        let periods = vec![
            (d(2023, 3, 1), d(2023, 3, 15)),
            (d(2023, 3, 10), d(2023, 3, 25)),
        ];
        let merged = merge_absence_periods(&periods);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], (d(2023, 3, 1), d(2023, 3, 25)));
    }

    #[test]
    fn test_merge_absence_periods_adjacent() {
        let periods = vec![
            (d(2023, 3, 1), d(2023, 3, 10)),
            (d(2023, 3, 11), d(2023, 3, 20)), // Adjacent (1 day gap)
        ];
        let merged = merge_absence_periods(&periods);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], (d(2023, 3, 1), d(2023, 3, 20)));
    }

    #[test]
    fn test_merge_absence_periods_exactly_adjacent() {
        let periods = vec![
            (d(2023, 3, 1), d(2023, 3, 10)),
            (d(2023, 3, 10), d(2023, 3, 20)), // Exactly adjacent
        ];
        let merged = merge_absence_periods(&periods);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], (d(2023, 3, 1), d(2023, 3, 20)));
    }

    #[test]
    fn test_merge_absence_periods_empty() {
        let periods: Vec<(NaiveDate, NaiveDate)> = Vec::new();
        let merged = merge_absence_periods(&periods);
        assert_eq!(merged.len(), 0);
    }

    #[test]
    fn test_calculate_overlap_days_full_overlap() {
        let days =
            calculate_overlap_days(d(2023, 1, 5), d(2023, 1, 10), d(2023, 1, 1), d(2023, 1, 15));
        assert_eq!(days, 6); // Jan 5-10 inclusive = 6 days
    }

    #[test]
    fn test_calculate_overlap_days_partial_overlap() {
        let days = calculate_overlap_days(
            d(2023, 1, 5),
            d(2023, 1, 15),
            d(2023, 1, 10),
            d(2023, 1, 20),
        );
        assert_eq!(days, 6); // Jan 10-15 inclusive = 6 days
    }

    #[test]
    fn test_calculate_overlap_days_no_overlap() {
        let days = calculate_overlap_days(
            d(2023, 1, 5),
            d(2023, 1, 10),
            d(2023, 1, 15),
            d(2023, 1, 20),
        );
        assert_eq!(days, 0);
    }

    #[test]
    fn test_calculate_overlap_days_single_day() {
        let days = calculate_overlap_days(
            d(2023, 1, 10),
            d(2023, 1, 10),
            d(2023, 1, 10),
            d(2023, 1, 10),
        );
        assert_eq!(days, 1);
    }

    #[test]
    fn test_calculate_rolling_absences_single_absence() {
        let periods = vec![(d(2023, 4, 1), d(2023, 4, 10))]; // 10 days
        let results = calculate_rolling_absences(&periods);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].total_days_in_window, 10);
        assert_eq!(results[0].window_start, d(2022, 4, 10));
        assert_eq!(results[0].window_end, d(2023, 4, 10));
    }

    #[test]
    fn test_calculate_rolling_absences_multiple_separate() {
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
    fn test_calculate_rolling_absences_overlapping() {
        let periods = vec![
            (d(2023, 3, 1), d(2023, 3, 15)),  // 15 days
            (d(2023, 3, 10), d(2023, 3, 25)), // 16 days
        ];
        let results = calculate_rolling_absences(&periods);
        assert_eq!(results.len(), 2);

        // Merged period: March 1-25 (25 days)
        // Period 1 window: Feb 13, 2023 - March 15, 2023 (merged overlaps March 1-15 = 15 days)
        // Period 2 window: Feb 13, 2023 - March 25, 2023 (merged overlaps March 1-25 = 25 days)
        assert_eq!(results[0].total_days_in_window, 15);
        assert_eq!(results[1].total_days_in_window, 25);
    }

    #[test]
    fn test_calculate_rolling_absences_outside_window() {
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
    fn test_calculate_rolling_absences_partially_in_window() {
        let periods = vec![
            (d(2022, 8, 25), d(2022, 9, 5)),  // 12 days total
            (d(2023, 8, 30), d(2023, 9, 10)), // 12 days total
        ];
        let results = calculate_rolling_absences(&periods);

        // Window for the second period: 2022-09-10 to 2023-09-10.
        // First period (2022-08-25 to 2022-09-05) is outside this window.
        // Second period is fully inside.
        assert_eq!(results[1].total_days_in_window, 12);
    }

    #[test]
    fn test_calculate_rolling_absences_empty() {
        let periods: Vec<(NaiveDate, NaiveDate)> = Vec::new();
        let results = calculate_rolling_absences(&periods);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_calculate_rolling_absences_leap_year() {
        // Test with a period ending on Feb 29 in a leap year
        // 365 days before Feb 29, 2024 = Feb 29, 2023, but 2023 is not a leap year
        // Chrono handles this by normalizing to Feb 28, 2023
        let periods = vec![(d(2024, 2, 1), d(2024, 2, 29))];
        let results = calculate_rolling_absences(&periods);
        assert_eq!(results.len(), 1);
        // The actual window_start will be calculated by chrono, which may normalize
        // Feb 29, 2023 to Feb 28, 2023 or March 1, 2023 depending on implementation
        // We just verify the calculation completes successfully
        assert_eq!(results[0].window_end, d(2024, 2, 29));
        assert_eq!(results[0].total_days_in_window, 29); // Feb 1-29 = 29 days
    }

    #[test]
    fn test_calculate_from_json_valid() {
        let json_data = r#"[
            {"start_date": "2023-01-01", "end_date": "2023-01-10"},
            {"start_date": "2023-05-15", "end_date": "2023-05-20"}
        ]"#;
        let results = calculate_from_json(json_data).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].absence_start, d(2023, 1, 1));
        assert_eq!(results[0].absence_end, d(2023, 1, 10));
    }

    #[test]
    fn test_calculate_from_json_invalid_json() {
        let json_data = r#"[{"start_date": "2023-01-01" "end_date": "2023-01-10"}]"#;
        assert!(calculate_from_json(json_data).is_err());
    }

    #[test]
    fn test_calculate_from_json_invalid_period() {
        let json_data = r#"[
            {"start_date": "2023-06-01", "end_date": "2023-05-20"}
        ]"#;
        assert!(calculate_from_json(json_data).is_err());
    }

    #[test]
    fn test_calculate_rolling_absences_three_periods() {
        let periods = vec![
            (d(2023, 1, 1), d(2023, 1, 5)),   // 5 days
            (d(2023, 1, 10), d(2023, 1, 15)), // 6 days
            (d(2023, 1, 20), d(2023, 1, 25)), // 6 days
        ];
        let results = calculate_rolling_absences(&periods);
        assert_eq!(results.len(), 3);

        // First period window: 2022-01-05 to 2023-01-05 (only contains itself)
        assert_eq!(results[0].total_days_in_window, 5);

        // Second period window: 2022-01-15 to 2023-01-15 (contains first and second)
        assert_eq!(results[1].total_days_in_window, 11); // 5 + 6

        // Third period window: 2022-01-25 to 2023-01-25 (contains all three)
        assert_eq!(results[2].total_days_in_window, 17); // 5 + 6 + 6
    }

    #[test]
    fn test_calculate_rolling_absences_year_boundary() {
        let periods = vec![
            (d(2022, 12, 20), d(2022, 12, 31)), // 12 days
            (d(2023, 1, 1), d(2023, 1, 10)),    // 10 days
        ];
        let results = calculate_rolling_absences(&periods);
        assert_eq!(results.len(), 2);

        // Merged period: Dec 20, 2022 - Jan 10, 2023 (22 days)
        // Period 1 window: Dec 31, 2021 - Dec 31, 2022 (merged overlaps Dec 20-31, 2022 = 12 days)
        // Period 2 window: Jan 10, 2022 - Jan 10, 2023 (merged overlaps Dec 20, 2022 - Jan 10, 2023 = 22 days)
        assert_eq!(results[0].total_days_in_window, 12);
        assert_eq!(results[1].total_days_in_window, 22);
    }
}
