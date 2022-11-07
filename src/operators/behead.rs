pub fn run(
    mut records: impl Iterator<Item = csv::StringRecord>,
) -> Result<Vec<csv::StringRecord>, String> {
    records.next();
    Ok(records.collect())
}
