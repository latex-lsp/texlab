use crate::client::LspClient;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures::executor::block_on;
use futures_boxed::boxed;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use serde_repr::*;
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

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildOptions {
    pub executable: Option<String>,
    pub args: Option<Vec<String>>,
    pub on_save: Option<bool>,
}

impl BuildOptions {
    pub fn executable(&self) -> String {
        self.executable
            .as_ref()
            .map(Clone::clone)
            .unwrap_or_else(|| "latexmk".to_owned())
    }

    pub fn args(&self) -> Vec<String> {
        self.args.as_ref().map(Clone::clone).unwrap_or_else(|| {
            vec![
                "-pdf".to_owned(),
                "-interaction=nonstopmode".to_owned(),
                "-synctex=1".to_owned(),
            ]
        })
    }

    pub fn on_save(&self) -> bool {
        self.on_save.unwrap_or(true)
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
        let mut args = Vec::new();
        args.append(&mut self.options.args());
        args.push(path.file_name().unwrap().to_string_lossy().into_owned());

        let mut process = Command::new(self.options.executable())
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
                        message: line.into(),
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
            .workspace()
            .find_parent(&request.document().uri)
            .unwrap();
        let path = document.uri.to_file_path().unwrap();

        let progress_id = "build";
        let title = path.file_name().unwrap().to_string_lossy().into_owned();
        let params = ProgressStartParams {
            id: progress_id.into(),
            title: title.into(),
            cancellable: Some(false),
            message: Some("Building".into()),
            percentage: None,
        };
        self.client.progress_start(params).await;

        let status = match self.build(&path).await {
            Ok(true) => BuildStatus::Success,
            Ok(false) => BuildStatus::Error,
            Err(_) => BuildStatus::Failure,
        };

        let params = ProgressDoneParams {
            id: progress_id.into(),
        };
        self.client.progress_done(params).await;
        BuildResult { status }
    }
}
