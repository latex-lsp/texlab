use crate::client::LspClientMock;
use crate::server::LatexLspServer;
use crate::workspace::*;
use copy_dir::copy_dir;
use lsp_types::*;
use std::fs;
use std::fs::remove_dir;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

pub static DEFAULT_CAPABILITIES: ClientCapabilities = ClientCapabilities {
    workspace: Some(WorkspaceClientCapabilities {
        configuration: Some(true),
        did_change_watched_files: None,
        workspace_folders: None,
        apply_edit: None,
        execute_command: None,
        symbol: None,
        workspace_edit: None,
        did_change_configuration: None,
    }),
    text_document: Some(TextDocumentClientCapabilities {
        synchronization: None,
        completion: None,
        hover: None,
        signature_help: None,
        references: None,
        document_highlight: None,
        document_symbol: Some(DocumentSymbolCapability {
            dynamic_registration: None,
            hierarchical_document_symbol_support: Some(true),
            symbol_kind: None,
        }),
        formatting: None,
        range_formatting: None,
        on_type_formatting: None,
        declaration: None,
        definition: Some(GotoCapability {
            dynamic_registration: None,
            link_support: Some(true),
        }),
        type_definition: None,
        implementation: None,
        code_action: None,
        code_lens: None,
        document_link: None,
        color_provider: None,
        rename: None,
        publish_diagnostics: None,
        folding_range: None,
    }),
    experimental: None,
    window: Some(WindowClientCapabilities {
        work_done_progress: Some(true),
    }),
};

pub static NO_LINK_SUPPORT_CAPABILITIES: ClientCapabilities = ClientCapabilities {
    workspace: Some(WorkspaceClientCapabilities {
        configuration: Some(true),
        did_change_watched_files: None,
        workspace_folders: None,
        apply_edit: None,
        execute_command: None,
        symbol: None,
        workspace_edit: None,
        did_change_configuration: None,
    }),
    text_document: Some(TextDocumentClientCapabilities {
        synchronization: None,
        completion: None,
        hover: None,
        signature_help: None,
        references: None,
        document_highlight: None,
        document_symbol: None,
        formatting: None,
        range_formatting: None,
        on_type_formatting: None,
        declaration: None,
        definition: Some(GotoCapability {
            dynamic_registration: None,
            link_support: Some(false),
        }),
        type_definition: None,
        implementation: None,
        code_action: None,
        code_lens: None,
        document_link: None,
        color_provider: None,
        rename: None,
        publish_diagnostics: None,
        folding_range: None,
    }),
    experimental: None,
    window: Some(WindowClientCapabilities {
        work_done_progress: Some(true),
    }),
};

pub struct TestScenarioParams<'a> {
    pub name: &'a str,
    pub client_capabilities: &'a ClientCapabilities,
    pub distribution: Box<dyn tex::Distribution>,
}

impl<'a> TestScenarioParams<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            client_capabilities: &DEFAULT_CAPABILITIES,
            distribution: Box::new(tex::Unknown::default()),
        }
    }
}

pub struct TestScenario {
    pub server: LatexLspServer<LspClientMock>,
    pub client: Arc<LspClientMock>,
    pub directory: TempDir,
}

impl TestScenario {
    pub async fn new<'a>(params: TestScenarioParams<'a>) -> Self {
        let directory = tempfile::tempdir().unwrap();
        remove_dir(directory.path()).unwrap();
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("scenarios")
            .join(params.name);
        copy_dir(source, directory.path()).unwrap();

        let client = Arc::new(LspClientMock::default());
        let server = LatexLspServer::new(Arc::new(params.distribution), Arc::clone(&client));

        let root_uri = Uri::from_file_path(directory.path()).unwrap();
        let init_params = InitializeParams {
            process_id: None,
            root_path: Some(directory.path().to_string_lossy().into_owned()),
            root_uri: Some(root_uri.into()),
            initialization_options: None,
            capabilities: params.client_capabilities.to_owned(),
            trace: None,
            workspace_folders: None,
        };

        server
            .execute_async(|svr| svr.initialize(init_params))
            .await
            .unwrap();
        server
            .execute(|svr| svr.initialized(InitializedParams {}))
            .await;
        Self {
            server,
            client,
            directory,
        }
    }

    pub fn uri(&self, name: &str) -> Uri {
        let mut path = self.directory.path().to_owned();
        path.push(name);
        Uri::from_file_path(path).unwrap()
    }

    pub async fn read(&self, name: &'static str) -> String {
        let mut path = self.directory.path().to_owned();
        path.push(name);
        fs::read_to_string(path).unwrap().replace('\r', "")
    }

    pub async fn open(&self, name: &'static str) {
        let text = self.read(name).await;
        let language_id = if name.ends_with(".bib") {
            "bibtex"
        } else {
            "latex"
        };

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: self.uri(name).into(),
                version: 0,
                language_id: language_id.to_owned(),
                text,
            },
        };
        self.server.execute(|svr| svr.did_open(params)).await;
    }
}
