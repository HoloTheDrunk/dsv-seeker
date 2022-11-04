use super::lib::get_headers;

use crate::ast::SortDirection;

pub fn run(
    mut records: impl Iterator<Item = csv::StringRecord>,
    column: String,
    is_numerical: bool,
    direction: SortDirection,
) -> Result<Vec<csv::StringRecord>, String> {
    let (raw_headers, headers) = get_headers(&mut records)?;

    let column_index = *headers
        .get(column.as_str())
        .ok_or(format!("Invalid column '{column}'"))?;

    let mut records: Vec<csv::StringRecord> = records.collect();

    if is_numerical {
        let get_parsed = |record: &csv::StringRecord| {
            record
                .get(column_index)
                .unwrap()
                .parse::<isize>()
                .expect("Field is not numerical")
        };

        records.sort_by(|a, b| get_parsed(&a).cmp(&get_parsed(&b)))
    } else {
        records.sort_by(|a, b| {
            a.get(column_index)
                .unwrap()
                .cmp(&b.get(column_index).unwrap())
        });
    }

    Ok(match direction {
        SortDirection::Ascending => std::iter::once(raw_headers)
            .chain(records.into_iter())
            .collect(),
        SortDirection::Descending => std::iter::once(raw_headers)
            .chain(records.into_iter().rev())
            .collect(),
    })
}
