use crate::ast::*;

use std::collections::HashMap;

pub fn run(
    records: Box<dyn Iterator<Item = csv::StringRecord>>,
    columns: &Column,
    headers: &HashMap<String, usize>,
) -> Result<Box<dyn Iterator<Item = csv::StringRecord>>, String> {
    let column_indices = if let Column::Names(names) = columns {
        names
            .iter()
            .map(|name| {
                headers
                    .get(name.as_str())
                    .copied()
                    .ok_or(format!("Invalid column '{name}'"))
            })
            .collect::<Result<Vec<usize>, String>>()?
    } else {
        (0..headers.len()).collect::<Vec<usize>>()
    };

    Ok(Box::new(records.filter_map(move |record| {
        apply(column_indices.clone(), record).ok()
    })))
}

pub fn apply(columns: Vec<usize>, record: csv::StringRecord) -> Result<csv::StringRecord, String> {
    Ok(csv::StringRecord::from(
        columns
            .iter()
            .map(|&index| record.get(index).ok_or(format!("Missing column {index}")))
            .collect::<Result<Vec<&str>, String>>()?,
    ))
}
