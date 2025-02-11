mod dispatch;
mod extensions;
pub mod options;
mod progress;

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{atomic::AtomicI32, Arc},
    time::Duration,
};

use anyhow::Result;
use base_db::{deps, Owner, Workspace};
use commands::{BuildCommand, CleanCommand, CleanTarget, ForwardSearch};
use crossbeam_channel::{Receiver, Sender};
use distro::{Distro, Language};
use line_index::LineCol;
use lsp_server::{Connection, ErrorCode, Message, RequestId};
use lsp_types::{notification::*, request::*, *};
use notify::event::ModifyKind;
use notify_debouncer_full::{DebouncedEvent, Debouncer, RecommendedCache};
use parking_lot::{Mutex, RwLock};
use rustc_hash::FxHashSet;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Map, Value};
use threadpool::ThreadPool;

use crate::{
    client::LspClient,
    features::{
        completion, definition, folding, formatting, highlight, hover, inlay_hint, link, reference,
        rename, symbols,
    },
    util::{from_proto, line_index_ext::LineIndexExt, normalize_uri, to_proto, ClientFlags},
};

use self::{
    extensions::{
        BuildParams, BuildRequest, BuildResult, BuildStatus, EnvironmentLocation,
        ForwardSearchRequest, ForwardSearchResult, ForwardSearchStatus, TextWithRange,
    },
    options::{Options, StartupOptions},
    progress::ProgressReporter,
};

#[derive(Debug)]
enum InternalMessage {
    SetDistro(Distro),
    SetOptions(Box<Options>),
    FileEvent(Vec<DebouncedEvent>),
    Diagnostics,
    ChktexFinished(Url, Vec<diagnostics::Diagnostic>),
    ForwardSearch(Url, Option<Position>),
    InverseSearch(TextDocumentPositionParams),
}

pub struct Server {
    connection: Arc<Connection>,
    internal_tx: Sender<InternalMessage>,
    internal_rx: Receiver<InternalMessage>,
    workspace: Arc<RwLock<Workspace>>,
    client: LspClient,
    client_flags: Arc<ClientFlags>,
    diagnostic_manager: diagnostics::Manager,
    watcher: FileWatcher,
    pool: ThreadPool,
    pending_builds: Arc<Mutex<FxHashSet<u32>>>,
}

