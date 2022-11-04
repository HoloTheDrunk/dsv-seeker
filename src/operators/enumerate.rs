use std::collections::HashMap;

use super::lib::get_headers;

pub fn run(
    mut records: impl Iterator<Item = csv::StringRecord>,
    column: String,
) -> Result<Vec<csv::StringRecord>, String> {
    let (_raw, headers) = get_headers(&mut records)?;

    let column_index = *headers
        .get(column.as_str())
        .ok_or(format!("Invalid column '{column}'"))?;

    Ok(
        std::iter::once(csv::StringRecord::from(vec!["count", column.as_ref()]))
            .chain(
                records
                    .filter_map(|record| match apply(column_index, record) {
                        Ok(value) => Some(value),
                        Err(err) => {
                            eprintln!("{err}");
                            None
                        }
                    })
                    .fold(HashMap::<String, usize>::new(), |mut acc, cur| {
                        acc.entry(cur).and_modify(|v| *v += 1).or_insert(1);
                        acc
                    })
                    .into_iter()
                    .map(|(k, v)| csv::StringRecord::from(vec![v.to_string(), k])),
            )
            .collect(),
    )
}

fn apply(column: usize, record: csv::StringRecord) -> Result<String, String> {
    record
        .get(column)
        .map(String::from)
        .ok_or(format!("Invalid column index {column}"))
}
