mod dispatch;
mod extensions;
pub mod options;
mod progress;

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{atomic::AtomicI32, Arc},
};

use anyhow::Result;
use base_db::{util::LineCol, Config, Owner, Workspace};
use commands::{BuildCommand, CleanCommand, CleanTarget, ForwardSearch};
use crossbeam_channel::{Receiver, Sender};
use distro::{Distro, Language};
use lsp_server::{Connection, ErrorCode, Message, RequestId};
use lsp_types::{notification::*, request::*, *};
use parking_lot::{Mutex, RwLock};
use rowan::ast::AstNode;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{de::DeserializeOwned, Serialize};
use syntax::bibtex;
use threadpool::ThreadPool;

use crate::{
    client::LspClient,
    features::{
        completion::{self, builder::CompletionItemData},
        definition, folding, formatting, highlight, hover, inlay_hint, link, reference, rename,
        symbols,
    },
    util::{
        self, capabilities::ClientCapabilitiesExt, components::COMPONENT_DATABASE,
        line_index_ext::LineIndexExt, normalize_uri,
    },
};

use self::{
    extensions::{
        BuildParams, BuildRequest, BuildResult, BuildStatus, ForwardSearchRequest,
        ForwardSearchResult, ForwardSearchStatus,
    },
    options::{Options, StartupOptions},
    progress::ProgressReporter,
};

#[derive(Debug)]
enum InternalMessage {
    SetDistro(Distro),
    SetOptions(Options),
    FileEvent(notify::Event),
    Diagnostics,
    ChktexResult(Url, Vec<lsp_types::Diagnostic>),
    ForwardSearch(Url, Option<Position>),
}

pub struct Server {
    connection: Arc<Connection>,
    internal_tx: Sender<InternalMessage>,
    internal_rx: Receiver<InternalMessage>,
    workspace: Arc<RwLock<Workspace>>,
    client: LspClient,
    client_capabilities: Arc<ClientCapabilities>,
    client_info: Option<Arc<ClientInfo>>,
    chktex_diagnostics: FxHashMap<Url, Vec<Diagnostic>>,
    watcher: FileWatcher,
    pool: ThreadPool,
}

