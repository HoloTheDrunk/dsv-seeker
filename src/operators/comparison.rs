use crate::ast::Comparison;

use std::collections::HashMap;

/// Returns `true` if the record was rejected by the condition, false otherwise.
pub fn apply(
    column: &String,
    comparison: &Comparison,
    headers: &HashMap<String, usize>,
    record: &csv::StringRecord,
) -> Result<bool, String> {
    let value = record
        .get(
            *headers
                .get(column.as_str())
                .ok_or(format!("Invalid column '{column}'"))?,
        )
        .ok_or(format!("Missing column {column}"))?;

    match comparison {
        Comparison::Equals(other) => {
            if value != other {
                return Ok(true);
            }
        }
        Comparison::Matches(pattern) => {
            if !pattern.is_match(value) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
