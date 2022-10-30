use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use log::{error, info, warn};
use lsp_server::{Connection, Message, RequestId};
use lsp_types::{notification::*, request::*, *};
use rowan::{ast::AstNode, TextSize};
use rustc_hash::FxHashSet;
use salsa::ParallelDatabase;
use serde::Serialize;
use threadpool::ThreadPool;

use crate::{
    citation,
    client::LspClient,
    component_db::COMPONENT_DATABASE,
    db::{
        self,
        document::{Language, Owner},
        workspace::Workspace,
        Distro,
    },
    debouncer,
    diagnostics::DiagnosticManager,
    dispatch::{NotificationDispatcher, RequestDispatcher},
    distro::Distribution,
    features::{
        execute_command, find_all_references, find_document_highlights, find_document_links,
        find_document_symbols, find_foldings, find_hover, find_inlay_hints, find_workspace_symbols,
        format_source_code, goto_definition, prepare_rename_all, rename_all, BuildEngine,
        BuildParams, BuildResult, BuildStatus, CompletionItemData, FeatureRequest, ForwardSearch,
        ForwardSearchResult, ForwardSearchStatus,
    },
    normalize_uri,
    syntax::bibtex,
    ClientCapabilitiesExt, Database, Db, Document, DocumentData, Environment, LineIndexExt,
    Options, StartupOptions, WorkspaceEvent,
};

#[derive(Debug)]
enum InternalMessage {
    SetDistro(Distribution),
    SetOptions(Options),
    FileEvent(notify::Event),
}

struct ServerFork {
    connection: Arc<Connection>,
    internal_tx: Sender<InternalMessage>,
    client: LspClient,
    db: salsa::Snapshot<Database>,
    workspace: crate::Workspace,
    diagnostic_tx: debouncer::Sender<crate::Workspace>,
    diagnostic_manager: DiagnosticManager,
    build_engine: Arc<BuildEngine>,
}

impl ServerFork {
    pub fn feature_request<P>(&self, uri: Arc<Url>, params: P) -> FeatureRequest<P> {
        FeatureRequest {
            params,
            workspace: self.workspace.slice(&uri),
            uri,
        }
    }
}

pub struct Server {
    connection: Arc<Connection>,
    internal_tx: Sender<InternalMessage>,
    internal_rx: Receiver<InternalMessage>,
    client: LspClient,
    db: Database,
    watcher: FileWatcher,
    workspace: crate::Workspace,
    diagnostic_tx: debouncer::Sender<crate::Workspace>,
    diagnostic_manager: DiagnosticManager,
    pool: ThreadPool,
    build_engine: Arc<BuildEngine>,
}

impl Server {
    pub fn new(connection: Connection, current_dir: PathBuf) -> Self {
        let client = LspClient::new(connection.sender.clone());
        let (internal_tx, internal_rx) = crossbeam_channel::unbounded();

        let workspace = crate::Workspace::new(Environment::new(Arc::new(current_dir)));
        let diagnostic_manager = DiagnosticManager::default();
        let diagnostic_tx = create_debouncer(client.clone(), diagnostic_manager.clone());

        let db = Database::default();
        let watcher = FileWatcher::new(internal_tx.clone()).expect("init file watcher");

        Self {
            connection: Arc::new(connection),
            internal_tx,
            internal_rx,
            client,
            db,
            watcher,
            workspace,
            diagnostic_tx,
            diagnostic_manager,
            pool: threadpool::Builder::new().build(),
            build_engine: Arc::default(),
        }
    }

    fn spawn(&self, job: impl FnOnce(ServerFork) + Send + 'static) {
        let fork = self.fork();
        self.pool.execute(move || job(fork));
    }

