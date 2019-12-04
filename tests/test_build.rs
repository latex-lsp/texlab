pub mod support;

use support::build::*;
use texlab::build::{BuildResult, BuildStatus};

#[tokio::test]
async fn success_single_file() {
    if let Some(result) = run("success_single_file", "foo.tex", "pdflatex").await {
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
    if let Some(result) = run("success_multiple_files", "foo.tex", "pdflatex").await {
        assert_eq!(
            result,
            BuildResult {
                status: BuildStatus::Success
            }
        );
    }
}

#[tokio::test]
async fn success_on_save() {
    if let Some(scenario) = run_on_save("success_on_save", "foo.tex", "pdflatex").await {
        scenario.read("foo.pdf").await;
    }
}

#[tokio::test]
async fn error_single_file() {
    if let Some(result) = run("error_single_file", "foo.tex", "pdflatex").await {
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
    if let Some(result) = run("error_multiple_files", "foo.tex", "pdflatex").await {
        assert_eq!(
            result,
            BuildResult {
                status: BuildStatus::Error
            }
        );
    }
}

#[tokio::test]
async fn error_on_save() {
    if let Some(scenario) = run_on_save("error_on_save", "foo.tex", "pdflatex").await {
        scenario.read("foo.log").await;
    }
}

#[tokio::test]
async fn failure_single_file() {
    if let Some(result) = run(
        "error_multiple_files",
        "foo.tex",
        "2ae97e68b8074dca880f9c17ebafaa38",
    )
    .await
    {
        assert_eq!(
            result,
            BuildResult {
                status: BuildStatus::Failure
            }
        );
    }
}
