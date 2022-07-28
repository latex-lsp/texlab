use anyhow::Result;
use insta::assert_debug_snapshot;
use texlab::Options;

#[test]
fn issue_707() -> Result<()> {
    assert_debug_snapshot!(
        serde_json::from_value::<Option<Options>>(serde_json::json!({}))?.unwrap_or_default()
    );

    Ok(())
}