    fn fork(&self) -> ServerFork {
        ServerFork {
            connection: self.connection.clone(),
            internal_tx: self.internal_tx.clone(),
            client: self.client.clone(),
            db: self.db.snapshot(),
            workspace: self.workspace.clone(),
            diagnostic_tx: self.diagnostic_tx.clone(),
            diagnostic_manager: self.diagnostic_manager.clone(),
            build_engine: self.build_engine.clone(),
        }
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

        let workspace = Workspace::get(&self.db);
        workspace
            .set_client_capabilities(&mut self.db)
            .with_durability(salsa::Durability::HIGH)
            .to(params.capabilities);

        workspace
            .set_client_info(&mut self.db)
            .with_durability(salsa::Durability::HIGH)
            .to(params.client_info);

        let result = InitializeResult {
            capabilities: self.capabilities(),
            server_info: Some(ServerInfo {
                name: "TexLab".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
            offset_encoding: None,
        };
        self.connection
            .initialize_finish(id, serde_json::to_value(result)?)?;

        let StartupOptions { skip_distro } =
            serde_json::from_value(params.initialization_options.unwrap_or_default())
                .unwrap_or_default();

        if !skip_distro {
            self.spawn(move |server| {
                let distro = Distribution::detect();
                info!("Detected distribution: {}", distro.kind);

                server
                    .internal_tx
                    .send(InternalMessage::SetDistro(distro))
                    .unwrap();
            });
        }

        self.register_diagnostics_handler();
        self.register_configuration();
        self.pull_options();
        Ok(())
    }

    fn register_configuration(&mut self) {
        let workspace = Workspace::get(&self.db);
        if workspace
            .client_capabilities(&self.db)
            .has_push_configuration_support()
        {
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

    fn register_diagnostics_handler(&mut self) {
        let (event_sender, event_receiver) = crossbeam_channel::unbounded();
        let diagnostic_tx = self.diagnostic_tx.clone();
        let diagnostic_manager = self.diagnostic_manager.clone();
        std::thread::spawn(move || {
            for event in event_receiver {
                match event {
                    WorkspaceEvent::Changed(workspace, document) => {
                        diagnostic_manager.push_syntax(&workspace, document.uri());
                        let delay = workspace.environment.options.diagnostics_delay;
                        diagnostic_tx.send(workspace, delay.0).unwrap();
                    }
                };
            }
        });

        self.workspace.listeners.push(event_sender);
    }

    fn pull_options(&mut self) {
        let workspace = Workspace::get(&self.db);
        if !workspace
            .client_capabilities(&self.db)
            .has_pull_configuration_support()
        {
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
                    error!("Retrieving configuration failed: {}", why);
                }
            };
        });
    }

    fn update_options(&mut self, options: Options) {
        let workspace = Workspace::get(&self.db);
        workspace
            .set_options(&mut self.db)
            .with_durability(salsa::Durability::MEDIUM)
            .to(options);

        self.watcher.watch(&self.db);
    }

    fn cancel(&self, _params: CancelParams) -> Result<()> {
        Ok(())
    }

    fn did_change_watched_files(&mut self, _params: DidChangeWatchedFilesParams) -> Result<()> {
        Ok(())
    }

    fn did_change_configuration(&mut self, params: DidChangeConfigurationParams) -> Result<()> {
        let workspace = Workspace::get(&self.db);
        if workspace
            .client_capabilities(&self.db)
            .has_pull_configuration_support()
        {
            self.pull_options();
        } else {
            let options = self.client.parse_options(params.settings)?;
            self.update_options(options);
        }

        Ok(())
    }

    fn did_open(&mut self, mut params: DidOpenTextDocumentParams) -> Result<()> {
        normalize_uri(&mut params.text_document.uri);

        let workspace = Workspace::get(&self.db);
        let language_id = &params.text_document.language_id;
        let language = Language::from_id(language_id).unwrap_or(Language::Tex);
        let document = workspace.open(
            &mut self.db,
            params.text_document.uri,
            params.text_document.text,
            language,
            Owner::Client,
        );

        workspace.discover(&mut self.db);

        // if self.workspace.environment.options.chktex.on_open_and_save {
        //     self.run_chktex(document);
        // }

        Ok(())
    }

    fn did_change(&mut self, mut params: DidChangeTextDocumentParams) -> Result<()> {
        normalize_uri(&mut params.text_document.uri);

        let workspace = Workspace::get(&self.db);
        let location = db::document::Location::new(&self.db, params.text_document.uri);
        let document = match workspace.lookup(&self.db, location) {
            Some(document) => document,
            None => return Ok(()),
        };

        for change in params.content_changes {
            match change.range {
                Some(range) => {
                    let range = document
                        .contents(&self.db)
                        .line_index(&self.db)
                        .offset_lsp_range(range);

                    document.edit(&mut self.db, range, &change.text);
                }
                None => {
                    document
                        .contents(&self.db)
                        .set_text(&mut self.db)
                        .with_durability(salsa::Durability::LOW)
                        .to(change.text);

                    document
                        .set_cursor(&mut self.db)
                        .with_durability(salsa::Durability::LOW)
                        .to(TextSize::from(0));
                }
            };
        }

        workspace.discover(&mut self.db);

        // TODO: ChkTeX
        // if self.workspace.environment.options.chktex.on_edit {
        //     self.run_chktex(new_document);
        // }

        Ok(())
    }

    fn did_save(&mut self, params: DidSaveTextDocumentParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);

        if let Some(request) = self
            .workspace
            .get(&uri)
            .filter(|_| self.workspace.environment.options.build.on_save)
            .map(|document| {
                self.feature_request(
                    Arc::clone(document.uri()),
                    BuildParams {
                        text_document: TextDocumentIdentifier::new(uri.clone()),
                    },
                )
            })
        {
            self.spawn(move |server| {
                server
                    .build_engine
                    .build(request, server.client)
                    .unwrap_or_else(|why| {
                        error!("Build failed: {}", why);
                        BuildResult {
                            status: BuildStatus::FAILURE,
                        }
                    });
            });
        }

        if let Some(document) = self
            .workspace
            .get(&uri)
            .filter(|_| self.workspace.environment.options.chktex.on_open_and_save)
        {
            self.run_chktex(document);
        }

        Ok(())
    }

