use futures_boxed::boxed;
use jsonrpc::client::Result;
use jsonrpc_derive::{jsonrpc_client, jsonrpc_method};
use texlab_protocol::*;

#[jsonrpc_client(TestLatexLspClient)]
pub trait TestLspClient {
    #[jsonrpc_method("initialize", kind = "request")]
    #[boxed]
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult>;

    #[jsonrpc_method("initialized", kind = "notification")]
    #[boxed]
    async fn initialized(&self, params: InitializedParams);

    #[jsonrpc_method("shutdown", kind = "request")]
    #[boxed]
    async fn shutdown(&self, params: ()) -> Result<()>;

    #[jsonrpc_method("exit", kind = "notification")]
    #[boxed]
    async fn exit(&self, params: ());

    #[jsonrpc_method("textDocument/didOpen", kind = "notification")]
    #[boxed]
    async fn did_open(&self, params: DidOpenTextDocumentParams);

    #[jsonrpc_method("textDocument/didChange", kind = "notification")]
    #[boxed]
    async fn did_change(&self, params: DidChangeTextDocumentParams);

    #[jsonrpc_method("workspace/didChangeConfiguration", kind = "notification")]
    #[boxed]
    async fn did_change_configuration(&self, params: DidChangeConfigurationParams);

    #[jsonrpc_method("textDocument/definition", kind = "request")]
    #[boxed]
    async fn definition(&self, params: TextDocumentPositionParams) -> Result<DefinitionResponse>;

    #[jsonrpc_method("textDocument/completion", kind = "request")]
    #[boxed]
    async fn completion(&self, params: CompletionParams) -> Result<CompletionList>;

    #[jsonrpc_method("completionItem/resolve", kind = "request")]
    #[boxed]
    async fn completion_resolve(&self, item: CompletionItem) -> Result<CompletionItem>;

    #[jsonrpc_method("textDocument/foldingRange", kind = "request")]
    #[boxed]
    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Vec<FoldingRange>>;

    #[jsonrpc_method("textDocument/documentHighlight", kind = "request")]
    #[boxed]
    async fn document_highlight(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Vec<DocumentHighlight>>;

    #[jsonrpc_method("textDocument/documentLink", kind = "request")]
    #[boxed]
    async fn document_link(&self, params: DocumentLinkParams) -> Result<Vec<DocumentLink>>;

    #[jsonrpc_method("textDocument/references", kind = "request")]
    #[boxed]
    async fn references(&self, params: ReferenceParams) -> Result<Vec<Location>>;

    #[jsonrpc_method("textDocument/prepareRename", kind = "request")]
    #[boxed]
    async fn prepare_rename(&self, params: TextDocumentPositionParams) -> Result<Option<Range>>;

    #[jsonrpc_method("textDocument/rename", kind = "request")]
    #[boxed]
    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>>;

    #[jsonrpc_method("textDocument/hover", kind = "request")]
    #[boxed]
    async fn hover(&self, params: TextDocumentPositionParams) -> Result<Option<Hover>>;

    #[jsonrpc_method("workspace/symbol", kind = "request")]
    #[boxed]
    async fn workspace_symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Vec<SymbolInformation>>;

    #[jsonrpc_method("textDocument/documentSymbol", kind = "request")]
    #[boxed]
    async fn document_symbol(&self, params: DocumentSymbolParams)
        -> Result<DocumentSymbolResponse>;

    #[jsonrpc_method("$/detectRoot", kind = "request")]
    #[boxed]
    async fn detect_root(&self, params: TextDocumentIdentifier) -> Result<()>;
}
