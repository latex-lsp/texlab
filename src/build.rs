use crate::client::LspClient;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures::executor::block_on;
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
use std::thread;

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
    C: LspClient + Send + Sync + 'static,
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
            .stderr(Stdio::piped())
            .current_dir(path.parent().unwrap())
            .spawn()?;

        Self::read(Arc::clone(&self.client), process.stdout.take().unwrap());
        Self::read(Arc::clone(&self.client), process.stderr.take().unwrap());
        Ok(process.wait()?.success())
    }

    fn read<R>(client: Arc<C>, output: R)
    where
        R: io::Read + Send + 'static,
    {
        thread::spawn(move || {
            let client = Arc::clone(&client);
            let reader = io::BufReader::new(output);
            reader.lines().for_each(|line| {
                if let Ok(line) = line {
                    let params = LogMessageParams {
                        typ: MessageType::Log,
                        message: Cow::from(line),
                    };
                    block_on(client.log_message(params));
                }
            });
        });
    }
}

impl<C> FeatureProvider for BuildProvider<C>
where
    C: LspClient + Send + Sync + 'static,
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
