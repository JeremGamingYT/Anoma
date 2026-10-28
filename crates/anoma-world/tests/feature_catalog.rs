use anoma_world::worldgen::{feature_catalog::FeatureCatalog, tags::TagDefinitions};
use std::collections::BTreeMap;
fn map(values: &[(&str, &str)]) -> BTreeMap<String, String> {
    values
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}
#[test]
fn canonical_duplicates_cycles_and_unknown_unselected_children_fail() {
    let tags = TagDefinitions::default();
    assert!(
        FeatureCatalog::from_resources(
            &map(&[
                ("a", r#"{"type":"no_op"}"#),
                ("minecraft:a", r#"{"type":"no_op"}"#)
            ]),
            &BTreeMap::new(),
            &tags
        )
        .is_err()
    );
    let features = map(&[
        (
            "anoma:loop",
            r#"{"type":"sequence","features":["anoma:loop"]}"#,
        ),
        ("anoma:unused", r#"{"type":"not_implemented"}"#),
    ]);
    let placed = map(&[(
        "anoma:loop",
        r#"{"feature":"anoma:loop","placement":[]}"#,
    )]);
    let catalog = FeatureCatalog::from_resources(&features, &placed, &tags).unwrap();
    assert!(catalog.feature(r#""anoma:loop""#).is_err());
    assert!(catalog.placed_feature(r#""anoma:loop""#).is_err());
    assert!(catalog.feature(r#"{"type":"no_op"}"#).is_ok());
    assert!(catalog.feature(r#"{"type":"weighted_random_selector","features":[{"data":"anoma:missing","weight":0}]}"#).is_err());
}
#[test]
fn shared_reference_expansion_is_bounded_before_building_a_large_tree() {
    let leaf = r#"{"feature":{"type":"no_op"},"placement":[]}"#;
    let branch = format!(
        r#"{{"type":"sequence","features":[{}]}}"#,
        vec![r#""anoma:leaf""#; 8192].join(",")
    );
    let features = BTreeMap::from([("anoma:branch".into(), branch)]);
    let placed = map(&[
        ("anoma:leaf", leaf),
        (
            "anoma:branch",
            r#"{"feature":"anoma:branch","placement":[]}"#,
        ),
    ]);
    let catalog =
        FeatureCatalog::from_resources(&features, &placed, &TagDefinitions::default()).unwrap();
    let root = format!(
        r#"{{"type":"sequence","features":[{}]}}"#,
        [r#""anoma:branch""#; 9].join(",")
    );
    assert!(catalog.feature(&root).is_err());
}
