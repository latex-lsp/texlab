use crate::client::LspClient;
use crate::workspace::*;
use futures::channel::*;
use futures::compat::*;
use futures::future::{AbortHandle, Abortable, Aborted};
use futures::prelude::*;
use futures::stream;
use futures_boxed::boxed;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::collections::HashMap;
use std::io::{self, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::io::lines;
use tokio_process::CommandExt;
use uuid::Uuid;

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
        self.on_save.unwrap_or(false)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum BuildStatus {
    Success = 0,
    Error = 1,
    Failure = 2,
    Cancelled = 3,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildResult {
    pub status: BuildStatus,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BuildProvider<C> {
    pub client: Arc<C>,
    pub options: BuildOptions,
    pub progress_id: String,
}

impl<C> BuildProvider<C>
where
    C: LspClient + Send + Sync + 'static,
{
    pub fn new(client: Arc<C>, options: BuildOptions) -> Self {
        Self {
            client,
            options,
            progress_id: format!("texlab-build-{}", Uuid::new_v4()),
        }
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
            .spawn_async()?;

        let stdout = lines(BufReader::new(process.stdout().take().unwrap())).compat();
        let stderr = lines(BufReader::new(process.stderr().take().unwrap())).compat();
        let mut output = stream::select(stdout, stderr);

        while let Some(Ok(line)) = output.next().await {
            let params = LogMessageParams {
                typ: MessageType::Log,
                message: line.into(),
            };

            self.client.log_message(params).await;
        }

        let status = process.compat().await?;
        Ok(status.success())
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

        let title = path.file_name().unwrap().to_string_lossy().into_owned();
        let params = ProgressStartParams {
            id: self.progress_id.clone().into(),
            title: title.into(),
            cancellable: Some(true),
            message: Some("Building".into()),
            percentage: None,
        };
        self.client.progress_start(params).await;

        let status = match self.build(&path).await {
            Ok(true) => BuildStatus::Success,
            Ok(false) => BuildStatus::Error,
            Err(_) => BuildStatus::Failure,
        };

        BuildResult { status }
    }
}

pub enum BuildEngineMessage {
    Build(
        FeatureRequest<BuildParams>,
        BuildOptions,
        oneshot::Sender<BuildResult>,
    ),
    Cancel(String),
}

pub struct BuildEngine<C> {
    pub message_tx: mpsc::Sender<BuildEngineMessage>,
    message_rx: mpsc::Receiver<BuildEngineMessage>,
    client: Arc<C>,
}

impl<C> BuildEngine<C>
where
    C: LspClient + Send + Sync + 'static,
{
    pub fn new(client: Arc<C>) -> Self {
        let (message_tx, message_rx) = mpsc::channel(0);
        Self {
            message_tx,
            message_rx,
            client,
        }
    }

    pub async fn listen(&mut self) {
        let mut handles_by_id = HashMap::new();
        while let Some(message) = self.message_rx.next().await {
            match message {
                BuildEngineMessage::Build(request, options, result_tx) => {
                    let provider = BuildProvider::new(Arc::clone(&self.client), options);
                    let (handle, reg) = AbortHandle::new_pair();
                    handles_by_id.insert(provider.progress_id.clone(), handle);

                    let result = match Abortable::new(provider.execute(&request), reg).await {
                        Ok(result) => result,
                        Err(Aborted) => BuildResult {
                            status: BuildStatus::Cancelled,
                        },
                    };

                    let params = ProgressDoneParams {
                        id: provider.progress_id.clone().into(),
                    };

                    self.client.progress_done(params).await;
                    handles_by_id.remove(&provider.progress_id);
                    result_tx.send(result).unwrap();
                }
                BuildEngineMessage::Cancel(id) => {
                    if id == "texlab-build-*" {
                        handles_by_id
                            .iter()
                            .filter(|(id, _)| id.starts_with("texlab-build-"))
                            .for_each(|(_, handle)| handle.abort());
                    } else {
                        if let Some(handle) = handles_by_id.get(&id) {
                            handle.abort();
                        }
                    }
                }
            }
        }
    }
}
