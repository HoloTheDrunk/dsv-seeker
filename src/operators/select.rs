use super::lib::get_headers;

use crate::ast::*;

pub fn run(
    mut records: impl Iterator<Item = csv::StringRecord>,
    columns: &Column,
) -> Result<Vec<csv::StringRecord>, String> {
    let (raw_headers, headers) = get_headers(&mut records)?;

    match columns {
        Column::All => Ok(std::iter::once(raw_headers).chain(records).collect()),
        Column::Names(names) => {
            let indices = names
                .iter()
                .map(|name| {
                    headers
                        .get(name.as_str())
                        .copied()
                        .ok_or(format!("Invalid column '{name}'"))
                })
                .collect::<Result<Vec<usize>, String>>()?;

            Ok(std::iter::once(csv::StringRecord::from(names.clone()))
                .chain(records.filter_map(move |record| apply(indices.clone(), record).ok()))
                .collect())
        }
    }
}

/// Returns the desired columns from the `StringRecord` in a new `StringRecord`.
fn apply(columns: Vec<usize>, record: csv::StringRecord) -> Result<csv::StringRecord, String> {
    Ok(csv::StringRecord::from(
        columns
            .iter()
            .map(|&index| record.get(index).ok_or(format!("Missing column {index}")))
            .collect::<Result<Vec<&str>, String>>()?,
    ))
}
