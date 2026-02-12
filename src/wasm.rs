//! WebAssembly bindings for the ILR calculator.
//!
//! This module provides JavaScript-compatible functions for calculating
//! rolling absence periods. All functions accept and return JSON strings
//! for easy integration with JavaScript/TypeScript.

use crate::{calculate_from_json, AbsencePeriod};
use wasm_bindgen::prelude::*;

/// Calculates rolling absences from a JSON string input.
///
/// # Arguments
///
/// * `json_input` - JSON string containing an array of absence periods
///   Format: `[{"start_date": "YYYY-MM-DD", "end_date": "YYYY-MM-DD"}]`
///
/// # Returns
///
/// A JSON string containing the calculation results, or an error message.
///
/// # Example
///
/// ```javascript
/// const input = '[{"start_date": "2023-01-01", "end_date": "2023-01-10"}]';
/// const result = calculate_rolling_absences(input);
/// ```
#[wasm_bindgen]
pub fn calculate_rolling_absences(json_input: &str) -> Result<String, JsValue> {
    let results =
        calculate_from_json(json_input).map_err(|e| JsValue::from_str(&format!("Error: {}", e)))?;

    serde_json::to_string(&results)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Validates absence periods from a JSON string without performing calculations.
///
/// # Arguments
///
/// * `json_input` - JSON string containing an array of absence periods
///
/// # Returns
///
/// A JSON string containing the validated periods, or an error message.
#[wasm_bindgen]
pub fn validate_absence_periods(json_input: &str) -> Result<String, JsValue> {
    let periods: Vec<AbsencePeriod> = serde_json::from_str(json_input)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    crate::validate_absence_periods(&periods)
        .map_err(|e| JsValue::from_str(&format!("Validation error: {}", e)))?;

    serde_json::to_string(&periods)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_calculate_rolling_absences_valid() {
        let json = r#"[{"start_date": "2023-01-01", "end_date": "2023-01-10"}]"#;
        let result = calculate_rolling_absences(json).unwrap();
        assert!(result.contains("total_days_in_window"));
        assert!(result.contains("10"));
    }

    #[wasm_bindgen_test]
    fn test_calculate_rolling_absences_invalid_json() {
        let result = calculate_rolling_absences("not json");
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_validate_absence_periods_valid() {
        let json = r#"[{"start_date": "2023-01-01", "end_date": "2023-01-10"}]"#;
        let result = validate_absence_periods(json).unwrap();
        assert!(result.contains("start_date"));
    }

    #[wasm_bindgen_test]
    fn test_validate_absence_periods_invalid_period() {
        let json = r#"[{"start_date": "2023-06-01", "end_date": "2023-05-20"}]"#;
        let result = validate_absence_periods(json);
        assert!(result.is_err());
    }
}
