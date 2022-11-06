use super::lib::get_headers;

use crate::ast::Column;

pub fn run(
    mut records: impl Iterator<Item = csv::StringRecord>,
    columns: &Column,
) -> Result<Vec<csv::StringRecord>, String> {
    let (raw_headers, headers) = get_headers(&mut records)?;

    let records: Vec<csv::StringRecord> = match columns {
        Column::All => records
            .map(|record| {
                csv::StringRecord::from(
                    record
                        .iter()
                        .map(|field| field.trim())
                        .collect::<Vec<&str>>(),
                )
            })
            .collect(),
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

            records
                .map(|record| {
                    csv::StringRecord::from(
                        record
                            .iter()
                            .enumerate()
                            .map(|(index, field)| {
                                if indices.contains(&index) {
                                    field.trim()
                                } else {
                                    field
                                }
                            })
                            .collect::<Vec<&str>>(),
                    )
                })
                .collect()
        }
    };

    Ok(std::iter::once(raw_headers)
        .chain(records.into_iter())
        .collect())
}
