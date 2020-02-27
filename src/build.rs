use futures::future::{AbortHandle, Abortable, Aborted};
use futures::lock::Mutex;
use futures::prelude::*;
use futures::stream;
use futures_boxed::boxed;
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use texlab_protocol::*;
use texlab_workspace::*;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BuildProvider<C> {
    pub client: Arc<C>,
    pub options: LatexOptions,
    pub token: ProgressToken,
}

impl<C> BuildProvider<C>
where
    C: LspClient + Send + Sync + 'static,
{
    pub fn new(client: Arc<C>, options: LatexOptions) -> Self {
        Self {
            client,
            options,
            token: ProgressToken::String(format!("texlab-build-{}", Uuid::new_v4())),
        }
    }

    async fn build<'a>(&'a self, path: &'a Path) -> io::Result<bool> {
        let build_options = self
            .options
            .build
            .as_ref()
            .map(Clone::clone)
            .unwrap_or_default();

        let root_directory = self.options.root_directory();
        let build_dir = root_directory
            .as_ref()
            .map(AsRef::as_ref)
            .or_else(|| path.parent())
            .unwrap();

        let mut args = Vec::new();
        args.append(&mut build_options.args());
        args.push(path.to_string_lossy().into_owned());

        let mut process = Command::new(build_options.executable())
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(build_dir)
            .spawn()?;

        let stdout = BufReader::new(process.stdout.take().unwrap()).lines();
        let stderr = BufReader::new(process.stderr.take().unwrap()).lines();
        let mut output = stream::select(stdout, stderr);

        while let Some(Ok(line)) = output.next().await {
            let params = LogMessageParams {
                typ: MessageType::Log,
                message: line,
            };

            self.client.log_message(params).await;
        }

        Ok(process.await?.success())
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
            .find_parent(&request.document().uri, &request.options)
            .or_else(|| request.workspace().find(&request.document().uri))
            .unwrap();

        match document.uri.to_file_path() {
            Ok(path) => {
                if request.client_capabilities.has_work_done_progress_support() {
                    let params = WorkDoneProgressCreateParams {
                        token: self.token.clone(),
                    };
                    self.client.work_done_progress_create(params).await.unwrap();

                    let title = path.file_name().unwrap().to_string_lossy().into_owned();
                    let params = ProgressParams {
                        token: self.token.clone(),
                        value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(
                            WorkDoneProgressBegin {
                                title,
                                cancellable: Some(true),
                                message: Some("Building".into()),
                                percentage: None,
                            },
                        )),
                    };
                    self.client.progress(params).await;
                }

                let status = match self.build(&path).await {
                    Ok(true) => BuildStatus::Success,
                    Ok(false) => BuildStatus::Error,
                    Err(_) => BuildStatus::Failure,
                };

                BuildResult { status }
            }
            Err(()) => BuildResult {
                status: BuildStatus::Failure,
            },
        }
    }
}

pub struct BuildManager<C> {
    handles_by_token: Mutex<HashMap<ProgressToken, AbortHandle>>,
    client: Arc<C>,
}

impl<C> BuildManager<C>
where
    C: LspClient + Send + Sync + 'static,
{
    pub fn new(client: Arc<C>) -> Self {
        Self {
            handles_by_token: Mutex::new(HashMap::new()),
            client,
        }
    }

    pub async fn build(
        &self,
        request: FeatureRequest<BuildParams>,
        options: LatexOptions,
    ) -> BuildResult {
        let provider = BuildProvider::new(Arc::clone(&self.client), options);
        let (handle, reg) = AbortHandle::new_pair();
        {
            let mut handles_by_token = self.handles_by_token.lock().await;
            handles_by_token.insert(provider.token.clone(), handle);
        }

        let result = match Abortable::new(provider.execute(&request), reg).await {
            Ok(result) => result,
            Err(Aborted) => BuildResult {
                status: BuildStatus::Cancelled,
            },
        };

        if request.client_capabilities.has_work_done_progress_support() {
            let params = ProgressParams {
                token: provider.token.clone(),
                value: ProgressParamsValue::WorkDone(WorkDoneProgress::Done(
                    WorkDoneProgressDone { message: None },
                )),
            };
            self.client.progress(params).await;
        }

        {
            let mut handles_by_token = self.handles_by_token.lock().await;
            handles_by_token.remove(&provider.token);
        }

        result
    }

    pub async fn cancel(&self, token: ProgressToken) {
        let handles_by_token = self.handles_by_token.lock().await;
        if let Some(handle) = handles_by_token.get(&token) {
            handle.abort();
        } else if let ProgressToken::String(id) = token {
            if id == "texlab-build-*" {
                handles_by_token.values().for_each(|handle| handle.abort());
            }
        }
    }
}
