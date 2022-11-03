use crate::ast::Comparison;

use std::collections::HashMap;

pub fn run(
    records: Box<dyn Iterator<Item = csv::StringRecord>>,
    column: String,
    headers: &HashMap<String, usize>,
    comparison: Comparison,
) -> Result<Box<dyn Iterator<Item = csv::StringRecord>>, String> {
    let column_index = *headers
        .get(column.as_str())
        .ok_or(format!("Invalid column '{column}'"))?;

    Ok(Box::new(records.filter(move |record| {
        match apply(column_index, comparison.clone(), record) {
            Ok(rejected) => rejected,
            Err(err) => {
                eprintln!("{err}");
                false
            }
        }
    })))
}

/// Returns `true` if the record was rejected by the condition, false otherwise.
pub fn apply(
    column: usize,
    comparison: Comparison,
    record: &csv::StringRecord,
) -> Result<bool, String> {
    let value = record
        .get(column)
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
