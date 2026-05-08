use std::collections::BTreeSet;

fn config_env_keys() -> BTreeSet<String> {
    let config_source = include_str!("../src/config.rs");
    let from_env_source = config_source
        .split("pub fn from_env()")
        .nth(1)
        .and_then(|tail| tail.split("fn env_string(").next())
        .expect("Config::from_env source should be present");

    let mut keys = BTreeSet::new();
    let mut search_start = 0;

    while let Some(relative_idx) = from_env_source[search_start..].find("env_") {
        let env_call_idx = search_start + relative_idx;
        let after_call = &from_env_source[env_call_idx..];

        let Some(first_quote_relative_idx) = after_call.find('"') else {
            break;
        };
        let value_start = env_call_idx + first_quote_relative_idx + 1;
        let Some(second_quote_relative_idx) = from_env_source[value_start..].find('"') else {
            break;
        };
        let value_end = value_start + second_quote_relative_idx;
        let candidate = &from_env_source[value_start..value_end];

        if candidate
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        {
            keys.insert(candidate.to_owned());
        }

        search_start = value_end + 1;
    }

    keys
}

fn env_example_keys() -> BTreeSet<String> {
    include_str!("../.env.example")
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }

            let uncommented = trimmed.strip_prefix('#').unwrap_or(trimmed).trim();
            let (key, _) = uncommented.split_once('=')?;
            Some(key.trim().to_owned())
        })
        .collect()
}

#[test]
fn env_example_covers_all_config_keys() {
    let config_keys = config_env_keys();
    let example_keys = env_example_keys();

    let missing: Vec<_> = config_keys.difference(&example_keys).cloned().collect();

    assert!(
        missing.is_empty(),
        ".env.example is missing config keys: {missing:?}"
    );
}
