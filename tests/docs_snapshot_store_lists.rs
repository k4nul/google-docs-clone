fn supported_snapshot_store_names() -> Vec<String> {
    let source = include_str!("../src/storage/mod.rs");
    let stores_block = source
        .split("const SUPPORTED_SNAPSHOT_STORES: &[&str] = &[")
        .nth(1)
        .and_then(|tail| tail.split("];").next())
        .expect("SUPPORTED_SNAPSHOT_STORES source should be present");

    stores_block
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim().trim_end_matches(',');
            let value = trimmed.strip_prefix('"')?.strip_suffix('"')?;
            Some(value.to_owned())
        })
        .collect()
}

fn backtick_values(line: &str) -> Vec<String> {
    line.split('`')
        .enumerate()
        .filter_map(|(index, part)| (index % 2 == 1).then_some(part.to_owned()))
        .collect()
}

fn snapshot_store_summary_from_doc(doc: &str, prefix: &str) -> Vec<String> {
    backtick_values(
        doc.lines()
            .find(|line| line.starts_with(prefix))
            .expect("snapshot store summary line should exist"),
    )
    .into_iter()
    .filter(|value| value != "SNAPSHOT_STORE")
    .collect()
}

#[test]
fn readme_snapshot_store_summary_matches_supported_list() {
    let supported = supported_snapshot_store_names();
    let documented =
        snapshot_store_summary_from_doc(include_str!("../README.md"), "- `SNAPSHOT_STORE`:");

    assert_eq!(documented, supported);
}

#[test]
fn setup_snapshot_store_summary_matches_supported_list() {
    let supported = supported_snapshot_store_names();
    let documented =
        snapshot_store_summary_from_doc(include_str!("../docs/setup.md"), "- `SNAPSHOT_STORE`:");

    assert_eq!(documented, supported);
}

#[test]
fn api_snapshot_store_summary_matches_supported_list() {
    let supported = supported_snapshot_store_names();
    let documented =
        snapshot_store_summary_from_doc(include_str!("../docs/api.md"), "- snapshot store는 현재");

    assert_eq!(documented, supported);
}
