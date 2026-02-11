use std::collections::HashSet;

/// Verify all embedded pattern files parse correctly and produce unique rule IDs.
#[test]
fn test_all_patterns_parse_and_unique_ids() {
    let pattern_files: Vec<(&str, &str)> = vec![
        ("hidden", include_str!("../patterns/hidden.toml")),
        ("secrets", include_str!("../patterns/secrets.toml")),
        ("network", include_str!("../patterns/network.toml")),
        ("filesystem", include_str!("../patterns/filesystem.toml")),
        ("execution", include_str!("../patterns/execution.toml")),
        ("injection", include_str!("../patterns/injection.toml")),
        ("social", include_str!("../patterns/social.toml")),
        ("metadata", include_str!("../patterns/metadata.toml")),
    ];

    let mut all_ids = HashSet::new();

    for (name, content) in &pattern_files {
        let file: toml::Value =
            toml::from_str(content).unwrap_or_else(|e| panic!("Failed to parse {name}.toml: {e}"));

        let rules = file["rules"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}.toml missing [[rules]] array"));

        for rule in rules {
            let id = rule["id"]
                .as_str()
                .unwrap_or_else(|| panic!("{name}.toml: rule missing id"));
            let pattern = rule["pattern"]
                .as_str()
                .unwrap_or_else(|| panic!("{name}.toml: rule {id} missing pattern"));

            // Verify regex compiles
            let multiline = rule
                .get("multiline")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if multiline {
                regex::RegexBuilder::new(pattern)
                    .multi_line(true)
                    .dot_matches_new_line(true)
                    .build()
                    .unwrap_or_else(|e| panic!("{name}.toml: rule {id} regex error: {e}"));
            } else {
                regex::Regex::new(pattern)
                    .unwrap_or_else(|e| panic!("{name}.toml: rule {id} regex error: {e}"));
            }

            // Verify unique ID
            assert!(
                all_ids.insert(id.to_string()),
                "Duplicate rule ID: {id} in {name}.toml"
            );
        }
    }

    // Verify we loaded a reasonable number of rules
    assert!(
        all_ids.len() >= 50,
        "Expected at least 50 rules, found {}",
        all_ids.len()
    );
}
