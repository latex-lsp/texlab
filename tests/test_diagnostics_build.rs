pub mod support;

use std::sync::Arc;
use support::*;
use texlab_distro::UnknownDistribution;
use tokio::fs;

#[tokio::test]
async fn did_change_update() {
    let scenario = Scenario::new("diagnostics/build", Arc::new(Box::new(UnknownDistribution)));
    scenario.open("foo.tex").await;
    {
        let diagnostics_by_uri = scenario.client.diagnostics_by_uri.lock().await;
        let diagnostics = &diagnostics_by_uri[&scenario.uri("foo.tex")];
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "Undefined control sequence.");
    }
    let log_path = scenario.uri("foo.log").to_file_path().unwrap();
    fs::write(log_path, "").await.unwrap();
    scenario.server.execute(|_| ()).await;
    {
        let diagnostics_by_uri = scenario.client.diagnostics_by_uri.lock().await;
        let diagnostics = &diagnostics_by_uri[&scenario.uri("foo.tex")];
        assert!(diagnostics.is_empty());
    }
}
