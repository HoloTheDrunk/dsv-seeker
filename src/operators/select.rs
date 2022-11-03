use crate::ast::*;

use std::collections::HashMap;

pub fn run(
    mut records: impl Iterator<Item = csv::StringRecord>,
    columns: &Column,
) -> Result<Vec<csv::StringRecord>, String> {
    let raw_headers = records.next().ok_or_else(|| "Empty stream".to_string())?;
    dbg!(&raw_headers);
    let headers = raw_headers
        .iter()
        .enumerate()
        .map(|(k, v)| (v.to_owned(), k))
        .collect::<HashMap<String, usize>>();

    if let Column::Names(names) = columns {
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
    } else {
        Ok(std::iter::once(raw_headers).chain(records).collect())
    }
}

fn apply(columns: Vec<usize>, record: csv::StringRecord) -> Result<csv::StringRecord, String> {
    Ok(csv::StringRecord::from(
        columns
            .iter()
            .map(|&index| record.get(index).ok_or(format!("Missing column {index}")))
            .collect::<Result<Vec<&str>, String>>()?,
    ))
}
