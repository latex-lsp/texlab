use texlab_protocol::{BuildResult, BuildStatus};
use texlab_test::build::*;

#[tokio::test]
async fn success_single_file() {
    if let Some(result) = run_command("pdflatex", "success_single_file.tex").await {
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
    if let Some(result) = run_command("pdflatex", "success_multiple_files_main.tex").await {
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
    if let Some(scenario) = run_on_save("pdflatex", "success_on_save.tex").await {
        scenario.read("success_on_save.pdf").await;
    }
}

#[tokio::test]
async fn error_single_file() {
    if let Some(result) = run_command("pdflatex", "error_single_file.tex").await {
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
    if let Some(result) = run_command("pdflatex", "error_multiple_files_main.tex").await {
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
    if let Some(scenario) = run_on_save("pdflatex", "error_on_save.tex").await {
        scenario.read("error_on_save.log").await;
    }
}

#[tokio::test]
async fn failure_single_file() {
    let executable = "2ae97e68b8074dca880f9c17ebafaa38";
    if let Some(result) = run_command(executable, "failure_single_file.tex").await {
        assert_eq!(
            result,
            BuildResult {
                status: BuildStatus::Failure
            }
        );
    }
}
