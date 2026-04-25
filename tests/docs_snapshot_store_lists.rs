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

#[test]
fn readme_scope_section_references_snapshot_store_summary() {
    let readme = include_str!("../README.md");

    assert!(readme.contains(
        "- 기본 in-memory snapshot store와 상단 `SNAPSHOT_STORE` 항목에 나열된 모든 로컬/embedded backend, S3-compatible object storage, external managed snapshot store 지원"
    ));
    assert!(readme.contains(
        "그 외 상단 `SNAPSHOT_STORE` 항목의 embedded/local durability backend는 같은 `SnapshotStore` 경계를 통해 로컬 durable restart 복구를 제공한다."
    ));
    assert!(readme.contains(
        "반면 상단 `SNAPSHOT_STORE` 항목에서 `memory`를 제외한 durability backend와 managed-managed owner handoff rehearsal은 이제 회귀 테스트로 검증됐다."
    ));
    assert!(readme.contains(
        "다음 기준은 상단 `SNAPSHOT_STORE` 항목에서 `memory`, `sqlite`, `s3`, `managed`를 제외한 embedded/local durability backend 중 어떤 것을 운영 기본값으로 둘지 고를 때 사용한다."
    ));
}

#[test]
fn architecture_references_supported_snapshot_store_constant() {
    let architecture = include_str!("../docs/architecture.md");

    assert!(architecture.contains(
        "`src/storage/mod.rs`의 `SUPPORTED_SNAPSHOT_STORES` canonical 목록에 대응하는 adapter"
    ));
    assert!(architecture.contains(
        "`Config.snapshot_store`가 `src/storage/mod.rs`의 `SUPPORTED_SNAPSHOT_STORES` canonical 목록 전체에 대한 어댑터 선택을 담당한다."
    ));
    assert!(architecture.contains(
        "`src/storage/mod.rs`의 `SUPPORTED_SNAPSHOT_STORES` canonical 목록에 대응하는 snapshot adapter, shared SQLite lease 기반 owner coordination, 그리고 external managed lease coordination이 있으므로"
    ));
}
