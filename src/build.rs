use crate::client::LspClient;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::borrow::Cow;
use std::io;
use std::io::BufRead;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildOptions {
    pub executable: String,
    pub args: Vec<String>,
    pub on_save: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            executable: "latexmk".to_owned(),
            args: vec![
                "-pdf".to_owned(),
                "-interaction=nonstopmode".to_owned(),
                "-synctex=1".to_owned(),
            ],
            on_save: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum BuildStatus {
    Success = 0,
    Error = 1,
    Failure = 2,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildResult {
    pub status: BuildStatus,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BuildProvider<C> {
    client: Arc<C>,
    options: BuildOptions,
}

impl<C> BuildProvider<C>
where
    C: LspClient + Send + Sync,
{
    pub fn new(client: Arc<C>, options: BuildOptions) -> Self {
        Self { client, options }
    }

    async fn build<'a>(&'a self, path: &'a Path) -> io::Result<bool> {
        let mut options = self.options.clone();
        let mut args = Vec::new();
        args.append(&mut options.args);
        args.push(path.to_string_lossy().into_owned());

        let mut process = Command::new(options.executable)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .current_dir(path.parent().unwrap())
            .spawn()?;

        let stdout = process.stdout.as_mut().unwrap();
        let mut reader = io::BufReader::new(stdout);
        loop {
            let mut line = String::new();
            let count = reader.read_line(&mut line)?;
            if count == 0 {
                break;
            }
            let params = LogMessageParams {
                typ: MessageType::Log,
                message: Cow::from(line.trim().to_owned()),
            };
            self.client.log_message(params).await;
        }

        Ok(process.wait()?.success())
    }
}

impl<C> FeatureProvider for BuildProvider<C>
where
    C: LspClient + Send + Sync,
{
    type Params = BuildParams;
    type Output = BuildResult;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<BuildParams>) -> BuildResult {
        let document = request
            .workspace
            .find_parent(&request.document.uri)
            .unwrap();
        let path = document.uri.to_file_path().unwrap();

        let title = path.file_name().unwrap().to_string_lossy().into_owned();
        let mut progress_params = ProgressParams {
            id: Cow::from("build"),
            title: Cow::from(title),
            message: Some(Cow::from("Building...")),
            percentage: None,
            done: None,
        };
        self.client.progress(progress_params.clone()).await;

        let status = match self.build(&path).await {
            Ok(true) => BuildStatus::Success,
            Ok(false) => BuildStatus::Error,
            Err(_) => BuildStatus::Failure,
        };

        progress_params.done = Some(true);
        self.client.progress(progress_params).await;
        BuildResult { status }
    }
}
