mod dispatch;
mod query;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use log::{error, info};
use lsp_server::{Connection, ErrorCode, Message, RequestId};
use lsp_types::{notification::*, request::*, *};
use once_cell::sync::Lazy;
use rowan::{ast::AstNode, TextSize};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use threadpool::ThreadPool;

use crate::{
    citation,
    client::LspClient,
    db::{self, discover_dependencies, Document, Language, Owner, Workspace},
    distro::Distro,
    features::{
        build::{self, BuildParams, BuildResult, BuildStatus},
        completion::{self, builder::CompletionItemData},
        definition, folding, formatting, forward_search, highlight, hover, inlay_hint, link,
        reference, rename, symbol, workspace_command,
    },
    normalize_uri,
    syntax::bibtex,
    util::{
        self, capabilities::ClientCapabilitiesExt, components::COMPONENT_DATABASE,
        line_index_ext::LineIndexExt,
    },
    Db, Options, StartupOptions,
};

#[derive(Debug)]
enum InternalMessage {
    SetDistro(Distro),
    SetOptions(Options),
    FileEvent(notify::Event),
    ForwardSearch(Url),
    Diagnostics,
    ChktexResult(Url, Vec<db::diagnostics::Diagnostic>),
}

pub struct Server {
    connection: Arc<Connection>,
    internal_tx: Sender<InternalMessage>,
    internal_rx: Receiver<InternalMessage>,
    client: LspClient,
    engine: query::Engine,
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
            client,
            engine: query::Engine::default(),
            watcher,
            pool: threadpool::Builder::new().build(),
        }
    }

    fn run_with_db<R, Q>(&self, id: RequestId, query: Q)
    where
        R: Serialize,
        Q: FnOnce(&dyn Db) -> R + Send + 'static,
    {
        let client = self.client.clone();
        self.engine.fork(move |db| {
            let response = lsp_server::Response::new_ok(id, query(db));
            client.send_response(response).unwrap();
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

        let db = self.engine.write();
        let workspace = Workspace::get(db);
        workspace
            .set_client_capabilities(db)
            .with_durability(salsa::Durability::HIGH)
            .to(params.capabilities);

        workspace
            .set_client_info(db)
            .with_durability(salsa::Durability::HIGH)
            .to(params.client_info);

        let root_dirs = params
            .workspace_folders
            .unwrap_or_default()
            .into_iter()
            .map(|folder| db::Location::new(db, folder.uri))
            .collect();

        workspace
            .set_root_dirs(db)
            .with_durability(salsa::Durability::HIGH)
            .to(root_dirs);

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
            let sender = self.internal_tx.clone();
            self.pool.execute(move || {
                let distro = Distro::detect().unwrap_or_else(|why| {
                    log::warn!("Unable to load distro files: {}", why);
                    Distro::default()
                });

                info!("Detected distribution: {:?}", distro.kind);
                sender.send(InternalMessage::SetDistro(distro)).unwrap();
            });
        }

        self.register_configuration();
        self.pull_options();
        Ok(())
    }

    fn register_configuration(&mut self) {
        let db = self.engine.read();

        if Workspace::get(db)
            .client_capabilities(db)
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

    fn update_workspace(&mut self) {
        let db = self.engine.write();
        discover_dependencies(db);
        self.watcher.watch(db);
        self.publish_diagnostics_with_delay();
    }

    fn publish_diagnostics(&mut self) -> Result<()> {
        let db = self.engine.read();

        let all_diagnostics = db::diagnostics::collect_filtered(db, Workspace::get(db));

        for (document, diagnostics) in all_diagnostics {
            let uri = document.location(db).uri(db).clone();
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
        let db = self.engine.read();
        let sender = self.internal_tx.clone();
        let delay = Workspace::get(db).options(db).diagnostics_delay.0;
        self.pool.execute(move || {
            std::thread::sleep(delay);
            sender.send(InternalMessage::Diagnostics).unwrap();
        });
    }

    fn pull_options(&mut self) {
        let db = self.engine.read();
        let workspace = Workspace::get(db);
        if !workspace
            .client_capabilities(db)
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
        let db = self.engine.write();
        let workspace = Workspace::get(db);
        workspace
            .set_options(db)
            .with_durability(salsa::Durability::MEDIUM)
            .to(options);

        self.watcher.watch(db);
    }

    fn cancel(&self, _params: CancelParams) -> Result<()> {
        Ok(())
    }

    fn did_change_watched_files(&mut self, _params: DidChangeWatchedFilesParams) -> Result<()> {
        Ok(())
    }

    fn did_change_configuration(&mut self, params: DidChangeConfigurationParams) -> Result<()> {
        let db = self.engine.read();
        let workspace = Workspace::get(db);
        if workspace
            .client_capabilities(db)
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

        let db = self.engine.write();
        let workspace = Workspace::get(db);
        let language_id = &params.text_document.language_id;
        let language = Language::from_id(language_id).unwrap_or(Language::Tex);
        let document = workspace.open(
            db,
            params.text_document.uri,
            params.text_document.text,
            language,
            Owner::Client,
        );

        self.update_workspace();

        if workspace
            .options(self.engine.read())
            .chktex
            .on_open_and_save
        {
            self.run_chktex(document);
        }

        Ok(())
    }

    fn did_change(&mut self, params: DidChangeTextDocumentParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);

        let db = self.engine.write();
        let workspace = Workspace::get(db);
        let document = match workspace.lookup_uri(db, &uri) {
            Some(document) => document,
            None => return Ok(()),
        };

        for change in params.content_changes {
            match change.range {
                Some(range) => {
                    let range = document.contents(db).line_index(db).offset_lsp_range(range);
                    document.edit(db, range, &change.text);
                }
                None => {
                    document
                        .contents(db)
                        .set_text(db)
                        .with_durability(salsa::Durability::LOW)
                        .to(change.text);

                    document
                        .set_cursor(db)
                        .with_durability(salsa::Durability::LOW)
                        .to(TextSize::from(0));
                }
            };
        }

        self.update_workspace();

        if workspace.options(self.engine.read()).chktex.on_edit {
            self.run_chktex(document);
        }

        Ok(())
    }

    fn did_save(&mut self, params: DidSaveTextDocumentParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);

        let db = self.engine.read();
        let workspace = Workspace::get(db);
        if workspace.options(db).build.on_save {
            self.build_internal(uri.clone(), |_| ())?;
        }

        self.publish_diagnostics_with_delay();

        let db = self.engine.read();
        if let Some(document) = workspace.lookup_uri(db, &uri) {
            if workspace.options(db).chktex.on_open_and_save {
                self.run_chktex(document);
            }
        }

        Ok(())
    }

    fn did_close(&mut self, params: DidCloseTextDocumentParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);

        let db = self.engine.write();
        if let Some(document) = Workspace::get(db).lookup_uri(db, &uri) {
            document
                .set_owner(db)
                .with_durability(salsa::Durability::LOW)
                .to(Owner::Server);
        }

        self.publish_diagnostics_with_delay();
        Ok(())
    }

    fn run_chktex(&mut self, document: Document) {
        let db = self.engine.read();
        if let Some(command) = util::chktex::Command::new(db, document) {
            let sender = self.internal_tx.clone();
            let uri = document.location(db).uri(db).clone();
            self.pool.execute(move || {
                let diagnostics = command.run().unwrap_or_default();
                sender
                    .send(InternalMessage::ChktexResult(uri, diagnostics))
                    .unwrap();
            });
        }
    }

    fn document_link(&self, id: RequestId, params: DocumentLinkParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);
        self.run_with_db(id, move |db| link::find_all(db, &uri).unwrap_or_default());
        Ok(())
    }

    fn document_symbols(&self, id: RequestId, params: DocumentSymbolParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);
        self.run_with_db(id, move |db| symbol::find_document_symbols(db, &uri));
        Ok(())
    }

    fn workspace_symbols(&self, id: RequestId, params: WorkspaceSymbolParams) -> Result<()> {
        self.run_with_db(id, move |db| symbol::find_workspace_symbols(db, &params));
        Ok(())
    }

    fn completion(&mut self, id: RequestId, params: CompletionParams) -> Result<()> {
        let mut uri = params.text_document_position.text_document.uri;
        normalize_uri(&mut uri);
        let position = params.text_document_position.position;
        self.run_with_db(id, move |db| completion::complete(db, &uri, position));
        Ok(())
    }

    fn completion_resolve(&self, id: RequestId, mut item: CompletionItem) -> Result<()> {
        self.run_with_db(id, move |db| {
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
                    if let Some(root) = Workspace::get(db)
                        .lookup_uri(db, &uri)
                        .and_then(|document| document.parse(db).as_bib().map(|data| data.root(db)))
                    {
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
                None => {}
            };

            item
        });

        Ok(())
    }

    fn folding_range(&self, id: RequestId, params: FoldingRangeParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);
        self.run_with_db(id, move |db| {
            folding::find_all(db, &uri).unwrap_or_default()
        });
        Ok(())
    }

    fn references(&self, id: RequestId, params: ReferenceParams) -> Result<()> {
        let mut uri = params.text_document_position.text_document.uri;
        normalize_uri(&mut uri);
        let position = params.text_document_position.position;
        self.run_with_db(id, move |db| {
            reference::find_all(db, &uri, position, &params.context).unwrap_or_default()
        });

        Ok(())
    }

    fn hover(&mut self, id: RequestId, params: HoverParams) -> Result<()> {
        let mut uri = params.text_document_position_params.text_document.uri;
        normalize_uri(&mut uri);

        let db = self.engine.write();
        let workspace = Workspace::get(db);
        if let Some(document) = workspace.lookup_uri(db, &uri) {
            let position = document
                .contents(db)
                .line_index(db)
                .offset_lsp(params.text_document_position_params.position);

            document
                .set_cursor(db)
                .with_durability(salsa::Durability::LOW)
                .to(position);
        }

        let position = params.text_document_position_params.position;
        self.run_with_db(id, move |db| hover::find(db, &uri, position));
        Ok(())
    }

    fn goto_definition(&self, id: RequestId, params: GotoDefinitionParams) -> Result<()> {
        let mut uri = params.text_document_position_params.text_document.uri;
        normalize_uri(&mut uri);
        let position = params.text_document_position_params.position;
        self.run_with_db(id, move |db| {
            definition::goto_definition(db, &uri, position)
        });

        Ok(())
    }

    fn prepare_rename(&self, id: RequestId, params: TextDocumentPositionParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);
        self.run_with_db(id, move |db| {
            rename::prepare_rename_all(db, &uri, params.position)
        });

        Ok(())
    }

    fn rename(&self, id: RequestId, params: RenameParams) -> Result<()> {
        let mut uri = params.text_document_position.text_document.uri;
        normalize_uri(&mut uri);
        let position = params.text_document_position.position;
        self.run_with_db(id, move |db| {
            rename::rename_all(db, &uri, position, params.new_name)
        });

        Ok(())
    }

    fn document_highlight(&self, id: RequestId, params: DocumentHighlightParams) -> Result<()> {
        let mut uri = params.text_document_position_params.text_document.uri;
        normalize_uri(&mut uri);
        let position = params.text_document_position_params.position;
        self.run_with_db(id, move |db| {
            highlight::find_all(db, &uri, position).unwrap_or_default()
        });
        Ok(())
    }

    fn formatting(&self, id: RequestId, params: DocumentFormattingParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);
        self.run_with_db(id, move |db| {
            formatting::format_source_code(db, &uri, &params.options)
        });

        Ok(())
    }

    fn execute_command(&mut self, id: RequestId, params: ExecuteCommandParams) -> Result<()> {
        let db = self.engine.read();
        match workspace_command::select(db, &params.command, params.arguments) {
            Ok(command) => {
                let client = self.client.clone();
                self.pool.execute(move || {
                    match command.run() {
                        Ok(()) => {
                            client
                                .send_response(lsp_server::Response::new_ok(id, ()))
                                .unwrap();
                        }
                        Err(why) => {
                            client
                                .send_error(id, ErrorCode::InternalError, why.to_string())
                                .unwrap();
                        }
                    };
                });
            }
            Err(why) => {
                self.client
                    .send_error(id, ErrorCode::InvalidParams, why.to_string())
                    .unwrap();
            }
        };

        Ok(())
    }

    fn inlay_hints(&self, id: RequestId, params: InlayHintParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);
        self.run_with_db(id, move |db| {
            inlay_hint::find_all(db, &uri, params.range).unwrap_or_default()
        });
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

    fn build(&mut self, id: RequestId, params: BuildParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);

        let client = self.client.clone();
        self.build_internal(uri, move |status| {
            let result = BuildResult { status };
            client
                .send_response(lsp_server::Response::new_ok(id, result))
                .unwrap();
        })?;

        Ok(())
    }

    fn build_internal(
        &mut self,
        uri: Url,
        callback: impl FnOnce(BuildStatus) + Send + 'static,
    ) -> Result<()> {
        static LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

        let db = self.engine.read();
        let compiler = match build::Command::new(db, uri.clone(), self.client.clone()) {
            Some(compiler) => compiler,
            None => {
                callback(BuildStatus::FAILURE);
                return Ok(());
            }
        };

        let forward_search_after = Workspace::get(db).options(db).build.forward_search_after;

        let sender = self.internal_tx.clone();
        self.pool.execute(move || {
            let guard = LOCK.lock().unwrap();

            let status = compiler.run();
            if forward_search_after {
                sender.send(InternalMessage::ForwardSearch(uri)).unwrap();
            }

            drop(guard);
            callback(status);
        });

        Ok(())
    }

    fn forward_search(&mut self, id: RequestId, params: TextDocumentPositionParams) -> Result<()> {
        let mut uri = params.text_document.uri;
        normalize_uri(&mut uri);

        let client = self.client.clone();
        self.forward_search_internal(uri, Some(params.position), move |status| {
            let result = ForwardSearchResult { status };
            client
                .send_response(lsp_server::Response::new_ok(id, result))
                .unwrap();
        })?;

        Ok(())
    }

    fn forward_search_internal(
        &mut self,
        uri: Url,
        position: Option<Position>,
        callback: impl FnOnce(ForwardSearchStatus) + Send + 'static,
    ) -> Result<()> {
        let db = self.engine.read();
        let command = match forward_search::Command::configure(db, &uri, position) {
            Ok(command) => command,
            Err(why) => {
                log::error!("Forward search failed: {}", why);
                callback(why.into());
                return Ok(());
            }
        };

        self.pool.execute(move || {
            let status = command
                .run()
                .map_or_else(ForwardSearchStatus::from, |()| ForwardSearchStatus::SUCCESS);

            callback(status);
        });

        Ok(())
    }

    fn handle_file_event(&mut self, event: notify::Event) {
        let mut changed = false;

        let db = self.engine.write();
        let workspace = Workspace::get(db);
        match event.kind {
            notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                for path in event.paths {
                    if workspace
                        .lookup_path(db, &path)
                        .map_or(true, |document| document.owner(db) == Owner::Server)
                    {
                        if let Some(language) = Language::from_path(&path) {
                            workspace.load(db, &path, language, Owner::Server);
                            changed = true;
                        }
                    }
                }
            }
            notify::EventKind::Remove(_) => {
                for path in event.paths {
                    if let Some(document) = workspace.lookup_path(db, &path) {
                        if document.owner(db) == Owner::Server {
                            let mut documents = workspace
                                .set_documents(db)
                                .with_durability(salsa::Durability::LOW)
                                .to(FxHashSet::default());

                            documents.remove(&document);
                            workspace
                                .set_documents(db)
                                .with_durability(salsa::Durability::MEDIUM)
                                .to(documents);

                            changed = true;
                        }
                    }
                }
            }
            notify::EventKind::Any | notify::EventKind::Access(_) | notify::EventKind::Other => {}
        };

        if changed {
            self.publish_diagnostics_with_delay();
        }
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
                            let db = self.engine.write();
                            Workspace::get(db)
                                .set_file_name_db(db)
                                .with_durability(salsa::Durability::HIGH)
                                .to(distro.file_name_db);
                        }
                        InternalMessage::SetOptions(options) => {
                            self.update_options(options);
                        }
                        InternalMessage::FileEvent(event) => {
                            self.handle_file_event(event);
                        }
                        InternalMessage::ForwardSearch(uri) => {
                            self.forward_search_internal(uri, None, |_| ())?;
                        }
                        InternalMessage::Diagnostics => {
                            self.publish_diagnostics()?;
                        }
                        InternalMessage::ChktexResult(uri, diagnostics) => {
                            let db = self.engine.write();
                            let workspace = Workspace::get(db);
                            if let Some(document) = workspace.lookup_uri(db, &uri) {
                                document.linter(db).set_chktex(db).to(diagnostics);
                            }

                            self.publish_diagnostics()?;
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
        self.engine.finish();
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum ForwardSearchStatus {
    SUCCESS = 0,
    ERROR = 1,
    FAILURE = 2,
    UNCONFIGURED = 3,
}

impl From<forward_search::Error> for ForwardSearchStatus {
    fn from(err: forward_search::Error) -> Self {
        match err {
            forward_search::Error::TexNotFound(_) => ForwardSearchStatus::FAILURE,
            forward_search::Error::PdfNotFound(_) => ForwardSearchStatus::ERROR,
            forward_search::Error::NoLocalFile(_) => ForwardSearchStatus::FAILURE,
            forward_search::Error::Unconfigured => ForwardSearchStatus::UNCONFIGURED,
            forward_search::Error::Spawn(_) => ForwardSearchStatus::ERROR,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ForwardSearchResult {
    pub status: ForwardSearchStatus,
}