    fn did_close(&mut self, params: DidCloseTextDocumentParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);

        let location = db::document::Location::new(&self.db, uri);
        if let Some(document) = Workspace::get(&self.db).lookup(&self.db, location) {
            document
                .set_owner(&mut self.db)
                .with_durability(salsa::Durability::LOW)
                .to(Owner::Server);
        }

        Ok(())
    }

    fn run_chktex(&mut self, document: Document) {
        self.spawn(move |server| {
            server
                .diagnostic_manager
                .push_chktex(&server.workspace, document.uri());

            let delay = server.workspace.environment.options.diagnostics_delay;
            server
                .diagnostic_tx
                .send(server.workspace.clone(), delay.0)
                .unwrap();
        });
    }

    fn feature_request<P>(&self, uri: Arc<Url>, params: P) -> FeatureRequest<P> {
        FeatureRequest {
            params,
            workspace: self.workspace.slice(&uri),
            uri,
        }
    }

    fn handle_feature_request<P, R, H>(
        &self,
        id: RequestId,
        params: P,
        uri: Arc<Url>,
        handler: H,
    ) -> Result<()>
    where
        P: Send + 'static,
        R: Serialize,
        H: FnOnce(FeatureRequest<P>) -> R + Send + 'static,
    {
        self.spawn(move |server| {
            let request = server.feature_request(uri, params);
            if request.workspace.iter().next().is_none() {
                let code = lsp_server::ErrorCode::InvalidRequest as i32;
                let message = "unknown document".to_string();
                let response = lsp_server::Response::new_err(id, code, message);
                server.connection.sender.send(response.into()).unwrap();
            } else {
                let result = handler(request);
                server
                    .connection
                    .sender
                    .send(lsp_server::Response::new_ok(id, result).into())
                    .unwrap();
            }
        });

        Ok(())
    }

    fn document_link(&self, id: RequestId, mut params: DocumentLinkParams) -> Result<()> {
        normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri.clone());
        self.handle_feature_request(id, params, uri, find_document_links)?;
        Ok(())
    }

    fn document_symbols(&self, id: RequestId, mut params: DocumentSymbolParams) -> Result<()> {
        normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri.clone());
        self.handle_feature_request(id, params, uri, find_document_symbols)?;
        Ok(())
    }

    fn workspace_symbols(&self, id: RequestId, params: WorkspaceSymbolParams) -> Result<()> {
        self.spawn(move |server| {
            let result = find_workspace_symbols(&server.workspace, &params);
            server
                .connection
                .sender
                .send(lsp_server::Response::new_ok(id, result).into())
                .unwrap();
        });
        Ok(())
    }

    fn completion(&mut self, id: RequestId, mut params: CompletionParams) -> Result<()> {
        normalize_uri(&mut params.text_document_position.text_document.uri);
        let uri = Arc::new(params.text_document_position.text_document.uri.clone());
        self.handle_feature_request(id, params, uri, crate::features::complete)?;
        Ok(())
    }

    fn completion_resolve(&self, id: RequestId, mut item: CompletionItem) -> Result<()> {
        self.spawn(move |server| {
            match serde_json::from_value(item.data.clone().unwrap()).unwrap() {
                CompletionItemData::Package | CompletionItemData::Class => {
                    item.documentation = COMPONENT_DATABASE
                        .documentation(&item.label)
                        .map(Documentation::MarkupContent);
                }
                CompletionItemData::Citation { uri, key } => {
                    if let Some(root) = server.workspace.get(&uri).and_then(|document| {
                        document
                            .data()
                            .as_bibtex()
                            .map(|data| bibtex::SyntaxNode::new_root(data.green.clone()))
                    }) {
                        item.documentation = bibtex::Root::cast(root)
                            .and_then(|root| root.find_entry(&key))
                            .and_then(|entry| citation::render(&entry))
                            .map(|value| {
                                Documentation::MarkupContent(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value,
                                })
                            });
                    }
                }
                _ => {}
            };

            server
                .connection
                .sender
                .send(lsp_server::Response::new_ok(id, item).into())
                .unwrap();
        });
        Ok(())
    }

    fn folding_range(&self, id: RequestId, mut params: FoldingRangeParams) -> Result<()> {
        normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri.clone());
        self.handle_feature_request(id, params, uri, find_foldings)?;
        Ok(())
    }

    fn references(&self, id: RequestId, mut params: ReferenceParams) -> Result<()> {
        normalize_uri(&mut params.text_document_position.text_document.uri);
        let uri = Arc::new(params.text_document_position.text_document.uri.clone());
        self.handle_feature_request(id, params, uri, find_all_references)?;
        Ok(())
    }

    fn hover(&mut self, id: RequestId, mut params: HoverParams) -> Result<()> {
        normalize_uri(&mut params.text_document_position_params.text_document.uri);
        let uri = Arc::new(
            params
                .text_document_position_params
                .text_document
                .uri
                .clone(),
        );

        let workspace = Workspace::get(&self.db);
        let location = db::document::Location::new(&self.db, uri.as_ref().clone());
        if let Some(document) = workspace.lookup(&self.db, location) {
            let position = document
                .contents(&self.db)
                .line_index(&self.db)
                .offset_lsp(params.text_document_position_params.position);

            document
                .set_cursor(&mut self.db)
                .with_durability(salsa::Durability::LOW)
                .to(position);
        }

        self.handle_feature_request(id, params, uri, find_hover)?;
        Ok(())
    }

    fn goto_definition(&self, id: RequestId, mut params: GotoDefinitionParams) -> Result<()> {
        normalize_uri(&mut params.text_document_position_params.text_document.uri);
        let uri = Arc::new(
            params
                .text_document_position_params
                .text_document
                .uri
                .clone(),
        );
        self.handle_feature_request(id, params, uri, goto_definition)?;
        Ok(())
    }

    fn prepare_rename(&self, id: RequestId, mut params: TextDocumentPositionParams) -> Result<()> {
        normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri.clone());
        self.handle_feature_request(id, params, uri, prepare_rename_all)?;
        Ok(())
    }

    fn rename(&self, id: RequestId, mut params: RenameParams) -> Result<()> {
        normalize_uri(&mut params.text_document_position.text_document.uri);
        let uri = Arc::new(params.text_document_position.text_document.uri.clone());
        self.handle_feature_request(id, params, uri, rename_all)?;
        Ok(())
    }

    fn document_highlight(&self, id: RequestId, mut params: DocumentHighlightParams) -> Result<()> {
        normalize_uri(&mut params.text_document_position_params.text_document.uri);
        let uri = Arc::new(
            params
                .text_document_position_params
                .text_document
                .uri
                .clone(),
        );
        self.handle_feature_request(id, params, uri, find_document_highlights)?;
        Ok(())
    }

    fn formatting(&self, id: RequestId, mut params: DocumentFormattingParams) -> Result<()> {
        normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri.clone());
        self.handle_feature_request(id, params, uri, format_source_code)?;
        Ok(())
    }

    fn execute_command(&self, id: RequestId, params: ExecuteCommandParams) -> Result<()> {
        self.spawn(move |server| {
            let result = execute_command(&server.workspace, &params.command, params.arguments);
            let response = match result {
                Ok(()) => lsp_server::Response::new_ok(id, ()),
                Err(why) => lsp_server::Response::new_err(
                    id,
                    lsp_server::ErrorCode::InternalError as i32,
                    why.to_string(),
                ),
            };

            server.connection.sender.send(response.into()).unwrap();
        });

        Ok(())
    }

    fn inlay_hints(&self, id: RequestId, mut params: InlayHintParams) -> Result<()> {
        normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri.clone());
        self.handle_feature_request(id, params, uri, find_inlay_hints)?;
        Ok(())
    }

    fn inlay_hint_resolve(&self, id: RequestId, hint: InlayHint) -> Result<()> {
        let response = lsp_server::Response::new_ok(id, hint);
        self.connection.sender.send(response.into()).unwrap();
        Ok(())
    }

    fn semantic_tokens_range(
        &self,
        _id: RequestId,
        _params: SemanticTokensRangeParams,
    ) -> Result<()> {
        Ok(())
    }

    fn build(&self, id: RequestId, mut params: BuildParams) -> Result<()> {
        normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri.clone());
        let client = self.client.clone();
        let build_engine = Arc::clone(&self.build_engine);
        self.handle_feature_request(id, params, uri, move |request| {
            build_engine.build(request, client).unwrap_or_else(|why| {
                error!("Build failed: {}", why);
                BuildResult {
                    status: BuildStatus::FAILURE,
                }
            })
        })?;
        Ok(())
    }

    fn forward_search(&self, id: RequestId, mut params: TextDocumentPositionParams) -> Result<()> {
        normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri.clone());
        self.handle_feature_request(id, params, uri, |req| {
            let options = &req.workspace.environment.options.forward_search;
            match options.executable.as_deref().zip(options.args.as_deref()) {
                Some((executable, args)) => ForwardSearch::builder()
                    .executable(executable)
                    .args(args)
                    .line(req.params.position.line)
                    .workspace(&req.workspace)
                    .tex_uri(&req.uri)
                    .build()
                    .execute()
                    .unwrap_or(ForwardSearchResult {
                        status: ForwardSearchStatus::ERROR,
                    }),
                None => ForwardSearchResult {
                    status: ForwardSearchStatus::UNCONFIGURED,
                },
            }
        })?;
        Ok(())
    }

    fn handle_file_event(&mut self, event: notify::Event) {
        let workspace = Workspace::get(&self.db);
        match event.kind {
            notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                for path in event.paths {
                    if workspace
                        .lookup_path(&self.db, &path)
                        .map_or(true, |document| document.owner(&self.db) == Owner::Server)
                    {
                        if let Some(language) = Language::from_path(&path) {
                            workspace.load(&mut self.db, &path, language, Owner::Server);
                        }
                    }
                }
            }
            notify::EventKind::Remove(_) => {
                for path in event.paths {
                    if let Some(document) = workspace.lookup_path(&self.db, &path) {
                        if document.owner(&self.db) == Owner::Server {
                            let mut documents = workspace
                                .set_documents(&mut self.db)
                                .with_durability(salsa::Durability::LOW)
                                .to(FxHashSet::default());

                            documents.remove(&document);
                            workspace
                                .set_documents(&mut self.db)
                                .with_durability(salsa::Durability::MEDIUM)
                                .to(documents);
                        }
                    }
                }
            }
            notify::EventKind::Any | notify::EventKind::Access(_) | notify::EventKind::Other => {}
        };
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

                            if let Some(response) = RequestDispatcher::new(request)
                                .on::<DocumentLinkRequest, _>(|id, params| self.document_link(id, params))?
                                .on::<FoldingRangeRequest, _>(|id, params| self.folding_range(id, params))?
                                .on::<References, _>(|id, params| self.references(id, params))?
                                .on::<HoverRequest, _>(|id, params| self.hover(id, params))?
                                .on::<DocumentSymbolRequest, _>(|id, params| {
                                    self.document_symbols(id, params)
                                })?
                                .on::<WorkspaceSymbol, _>(|id, params| self.workspace_symbols(id, params))?
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
                                .on::<BuildRequest, _>(|id, params| self.build(id, params))?
                                .on::<ForwardSearchRequest, _>(|id, params| {
                                    self.forward_search(id, params)
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
                                .default()
                            {
                                self.connection.sender.send(response.into())?;
                            }
                        }
                        Message::Notification(notification) => {
                            NotificationDispatcher::new(notification)
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
                            Distro::get(&self.db)
                                .set_file_name_db(&mut self.db)
                                .with_durability(salsa::Durability::HIGH)
                                .to(distro.resolver);
                        }
                        InternalMessage::SetOptions(options) => {
                            self.update_options(options);
                        }
                        InternalMessage::FileEvent(event) => {
                            self.handle_file_event(event);
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

fn create_debouncer(
    client: LspClient,
    diagnostic_manager: DiagnosticManager,
) -> debouncer::Sender<crate::Workspace> {
    let (tx, rx) = debouncer::unbounded();
    std::thread::spawn(move || {
        while let Ok(workspace) = rx.recv() {
            if let Err(why) = publish_diagnostics(&client, &diagnostic_manager, &workspace) {
                warn!("Failed to publish diagnostics: {}", why);
            }
        }
    });

    tx
}

fn publish_diagnostics(
    client: &LspClient,
    diagnostic_manager: &DiagnosticManager,
    workspace: &crate::Workspace,
) -> Result<()> {
    for document in workspace.iter() {
        if matches!(document.data(), DocumentData::BuildLog(_)) {
            continue;
        }

        let diagnostics = diagnostic_manager.publish(workspace, document.uri());
        client.send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
            uri: document.uri().as_ref().clone(),
            version: None,
            diagnostics,
        })?;
    }

    Ok(())
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

    pub fn watch(&mut self, db: &dyn Db) {
        let workspace = Workspace::get(db);
        workspace.watch(db, &mut self.watcher, &mut self.watched_dirs);
    }
}

struct BuildRequest;

impl lsp_types::request::Request for BuildRequest {
    type Params = BuildParams;

    type Result = BuildResult;

    const METHOD: &'static str = "textDocument/build";
}

struct ForwardSearchRequest;

impl lsp_types::request::Request for ForwardSearchRequest {
    type Params = TextDocumentPositionParams;

    type Result = ForwardSearchResult;

    const METHOD: &'static str = "textDocument/forwardSearch";
}
