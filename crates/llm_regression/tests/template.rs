use llm_regression::{sample_combat_roll, DEFAULT_SEED};

#[test]
fn regression_template() {
    let trace = sample_combat_roll(DEFAULT_SEED);
    insta::assert_json_snapshot!("combat_round", trace);
}
