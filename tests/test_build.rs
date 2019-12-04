pub mod support;

use support::build::*;
use texlab::build::{BuildResult, BuildStatus};

#[tokio::test]
async fn success_single_file() {
    if let Some(result) = run("success_single_file", "foo.tex").await {
        assert_eq!(
            result,
            BuildResult {
                status: BuildStatus::Success
            }
        );
    }
}

#[tokio::test]
async fn success_multiple_file() {
    if let Some(result) = run("success_multiple_files", "foo.tex").await {
        assert_eq!(
            result,
            BuildResult {
                status: BuildStatus::Success
            }
        );
    }
}

#[tokio::test]
async fn error_single_file() {
    if let Some(result) = run("error_single_file", "foo.tex").await {
        assert_eq!(
            result,
            BuildResult {
                status: BuildStatus::Error
            }
        );
    }
}

#[tokio::test]
async fn error_multiple_files() {
    if let Some(result) = run("error_multiple_files", "foo.tex").await {
        assert_eq!(
            result,
            BuildResult {
                status: BuildStatus::Error
            }
        );
    }
}
