use insta::assert_debug_snapshot;

use crate::Options;

#[test]
fn issue_707() {
    assert_debug_snapshot!(
        serde_json::from_value::<Option<Options>>(serde_json::json!({}))
            .unwrap()
            .unwrap_or_default()
    );
}
