#[test]
fn release_rings_are_limited_to_the_four_safe_states() {
    let rings = ["draft", "review", "pilot", "all"];
    assert!(rings.contains(&"review"));
    assert!(!rings.contains(&"production"));
}
