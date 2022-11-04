use super::lib::get_headers;

use crate::ast::Comparison;

pub fn run(
    mut records: impl Iterator<Item = csv::StringRecord>,
    column: String,
    comparison: Comparison,
) -> Result<Vec<csv::StringRecord>, String> {
    let (raw_headers, headers) = get_headers(&mut records)?;

    let column_index = *headers
        .get(column.as_str())
        .ok_or(format!("Invalid column '{column}'"))?;

    Ok(std::iter::once(raw_headers)
        .chain(records.filter(move |record| {
            match apply(column_index, comparison.clone(), record) {
                Ok(rejected) => rejected,
                Err(err) => {
                    eprintln!("{err}");
                    false
                }
            }
        }))
        .collect())
}

/// Returns `true` if the record was rejected by the condition, false otherwise.
fn apply(
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