impl Server {
    pub fn new(connection: Connection) -> Self {
        let client = LspClient::new(connection.sender.clone());
        let (internal_tx, internal_rx) = crossbeam_channel::unbounded();
        let watcher = FileWatcher::new(internal_tx.clone()).expect("init file watcher");

        Self {
            connection: Arc::new(connection),
            internal_tx,
            internal_rx,
            workspace: Default::default(),
            client,
            client_capabilities: Default::default(),
            client_info: Default::default(),
            chktex_diagnostics: Default::default(),
            watcher,
            pool: threadpool::Builder::new().build(),
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

    fn capabilities(&self) -> ServerCapabilities {
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
                ],
                ..Default::default()
            }),
            inlay_hint_provider: Some(OneOf::Left(true)),
            ..ServerCapabilities::default()
        }
    }

    fn initialize(&mut self) -> Result<()> {
        let (id, params) = self.connection.initialize_start()?;
        let params: InitializeParams = serde_json::from_value(params)?;

        self.client_capabilities = Arc::new(params.capabilities);
        self.client_info = params.client_info.map(Arc::new);

        let workspace_folders = params
            .workspace_folders
            .unwrap_or_default()
            .into_iter()
            .filter(|folder| folder.uri.scheme() == "file")
            .flat_map(|folder| folder.uri.to_file_path())
            .collect();

        self.workspace.write().set_folders(workspace_folders);

        let result = InitializeResult {
            capabilities: self.capabilities(),
            server_info: Some(ServerInfo {
                name: "TexLab".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
        };
        self.connection
            .initialize_finish(id, serde_json::to_value(result)?)?;

        let StartupOptions { skip_distro } =
            serde_json::from_value(params.initialization_options.unwrap_or_default())
                .unwrap_or_default();

        if !skip_distro {
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
        Ok(())
    }

    fn register_configuration(&mut self) {
        if self.client_capabilities.has_push_configuration_support() {
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
        let mut workspace = self.workspace.write();
        workspace.discover();
        self.watcher.watch(&mut workspace);
        drop(workspace);
        self.publish_diagnostics_with_delay();
    }

    fn publish_diagnostics(&mut self) -> Result<()> {
        let workspace = self.workspace.read();
        let mut all_diagnostics = util::diagnostics::collect(&workspace);

        for (uri, diagnostics) in &self.chktex_diagnostics {
            let Some(document) = workspace.lookup(uri) else { continue };
            let Some(existing) = all_diagnostics.get_mut(document) else { continue };
            existing.extend(diagnostics.iter().cloned());
        }

        util::diagnostics::filter(&mut all_diagnostics, &workspace);

        for (document, diagnostics) in all_diagnostics {
            let uri = document.uri.clone();
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
        if !self.client_capabilities.has_pull_configuration_support() {
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

                    sender.send(InternalMessage::SetOptions(options)).unwrap();
                }
                Err(why) => {
                    log::error!("Retrieving configuration failed: {}", why);
                }
            };
        });
    }

    fn update_options(&mut self, options: Options) {
        let mut workspace = self.workspace.write();
        workspace.set_config(Config::from(options));
        self.watcher.watch(&mut workspace);
    }

    fn cancel(&self, _params: CancelParams) -> Result<()> {
        Ok(())
    }

    fn did_change_watched_files(&mut self, _params: DidChangeWatchedFilesParams) -> Result<()> {
        Ok(())
    }

    fn did_change_configuration(&mut self, params: DidChangeConfigurationParams) -> Result<()> {
        if self.client_capabilities.has_pull_configuration_support() {
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

        if self.workspace.read().config().diagnostics.chktex.on_open {
            self.run_chktex(&uri);
        }

        Ok(())
    }

    fn did_change(&mut self, params: DidChangeTextDocumentParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);

        let mut workspace = self.workspace.write();

        for change in params.content_changes {
            let Some(document) = workspace.lookup(&uri) else { return Ok(()) };
            match change.range {
                Some(range) => {
                    let range = document.line_index.offset_lsp_range(range);
                    drop(document);
                    workspace.edit(&uri, range, &change.text);
                }
                None => {
                    let new_line = document.cursor.line.min(change.text.lines().count() as u32);
                    let language = document.language;
                    drop(document);
                    workspace.open(
                        uri.clone(),
                        change.text,
                        language,
                        Owner::Client,
                        LineCol {
                            line: new_line as u32,
                            col: 0,
                        },
                    );
                }
            };
        }

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

    fn run_chktex(&mut self, uri: &Url) {
        let workspace = self.workspace.read();
        let Some(document) = workspace.lookup(uri) else { return };
        let Some(command) = util::chktex::Command::new(&workspace, document) else { return };

        let sender = self.internal_tx.clone();
        let uri = document.uri.clone();
        self.pool.execute(move || {
            let diagnostics = command.run().unwrap_or_default();
            sender
                .send(InternalMessage::ChktexResult(uri, diagnostics))
                .unwrap();
        });
    }

    fn document_link(&self, id: RequestId, params: DocumentLinkParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);
        self.run_query(id, move |workspace| {
            link::find_all(workspace, &uri).unwrap_or_default()
        });
        Ok(())
    }

    fn document_symbols(&self, id: RequestId, params: DocumentSymbolParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);

        let capabilities = Arc::clone(&self.client_capabilities);
        self.run_query(id, move |workspace| {
            let Some(document) = workspace.lookup(&uri) else {
                return DocumentSymbolResponse::Flat(vec![]);
            };

            symbols::document_symbols(workspace, document, &capabilities)
        });

        Ok(())
    }

    fn workspace_symbols(&self, id: RequestId, params: WorkspaceSymbolParams) -> Result<()> {
        self.run_query(id, move |workspace| {
            symbols::workspace_symbols(workspace, &params.query)
        });

        Ok(())
    }

    fn completion(&self, id: RequestId, params: CompletionParams) -> Result<()> {
        let mut uri = params.text_document_position.text_document.uri;
        normalize_uri(&mut uri);
        let position = params.text_document_position.position;
        let client_capabilities = Arc::clone(&self.client_capabilities);
        let client_info = self.client_info.clone();

        self.update_cursor(&uri, position);

        self.run_query(id, move |db| {
            completion::complete(
                db,
                &uri,
                position,
                &client_capabilities,
                client_info.as_deref(),
            )
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
            match item
                .data
                .clone()
                .map(|data| serde_json::from_value(data).unwrap())
            {
                Some(CompletionItemData::Package | CompletionItemData::Class) => {
                    item.documentation = COMPONENT_DATABASE
                        .documentation(&item.label)
                        .map(Documentation::MarkupContent);
                }
                Some(CompletionItemData::Citation { uri, key }) => {
                    if let Some(data) = workspace
                        .lookup(&uri)
                        .and_then(|document| document.data.as_bib())
                    {
                        item.documentation = bibtex::Root::cast(data.root_node())
                            .and_then(|root| root.find_entry(&key))
                            .and_then(|entry| citeproc::render(&entry))
                            .map(|value| {
                                Documentation::MarkupContent(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value,
                                })
                            });
                    }
                }
                None => {}
            };

            item
        });

        Ok(())
    }

    fn folding_range(&self, id: RequestId, params: FoldingRangeParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);
        self.run_query(id, move |db| {
            folding::find_all(db, &uri).unwrap_or_default()
        });
        Ok(())
    }

    fn references(&self, id: RequestId, params: ReferenceParams) -> Result<()> {
        let mut uri = params.text_document_position.text_document.uri;
        normalize_uri(&mut uri);
        let position = params.text_document_position.position;
        self.run_query(id, move |db| {
            reference::find_all(db, &uri, position, &params.context).unwrap_or_default()
        });

        Ok(())
    }

    fn hover(&mut self, id: RequestId, params: HoverParams) -> Result<()> {
        let mut uri = params.text_document_position_params.text_document.uri;
        normalize_uri(&mut uri);

        let position = params.text_document_position_params.position;
        self.update_cursor(&uri, position);

        self.run_query(id, move |db| hover::find(db, &uri, position));
        Ok(())
    }

    fn goto_definition(&self, id: RequestId, params: GotoDefinitionParams) -> Result<()> {
        let mut uri = params.text_document_position_params.text_document.uri;
        normalize_uri(&mut uri);
        let position = params.text_document_position_params.position;
        self.run_query(id, move |db| {
            definition::goto_definition(db, &uri, position)
        });

        Ok(())
    }

    fn prepare_rename(&self, id: RequestId, params: TextDocumentPositionParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);
        self.run_query(id, move |db| {
            rename::prepare_rename_all(db, &uri, params.position)
        });

        Ok(())
    }

    fn rename(&self, id: RequestId, params: RenameParams) -> Result<()> {
        let mut uri = params.text_document_position.text_document.uri;
        normalize_uri(&mut uri);
        let position = params.text_document_position.position;
        self.run_query(id, move |db| {
            rename::rename_all(db, &uri, position, params.new_name)
        });

        Ok(())
    }

    fn document_highlight(&self, id: RequestId, params: DocumentHighlightParams) -> Result<()> {
        let mut uri = params.text_document_position_params.text_document.uri;
        normalize_uri(&mut uri);
        let position = params.text_document_position_params.position;
        self.run_query(id, move |db| {
            highlight::find_all(db, &uri, position).unwrap_or_default()
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
            "texlab.showDependencyGraph" => {
                let workspace = self.workspace.read();
                let dot = commands::show_dependency_graph(&workspace).unwrap();
                self.client
                    .send_response(lsp_server::Response::new_ok(id, dot))?;
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

    fn inlay_hints(&self, id: RequestId, params: InlayHintParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);
        self.run_query(id, move |db| {
            inlay_hint::find_all(db, &uri, params.range).unwrap_or_default()
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
        let progress = self.client_capabilities.has_work_done_progress_support();
        self.pool.execute(move || {
            let guard = LOCK.lock();

            let progress_reporter = if progress {
                let token = NEXT_TOKEN.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Some(ProgressReporter::new(client.clone(), token, &uri))
            } else {
                None
            };

            let status = match command.and_then(|command| command.run(sender)) {
                Ok(status) if status.success() => BuildStatus::SUCCESS,
                Ok(_) => BuildStatus::ERROR,
                Err(why) => {
                    log::error!("Failed to compile document \"{uri}\": {why}");
                    BuildStatus::FAILURE
                }
            };

            drop(progress_reporter);
            drop(guard);

            if let Some(id) = id {
                let result = BuildResult { status };
                let _ = client.send_response(lsp_server::Response::new_ok(id, result));
            }

            if fwd_search_after {
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
                Ok(()) => ForwardSearchStatus::SUCCESS,
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

    fn handle_file_event(&mut self, event: notify::Event) {
        let mut changed = false;

        let mut workspace = self.workspace.write();
        match event.kind {
            notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                for path in event.paths {
                    if workspace
                        .lookup_path(&path)
                        .map_or(true, |document| document.owner == Owner::Server)
                    {
                        if let Some(language) = Language::from_path(&path) {
                            changed |= workspace.load(&path, language, Owner::Server).is_ok();
                        }
                    }
                }
            }
            notify::EventKind::Remove(_) => {
                for path in event.paths {
                    if let Some(document) = workspace.lookup_path(&path) {
                        if document.owner == Owner::Server {
                            let uri = document.uri.clone();
                            drop(document);
                            workspace.remove(&uri);
                            changed = true;
                        }
                    }
                }
            }
            notify::EventKind::Any | notify::EventKind::Access(_) | notify::EventKind::Other => {}
        };

        drop(workspace);
        if changed {
            self.publish_diagnostics_with_delay();
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
        let position = line_index.offset_lsp(params.text_document_position.position);

        let Some(result) = commands::change_environment(document, position, &params.new_name) else {
            anyhow::bail!("No environment found at the current position");
        };

        let range1 = line_index.line_col_lsp_range(result.begin);
        let range2 = line_index.line_col_lsp_range(result.end);

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
                            self.update_options(options);
                        }
                        InternalMessage::FileEvent(event) => {
                            self.handle_file_event(event);
                        }
                        InternalMessage::Diagnostics => {
                            self.publish_diagnostics()?;
                        }
                        InternalMessage::ChktexResult(uri, diagnostics) => {
                            self.chktex_diagnostics.insert(uri, diagnostics);
                            self.publish_diagnostics()?;
                        }
                        InternalMessage::ForwardSearch(uri, position) => {
                            self.forward_search(None, uri, position)?;
                        }
                    };
                }
            };
        }
    }

    pub fn run(mut self) -> Result<()> {
        self.initialize()?;
        self.process_messages()?;
        self.pool.join();
        Ok(())
    }
}

struct FileWatcher {
    watcher: notify::RecommendedWatcher,
    watched_dirs: FxHashSet<PathBuf>,
}

impl FileWatcher {
    pub fn new(sender: Sender<InternalMessage>) -> Result<Self> {
        let handle = move |event| {
            if let Ok(event) = event {
                sender.send(InternalMessage::FileEvent(event)).unwrap();
            }
        };

        Ok(Self {
            watcher: notify::recommended_watcher(handle)?,
            watched_dirs: FxHashSet::default(),
        })
    }

    pub fn watch(&mut self, workspace: &mut Workspace) {
        workspace.watch(&mut self.watcher, &mut self.watched_dirs);
    }
}
