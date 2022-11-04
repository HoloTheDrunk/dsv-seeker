use std::collections::HashMap;

pub fn get_headers(
    records: &mut impl Iterator<Item = csv::StringRecord>,
) -> Result<(csv::StringRecord, HashMap<String, usize>), String> {
    records
        .next()
        .map(|record| {
            (
                record.clone(),
                record
                    .iter()
                    .enumerate()
                    .map(|(k, v)| (v.to_owned(), k))
                    .collect::<HashMap<String, usize>>(),
            )
        })
        .ok_or_else(|| "Empty stream".to_string())
}