impl Server {
    pub fn exec(connection: Connection) -> Result<()> {
        let client = LspClient::new(connection.sender.clone());
        let (internal_tx, internal_rx) = crossbeam_channel::unbounded();
        let watcher = FileWatcher::new(internal_tx.clone()).expect("init file watcher");

        let mut workspace = Workspace::default();

        let (id, params) = connection.initialize_start()?;
        let params: InitializeParams = serde_json::from_value(params)?;

        let workspace_folders = params
            .workspace_folders
            .unwrap_or_default()
            .into_iter()
            .filter(|folder| folder.uri.scheme() == "file")
            .flat_map(|folder| folder.uri.to_file_path())
            .collect();

        workspace.set_folders(workspace_folders);

        let result = InitializeResult {
            capabilities: Self::capabilities(),
            server_info: Some(ServerInfo {
                name: "TexLab".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
        };

        connection.initialize_finish(id, serde_json::to_value(result)?)?;

        let server = Self {
            connection: Arc::new(connection),
            internal_tx,
            internal_rx,
            workspace: Arc::new(RwLock::new(workspace)),
            client,
            client_flags: Arc::new(from_proto::client_flags(
                params.capabilities,
                params.client_info,
            )),
            diagnostic_manager: diagnostics::Manager::default(),
            watcher,
            pool: threadpool::Builder::new().build(),
            pending_builds: Default::default(),
        };

        let options = serde_json::from_value(params.initialization_options.unwrap_or_default())
            .unwrap_or_default();

        server.run(options)?;
        Ok(())
    }

    fn capabilities() -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::INCREMENTAL),
                    will_save: None,
                    will_save_wait_until: None,
                    save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                        include_text: Some(false),
                    })),
                },
            )),
            document_link_provider: Some(DocumentLinkOptions {
                resolve_provider: Some(false),
                work_done_progress_options: WorkDoneProgressOptions::default(),
            }),
            folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            references_provider: Some(OneOf::Left(true)),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(true),
                trigger_characters: Some(vec![
                    "\\".into(),
                    "{".into(),
                    "}".into(),
                    "@".into(),
                    "/".into(),
                    " ".into(),
                ]),
                ..CompletionOptions::default()
            }),
            document_symbol_provider: Some(OneOf::Left(true)),
            workspace_symbol_provider: Some(OneOf::Left(true)),
            rename_provider: Some(OneOf::Right(RenameOptions {
                prepare_provider: Some(true),
                work_done_progress_options: WorkDoneProgressOptions::default(),
            })),
            document_highlight_provider: Some(OneOf::Left(true)),
            document_formatting_provider: Some(OneOf::Left(true)),
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: vec![
                    "texlab.cleanAuxiliary".into(),
                    "texlab.cleanArtifacts".into(),
                    "texlab.changeEnvironment".into(),
                    "texlab.findEnvironments".into(),
                    "texlab.showDependencyGraph".into(),
                    "texlab.cancelBuild".into(),
                ],
                ..Default::default()
            }),
            inlay_hint_provider: Some(OneOf::Left(true)),
            experimental: Some(Value::Object(Map::from_iter(
                [
                    ("textDocumentBuild".to_string(), Value::Bool(true)),
                    ("textDocumentForwardSearch".to_string(), Value::Bool(true)),
                ]
                .into_iter(),
            ))),
            ..ServerCapabilities::default()
        }
    }

    fn run_query<R, Q>(&self, id: RequestId, query: Q)
    where
        R: Serialize,
        Q: FnOnce(&Workspace) -> R + Send + 'static,
    {
        let client = self.client.clone();
        let workspace = Arc::clone(&self.workspace);
        self.pool.execute(move || {
            let response = lsp_server::Response::new_ok(id, query(&workspace.read()));
            client.send_response(response).unwrap();
        });
    }

    fn run_fallible<R, Q>(&self, id: RequestId, query: Q)
    where
        R: Serialize,
        Q: FnOnce() -> Result<R> + Send + 'static,
    {
        let client = self.client.clone();
        self.pool.execute(move || match query() {
            Ok(result) => {
                let response = lsp_server::Response::new_ok(id, result);
                client.send_response(response).unwrap();
            }
            Err(why) => {
                client
                    .send_error(id, ErrorCode::InternalError, why.to_string())
                    .unwrap();
            }
        });
    }

    fn register_configuration(&mut self) {
        if self.client_flags.configuration_push {
            let registration = Registration {
                id: "pull-config".to_string(),
                method: DidChangeConfiguration::METHOD.to_string(),
                register_options: None,
            };

            let params = RegistrationParams {
                registrations: vec![registration],
            };

            let client = self.client.clone();
            self.pool.execute(move || {
                if let Err(why) = client.send_request::<RegisterCapability>(params) {
                    log::error!(
                        "Failed to register \"{}\" notification: {}",
                        DidChangeConfiguration::METHOD,
                        why
                    );
                }
            });
        }
    }

    fn update_workspace(&mut self) {
        let mut checked_paths = FxHashSet::default();
        let mut workspace = self.workspace.write();
        base_db::deps::discover(&mut workspace, &mut checked_paths);
        self.watcher.watch(&mut workspace);

        for document in checked_paths
            .iter()
            .filter_map(|path| workspace.lookup_file(path))
        {
            self.diagnostic_manager.update_syntax(&workspace, document);
        }

        drop(workspace);
        self.publish_diagnostics_with_delay();
    }

    fn publish_diagnostics(&mut self) -> Result<()> {
        let workspace = self.workspace.read();

        for (uri, diagnostics) in self.diagnostic_manager.get(&workspace) {
            let Some(document) = workspace.lookup(&uri) else {
                continue;
            };

            let diagnostics = diagnostics
                .into_iter()
                .filter_map(|diagnostic| to_proto::diagnostic(&workspace, document, &diagnostic))
                .collect();

            let version = None;
            let params = PublishDiagnosticsParams {
                uri,
                diagnostics,
                version,
            };

            self.client
                .send_notification::<PublishDiagnostics>(params)?;
        }

        Ok(())
    }

    fn publish_diagnostics_with_delay(&mut self) {
        let sender = self.internal_tx.clone();
        let delay = self.workspace.read().config().diagnostics.delay;
        self.pool.execute(move || {
            std::thread::sleep(delay);
            sender.send(InternalMessage::Diagnostics).unwrap();
        });
    }

    fn pull_options(&mut self) {
        if !self.client_flags.configuration_pull {
            return;
        }

        let params = ConfigurationParams {
            items: vec![ConfigurationItem {
                section: Some("texlab".to_string()),
                scope_uri: None,
            }],
        };

        let client = self.client.clone();
        let sender = self.internal_tx.clone();
        self.pool.execute(move || {
            match client.send_request::<WorkspaceConfiguration>(params) {
                Ok(mut json) => {
                    let options = client
                        .parse_options(json.pop().expect("invalid configuration request"))
                        .unwrap();

                    sender
                        .send(InternalMessage::SetOptions(Box::new(options)))
                        .unwrap();
                }
                Err(why) => {
                    log::error!("Retrieving configuration failed: {}", why);
                }
            };
        });
    }

    fn update_options(&mut self, options: Options) {
        let mut workspace = self.workspace.write();
        workspace.set_config(from_proto::config(options));
        self.watcher.watch(&mut workspace);
    }

    fn cancel(&self, _params: CancelParams) -> Result<()> {
        Ok(())
    }

    fn did_change_watched_files(&mut self, _params: DidChangeWatchedFilesParams) -> Result<()> {
        Ok(())
    }

    fn did_change_configuration(&mut self, params: DidChangeConfigurationParams) -> Result<()> {
        if self.client_flags.configuration_pull {
            self.pull_options();
        } else {
            let options = self.client.parse_options(params.settings)?;
            self.update_options(options);
        }

        Ok(())
    }

    fn did_open(&mut self, params: DidOpenTextDocumentParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);

        let language_id = &params.text_document.language_id;
        let language = Language::from_id(language_id).unwrap_or(Language::Tex);
        self.workspace.write().open(
            uri.clone(),
            params.text_document.text,
            language,
            Owner::Client,
            LineCol { line: 0, col: 0 },
        );

        self.update_workspace();

        let workspace = self.workspace.read();
        self.diagnostic_manager
            .update_syntax(&workspace, workspace.lookup(&uri).unwrap());

        if workspace.config().diagnostics.chktex.on_open {
            drop(workspace);
            self.run_chktex(&uri);
        }

        Ok(())
    }

    fn did_change(&mut self, params: DidChangeTextDocumentParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);

        let mut workspace = self.workspace.write();

        for change in params.content_changes {
            let Some(document) = workspace.lookup(&uri) else {
                return Ok(());
            };
            match change.range {
                Some(range) => {
                    let range = document.line_index.offset_lsp_range(range).unwrap();
                    workspace.edit(&uri, range, &change.text);
                }
                None => {
                    let new_line = document.cursor.line.min(change.text.lines().count() as u32);
                    let language = document.language;
                    workspace.open(
                        uri.clone(),
                        change.text,
                        language,
                        Owner::Client,
                        LineCol {
                            line: new_line,
                            col: 0,
                        },
                    );
                }
            };
        }

        self.diagnostic_manager
            .update_syntax(&workspace, workspace.lookup(&uri).unwrap());

        drop(workspace);
        self.update_workspace();

        if self.workspace.read().config().diagnostics.chktex.on_edit {
            self.run_chktex(&uri);
        }

        Ok(())
    }

    fn did_save(&mut self, params: DidSaveTextDocumentParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);

        if self.workspace.read().config().build.on_save {
            let text_document = TextDocumentIdentifier::new(uri.clone());
            let params = BuildParams {
                text_document,
                position: None,
            };

            self.build(None, params)?;
        }

        self.publish_diagnostics_with_delay();

        if self.workspace.read().config().diagnostics.chktex.on_save {
            self.run_chktex(&uri);
        }

        Ok(())
    }

    fn did_close(&mut self, params: DidCloseTextDocumentParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);
        self.workspace.write().close(&uri);
        self.publish_diagnostics_with_delay();
        Ok(())
    }

    fn run_chktex(&mut self, uri: &Url) -> Option<()> {
        let workspace = self.workspace.read();

        let document = workspace.lookup(uri)?;
        let command = diagnostics::chktex::Command::new(&workspace, document)?;

        let sender = self.internal_tx.clone();
        let uri = document.uri.clone();
        self.pool.execute(move || {
            let diagnostics = command.run().unwrap_or_default();
            sender
                .send(InternalMessage::ChktexFinished(uri, diagnostics))
                .unwrap();
        });

        Some(())
    }

    fn document_link(&self, id: RequestId, mut params: DocumentLinkParams) -> Result<()> {
        normalize_uri(&mut params.text_document.uri);
        self.run_query(id, move |workspace| {
            link::find_all(workspace, params).unwrap_or_default()
        });
        Ok(())
    }

    fn document_symbols(&self, id: RequestId, mut params: DocumentSymbolParams) -> Result<()> {
        normalize_uri(&mut params.text_document.uri);

        let client_flags = Arc::clone(&self.client_flags);
        self.run_query(id, move |workspace| {
            symbols::document_symbols(workspace, params, &client_flags)
        });

        Ok(())
    }

    fn workspace_symbols(&self, id: RequestId, params: WorkspaceSymbolParams) -> Result<()> {
        self.run_query(id, move |workspace| {
            symbols::workspace_symbols(workspace, &params.query)
        });

        Ok(())
    }

    fn completion(&self, id: RequestId, mut params: CompletionParams) -> Result<()> {
        normalize_uri(&mut params.text_document_position.text_document.uri);
        let position = params.text_document_position.position;
        let client_flags = Arc::clone(&self.client_flags);
        self.update_cursor(&params.text_document_position.text_document.uri, position);
        self.run_query(id, move |db| {
            completion::complete(db, params, &client_flags)
        });

        Ok(())
    }

    fn update_cursor(&self, uri: &Url, position: Position) {
        self.workspace.write().set_cursor(
            uri,
            LineCol {
                line: position.line,
                col: 0,
            },
        );
    }

    fn completion_resolve(&self, id: RequestId, mut item: CompletionItem) -> Result<()> {
        self.run_query(id, move |workspace| {
            completion::resolve(workspace, &mut item);
            item
        });

        Ok(())
    }

    fn folding_range(&self, id: RequestId, mut params: FoldingRangeParams) -> Result<()> {
        normalize_uri(&mut params.text_document.uri);
        let client_flags = Arc::clone(&self.client_flags);
        self.run_query(id, move |db| {
            folding::find_all(db, params, &client_flags).unwrap_or_default()
        });

        Ok(())
    }

    fn references(&self, id: RequestId, mut params: ReferenceParams) -> Result<()> {
        normalize_uri(&mut params.text_document_position.text_document.uri);
        self.run_query(id, move |db| {
            reference::find_all(db, params).unwrap_or_default()
        });

        Ok(())
    }

    fn hover(&mut self, id: RequestId, mut params: HoverParams) -> Result<()> {
        normalize_uri(&mut params.text_document_position_params.text_document.uri);
        let uri_and_pos = &params.text_document_position_params;
        let client_flags = Arc::clone(&self.client_flags);
        self.update_cursor(&uri_and_pos.text_document.uri, uri_and_pos.position);
        self.run_query(id, move |db| hover::find(db, params, &client_flags));
        Ok(())
    }

    fn goto_definition(&self, id: RequestId, mut params: GotoDefinitionParams) -> Result<()> {
        normalize_uri(&mut params.text_document_position_params.text_document.uri);
        self.run_query(id, move |db| definition::goto_definition(db, params));
        Ok(())
    }

    fn prepare_rename(&self, id: RequestId, mut params: TextDocumentPositionParams) -> Result<()> {
        normalize_uri(&mut params.text_document.uri);
        self.run_query(id, move |db| rename::prepare_rename_all(db, params));
        Ok(())
    }

    fn rename(&self, id: RequestId, mut params: RenameParams) -> Result<()> {
        normalize_uri(&mut params.text_document_position.text_document.uri);
        self.run_query(id, move |db| rename::rename_all(db, params));
        Ok(())
    }

    fn document_highlight(&self, id: RequestId, mut params: DocumentHighlightParams) -> Result<()> {
        normalize_uri(&mut params.text_document_position_params.text_document.uri);
        self.run_query(id, move |db| {
            highlight::find_all(db, params).unwrap_or_default()
        });

        Ok(())
    }

    fn formatting(&self, id: RequestId, params: DocumentFormattingParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);
        self.run_query(id, move |db| {
            formatting::format_source_code(db, &uri, &params.options)
        });

        Ok(())
    }

    fn execute_command(&self, id: RequestId, params: ExecuteCommandParams) -> Result<()> {
        match params.command.as_str() {
            "texlab.cleanAuxiliary" => {
                let command = self.prepare_clean_command(params, CleanTarget::Auxiliary);
                self.run_fallible(id, || command?.run());
            }
            "texlab.cleanArtifacts" => {
                let command = self.prepare_clean_command(params, CleanTarget::Artifacts);
                self.run_fallible(id, || command?.run());
            }
            "texlab.changeEnvironment" => {
                let client = self.client.clone();
                let params = self.change_environment(params);
                self.run_fallible(id, move || {
                    client.send_request::<ApplyWorkspaceEdit>(params?)
                });
            }
            "texlab.findEnvironments" => {
                let result = self.find_environments(params);
                self.run_fallible(id, move || result);
            }
            "texlab.showDependencyGraph" => {
                let workspace = self.workspace.read();
                let dot = commands::show_dependency_graph(&workspace).unwrap();
                self.client
                    .send_response(lsp_server::Response::new_ok(id, dot))?;
            }
            "texlab.cancelBuild" => {
                let pending_builds = Arc::clone(&self.pending_builds);
                self.run_fallible(id, move || {
                    for pid in pending_builds.lock().drain() {
                        let _ = BuildCommand::cancel(pid);
                    }

                    Ok(())
                });
            }
            _ => {
                self.client
                    .send_error(
                        id,
                        ErrorCode::InvalidParams,
                        format!("Unknown workspace command: {}", params.command),
                    )
                    .unwrap();
            }
        };

        Ok(())
    }

    fn inlay_hints(&self, id: RequestId, mut params: InlayHintParams) -> Result<()> {
        normalize_uri(&mut params.text_document.uri);
        self.run_query(id, move |db| {
            inlay_hint::find_all(db, params).unwrap_or_default()
        });
        Ok(())
    }

    fn inlay_hint_resolve(&self, id: RequestId, hint: InlayHint) -> Result<()> {
        let response = lsp_server::Response::new_ok(id, hint);
        self.client.send_response(response)?;
        Ok(())
    }

    fn semantic_tokens_range(
        &self,
        _id: RequestId,
        _params: SemanticTokensRangeParams,
    ) -> Result<()> {
        Ok(())
    }

    fn build(&self, id: Option<RequestId>, params: BuildParams) -> Result<()> {
        static LOCK: Mutex<()> = Mutex::new(());
        static NEXT_TOKEN: AtomicI32 = AtomicI32::new(1);

        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);

        let workspace = self.workspace.read();

        let client = self.client.clone();

        let fwd_search_after = workspace.config().build.forward_search_after;

        let (sender, receiver) = crossbeam_channel::unbounded();
        self.redirect_build_log(receiver);

        let command = BuildCommand::new(&workspace, &uri);
        let internal = self.internal_tx.clone();
        let progress = self.client_flags.progress;
        let pending_builds = Arc::clone(&self.pending_builds);

        self.pool.execute(move || {
            let guard = LOCK.lock();

            let progress_reporter = if progress {
                let token = NEXT_TOKEN.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Some(ProgressReporter::new(client.clone(), token, &uri))
            } else {
                None
            };

            let status = command
                .and_then(|command| {
                    let mut process = command.spawn(sender)?;
                    let pid = process.id();
                    pending_builds.lock().insert(pid);
                    let result = process.wait();

                    let status = if pending_builds.lock().remove(&pid) {
                        if result?.success() {
                            BuildStatus::Success
                        } else {
                            BuildStatus::Error
                        }
                    } else {
                        BuildStatus::Cancelled
                    };

                    Ok(status)
                })
                .unwrap_or_else(|why| {
                    log::error!("Failed to compile document \"{uri}\": {why}");
                    BuildStatus::Failure
                });

            drop(progress_reporter);
            drop(guard);

            if let Some(id) = id {
                let result = BuildResult { status };
                let _ = client.send_response(lsp_server::Response::new_ok(id, result));
            }

            if fwd_search_after && status != BuildStatus::Cancelled {
                let _ = internal.send(InternalMessage::ForwardSearch(uri, params.position));
            }
        });

        Ok(())
    }

    fn redirect_build_log(&self, receiver: Receiver<String>) {
        let client = self.client.clone();
        self.pool.execute(move || {
            let typ = MessageType::LOG;
            for message in receiver {
                client
                    .send_notification::<LogMessage>(LogMessageParams { message, typ })
                    .unwrap();
            }
        });
    }

    fn forward_search(
        &self,
        id: Option<RequestId>,
        mut uri: Url,
        position: Option<Position>,
    ) -> Result<()> {
        normalize_uri(&mut uri);

        let client = self.client.clone();
        let command = ForwardSearch::new(
            &self.workspace.read(),
            &uri,
            position.map(|position| position.line),
        );

        self.pool.execute(move || {
            let status = match command.and_then(ForwardSearch::run) {
                Ok(()) => ForwardSearchStatus::Success,
                Err(why) => {
                    log::error!("Failed to execute forward search: {why}");
                    ForwardSearchStatus::from(why)
                }
            };

            if let Some(id) = id {
                let result = ForwardSearchResult { status };
                client
                    .send_response(lsp_server::Response::new_ok(id, result))
                    .unwrap();
            }
        });

        Ok(())
    }

    fn code_actions(&self, id: RequestId, _params: CodeActionParams) -> Result<()> {
        self.client
            .send_response(lsp_server::Response::new_ok(id, Vec::<CodeAction>::new()))?;
        Ok(())
    }

    fn code_action_resolve(&self, id: RequestId, action: CodeAction) -> Result<()> {
        self.client
            .send_response(lsp_server::Response::new_ok(id, action))?;
        Ok(())
    }

    fn handle_file_event(&mut self, debounced_event: DebouncedEvent) {
        let event = debounced_event.event;
        let mut changed = false;

        let mut workspace = self.workspace.write();

        match event.kind {
            notify::EventKind::Remove(_) | notify::EventKind::Modify(ModifyKind::Name(_)) => {
                let affected_uris = event
                    .paths
                    .iter()
                    .flat_map(|file_or_dir| workspace.lookup_file_or_dir(file_or_dir))
                    .filter(|doc| doc.owner == Owner::Server)
                    .map(|doc| doc.uri.clone())
                    .collect::<Vec<_>>();

                for uri in affected_uris {
                    workspace.remove(&uri);
                    changed = true;
                }
            }
            notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                for file_or_dir in event.paths {
                    let affected_paths = if file_or_dir.is_dir() {
                        changed = true;
                        workspace
                            .lookup_file_or_dir(&file_or_dir)
                            .filter_map(|doc| doc.path.clone())
                            .collect::<Vec<_>>()
                    } else {
                        vec![file_or_dir]
                    };

                    for path in affected_paths {
                        if !workspace
                            .lookup_file(&path)
                            .map_or(true, |doc| doc.owner == Owner::Server)
                        {
                            continue;
                        }

                        let Some(language) = Language::from_path(&path) else {
                            continue;
                        };

                        changed |= workspace.load(&path, language).is_ok();

                        if let Some(document) = workspace.lookup_file(&path) {
                            self.diagnostic_manager.update_syntax(&workspace, document);
                        }
                    }
                }
            }
            notify::EventKind::Any | notify::EventKind::Access(_) | notify::EventKind::Other => {}
        };

        if changed {
            self.diagnostic_manager.cleanup(&workspace);
            drop(workspace);
            self.update_workspace();
        }
    }

    fn prepare_clean_command(
        &self,
        params: ExecuteCommandParams,
        target: CleanTarget,
    ) -> Result<CleanCommand> {
        let workspace = self.workspace.read();
        let mut params = self.parse_command_params::<TextDocumentIdentifier>(params.arguments)?;
        normalize_uri(&mut params.uri);
        let Some(document) = workspace.lookup(&params.uri) else {
            anyhow::bail!("Document {} is not opened!", params.uri)
        };

        CleanCommand::new(&workspace, document, target)
    }

    fn change_environment(&self, params: ExecuteCommandParams) -> Result<ApplyWorkspaceEditParams> {
        let workspace = self.workspace.read();
        let params = self.parse_command_params::<RenameParams>(params.arguments)?;
        let mut uri = params.text_document_position.text_document.uri;
        normalize_uri(&mut uri);

        let Some(document) = workspace.lookup(&uri) else {
            anyhow::bail!("Document {} is not opened!", uri)
        };

        let line_index = &document.line_index;
        let Some(position) = line_index.offset_lsp(params.text_document_position.position) else {
            anyhow::bail!("Invalid position for document {uri}!")
        };

        let Some(result) = commands::change_environment(document, position, &params.new_name)
        else {
            anyhow::bail!("No environment found at the current position");
        };

        let range1 = line_index.line_col_lsp_range(result.begin).unwrap();
        let range2 = line_index.line_col_lsp_range(result.end).unwrap();

        let mut changes = HashMap::new();
        changes.insert(
            document.uri.clone(),
            vec![
                TextEdit::new(range1, params.new_name.clone()),
                TextEdit::new(range2, params.new_name.clone()),
            ],
        );

        let label = Some(format!(
            "change environment: {} -> {}",
            result.old_name, result.new_name
        ));

        let edit = WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        };

        Ok(ApplyWorkspaceEditParams { label, edit })
    }

    fn find_environments(&self, params: ExecuteCommandParams) -> Result<Vec<EnvironmentLocation>> {
        let workspace = self.workspace.read();
        let params = self.parse_command_params::<TextDocumentPositionParams>(params.arguments)?;
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);

        let Some(document) = workspace.lookup(&uri) else {
            anyhow::bail!("Document {} is not opened!", uri)
        };

        let line_index = &document.line_index;
        let Some(offset) = line_index.offset_lsp(params.position) else {
            anyhow::bail!("Invalid position for document {uri}!")
        };

        let results = commands::find_environments(document, offset)
            .into_iter()
            .map(|result| EnvironmentLocation {
                name: TextWithRange {
                    range: line_index.line_col_lsp_range(result.name.range).unwrap(),
                    text: result.name.text,
                },
                full_range: line_index.line_col_lsp_range(result.full_range).unwrap(),
            })
            .collect();

        Ok(results)
    }

    fn inverse_search(&self, params: TextDocumentPositionParams) -> Result<()> {
        if !self.client_flags.show_document {
            log::warn!("Inverse search request received although the client does not support window/showDocument: {params:?}");
        }

        let position = lsp_types::Position::new(params.position.line, 0);
        let params = lsp_types::ShowDocumentParams {
            uri: params.text_document.uri,
            take_focus: Some(true),
            external: Some(false),
            selection: Some(lsp_types::Range::new(position, position)),
        };

        let client = self.client.clone();
        self.pool.execute(move || {
            let _ = client.send_request::<ShowDocument>(params);
        });

        Ok(())
    }

    fn parse_command_params<T: DeserializeOwned>(
        &self,
        params: Vec<serde_json::Value>,
    ) -> Result<T> {
        if params.is_empty() {
            anyhow::bail!("No argument provided!");
        }

        let value = params.into_iter().next().unwrap();
        let value = serde_json::from_value(value)?;
        Ok(value)
    }

    fn setup_ipc_server(&mut self) {
        let sender = self.internal_tx.clone();
        let _ = ipc::spawn_server(move |params: TextDocumentPositionParams| {
            let _ = sender.send(InternalMessage::InverseSearch(params));
        });
    }

    fn process_messages(&mut self) -> Result<()> {
        loop {
            crossbeam_channel::select! {
                recv(&self.connection.receiver) -> msg => {
                    match msg? {
                        Message::Request(request) => {
                            if self.connection.handle_shutdown(&request)? {
                                return Ok(());
                            }

                            if let Some(response) = dispatch::RequestDispatcher::new(request)
                                .on::<DocumentLinkRequest, _>(|id, params| self.document_link(id, params))?
                                .on::<FoldingRangeRequest, _>(|id, params| self.folding_range(id, params))?
                                .on::<References, _>(|id, params| self.references(id, params))?
                                .on::<HoverRequest, _>(|id, params| self.hover(id, params))?
                                .on::<DocumentSymbolRequest, _>(|id, params| {
                                    self.document_symbols(id, params)
                                })?
                                .on::<WorkspaceSymbolRequest, _>(|id, params| self.workspace_symbols(id, params))?
                                .on::<Completion, _>(|id, params| {
                                    self.completion(id, params)?;
                                    Ok(())
                                })?
                                .on::<ResolveCompletionItem, _>(|id, params| {
                                    self.completion_resolve(id, params)?;
                                    Ok(())
                                })?
                                .on::<GotoDefinition, _>(|id, params| self.goto_definition(id, params))?
                                .on::<PrepareRenameRequest, _>(|id, params| {
                                    self.prepare_rename(id, params)
                                })?
                                .on::<Rename, _>(|id, params| self.rename(id, params))?
                                .on::<DocumentHighlightRequest, _>(|id, params| {
                                    self.document_highlight(id, params)
                                })?
                                .on::<Formatting, _>(|id, params| self.formatting(id, params))?
                                .on::<BuildRequest, _>(|id, params| self.build(Some(id), params))?
                                .on::<ForwardSearchRequest, _>(|id, params| {
                                    self.forward_search(Some(id), params.text_document.uri, Some(params.position))
                                })?
                                .on::<ExecuteCommand,_>(|id, params| self.execute_command(id, params))?
                                .on::<SemanticTokensRangeRequest, _>(|id, params| {
                                    self.semantic_tokens_range(id, params)
                                })?
                                .on::<InlayHintRequest, _>(|id,params| {
                                    self.inlay_hints(id, params)
                                })?
                                .on::<InlayHintResolveRequest,_>(|id, params| {
                                    self.inlay_hint_resolve(id, params)
                                })?
                                .on::<CodeActionRequest, _>(|id, params| {
                                    self.code_actions(id, params)
                                })?
                                .on::<CodeActionResolveRequest, _>(|id, params| {
                                    self.code_action_resolve(id, params)
                                })?
                                .default()
                            {
                                self.client.send_response(response)?;
                            }
                        }
                        Message::Notification(notification) => {
                            dispatch::NotificationDispatcher::new(notification)
                                .on::<Cancel, _>(|params| self.cancel(params))?
                                .on::<DidChangeConfiguration, _>(|params| {
                                    self.did_change_configuration(params)
                                })?
                                .on::<DidChangeWatchedFiles, _>(|params| {
                                    self.did_change_watched_files(params)
                                })?
                                .on::<DidOpenTextDocument, _>(|params| self.did_open(params))?
                                .on::<DidChangeTextDocument, _>(|params| self.did_change(params))?
                                .on::<DidSaveTextDocument, _>(|params| self.did_save(params))?
                                .on::<DidCloseTextDocument, _>(|params| self.did_close(params))?
                                .default();
                        }
                        Message::Response(response) => {
                            self.client.recv_response(response)?;
                        }
                    };
                },
                recv(&self.internal_rx) -> msg => {
                    match msg? {
                        InternalMessage::SetDistro(distro) => {
                            self.workspace.write().set_distro(distro);
                        }
                        InternalMessage::SetOptions(options) => {
                            self.update_options(*options);
                        }
                        InternalMessage::FileEvent(events) => {
                            for event in events {
                                self.handle_file_event(event);
                            }
                        }
                        InternalMessage::Diagnostics => {
                            self.publish_diagnostics()?;
                        }
                        InternalMessage::ChktexFinished(uri, diagnostics) => {
                            self.diagnostic_manager.update_chktex(uri, diagnostics);
                            self.publish_diagnostics()?;
                        }
                        InternalMessage::ForwardSearch(uri, position) => {
                            self.forward_search(None, uri, position)?;
                        }
                        InternalMessage::InverseSearch(params) => {
                            self.inverse_search(params)?;
                        }
                    };
                }
            };
        }
    }

    pub fn run(mut self, options: StartupOptions) -> Result<()> {
        if !options.skip_distro {
            let sender = self.internal_tx.clone();
            self.pool.execute(move || {
                let distro = Distro::detect().unwrap_or_else(|why| {
                    log::warn!("Unable to load distro files: {}", why);
                    Distro::default()
                });

                log::info!("Detected distribution: {:?}", distro.kind);
                sender.send(InternalMessage::SetDistro(distro)).unwrap();
            });
        }

        self.register_configuration();
        self.pull_options();
        self.setup_ipc_server();
        self.process_messages()?;
        self.pool.join();
        Ok(())
    }
}

struct FileWatcher {
    watcher: Debouncer<notify::RecommendedWatcher, RecommendedCache>,
    watched_dirs: FxHashSet<PathBuf>,
}

impl FileWatcher {
    pub fn new(sender: Sender<InternalMessage>) -> Result<Self> {
        let handle = move |event| {
            if let Ok(event) = event {
                let _ = sender.send(InternalMessage::FileEvent(event));
            }
        };

        Ok(Self {
            watcher: notify_debouncer_full::new_debouncer(Duration::from_secs(1), None, handle)?,
            watched_dirs: FxHashSet::default(),
        })
    }

    pub fn watch(&mut self, workspace: &mut Workspace) {
        deps::watch(workspace, &mut self.watcher, &mut self.watched_dirs);
    }
}
