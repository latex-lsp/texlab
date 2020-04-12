use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{
        BuildParams, BuildResult, BuildStatus, ClientCapabilitiesExt, LatexOptions, LspClient,
        ProgressToken,
    },
};
use futures::{
    future::{AbortHandle, Abortable, Aborted},
    lock::Mutex,
    prelude::*,
    stream,
};
use futures_boxed::boxed;
use lsp_types::{
    LogMessageParams, MessageType, ProgressParams, ProgressParamsValue, WorkDoneProgress,
    WorkDoneProgressBegin, WorkDoneProgressCreateParams, WorkDoneProgressEnd,
};
use std::{collections::HashMap, io, path::Path, process::Stdio, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use uuid::Uuid;

pub struct BuildProvider<C> {
    client: Arc<C>,
    handles_by_token: Mutex<HashMap<ProgressToken, AbortHandle>>,
}

impl<C> BuildProvider<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            handles_by_token: Mutex::new(HashMap::new()),
        }
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

impl<C> FeatureProvider for BuildProvider<C>
where
    C: LspClient + Send + Sync + 'static,
{
    type Params = BuildParams;
    type Output = BuildResult;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<BuildParams>) -> BuildResult {
        let token = ProgressToken::String(format!("texlab-build-{}", Uuid::new_v4()));
        let (handle, reg) = AbortHandle::new_pair();
        {
            let mut handles_by_token = self.handles_by_token.lock().await;
            handles_by_token.insert(token.clone(), handle);
        }

        let doc = req
            .snapshot()
            .parent(&req.current().uri, &req.options, &req.current_dir)
            .unwrap_or_else(|| Arc::clone(&req.view.current));

        if !doc.is_file() {
            return BuildResult {
                status: BuildStatus::Failure,
            };
        }

        let status = match doc.uri.to_file_path() {
            Ok(path) => {
                if req.client_capabilities.has_work_done_progress_support() {
                    let params = WorkDoneProgressCreateParams {
                        token: token.clone(),
                    };
                    self.client.work_done_progress_create(params).await.unwrap();

                    let title = path.file_name().unwrap().to_string_lossy().into_owned();
                    let params = ProgressParams {
                        token: token.clone(),
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

                let latex_options = req.options.latex.clone().unwrap_or_default();
                let client = Arc::clone(&self.client);
                match Abortable::new(build(&path, &latex_options, client), reg).await {
                    Ok(Ok(true)) => BuildStatus::Success,
                    Ok(Ok(false)) => BuildStatus::Error,
                    Ok(Err(_)) => BuildStatus::Failure,
                    Err(Aborted) => BuildStatus::Cancelled,
                }
            }
            Err(()) => BuildStatus::Failure,
        };

        if req.client_capabilities.has_work_done_progress_support() {
            let params = ProgressParams {
                token: token.clone(),
                value: ProgressParamsValue::WorkDone(WorkDoneProgress::End(WorkDoneProgressEnd {
                    message: None,
                })),
            };
            self.client.progress(params).await;
        }
        {
            let mut handles_by_token = self.handles_by_token.lock().await;
            handles_by_token.remove(&token);
        }

        BuildResult { status }
    }
}

async fn build<C>(path: &Path, options: &LatexOptions, client: Arc<C>) -> io::Result<bool>
where
    C: LspClient + Send + Sync + 'static,
{
    let build_options = options.build.as_ref().cloned().unwrap_or_default();
    let build_dir = options
        .root_directory
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
        .kill_on_drop(true)
        .spawn()?;

    let stdout = BufReader::new(process.stdout.take().unwrap()).lines();
    let stderr = BufReader::new(process.stderr.take().unwrap()).lines();
    let mut output = stream::select(stdout, stderr);

    tokio::spawn(async move {
        while let Some(Ok(line)) = output.next().await {
            let params = LogMessageParams {
                typ: MessageType::Log,
                message: line,
            };

            client.log_message(params).await;
        }
    });

    Ok(process.await?.success())
}
