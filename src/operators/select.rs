use crate::ast::*;

use std::collections::HashMap;

pub fn apply(
    columns: &Column,
    headers: &HashMap<String, usize>,
    record: csv::StringRecord,
) -> Result<csv::StringRecord, String> {
    if let Column::Names(names) = columns {
        let mut to_keep = Vec::new();

        for name in names {
            to_keep.push(
                headers
                    .get(name.as_str())
                    .ok_or(format!("Invalid column '{name}'"))?,
            );
        }

        Ok(csv::StringRecord::from(
            to_keep
                .iter()
                .map(|&&index| record.get(index).ok_or(format!("Missing column {index}")))
                .collect::<Result<Vec<&str>, String>>()?,
        ))
    } else {
        Ok(record)
    }
}
