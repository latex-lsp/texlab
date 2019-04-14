use crate::lsp::server::*;
use futures::prelude::*;
use lsp_types::*;

pub struct LatexLspServer;

impl LspServer for LatexLspServer {
    fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        Box::new(futures::finished(InitializeResult::default()))
    }

    fn initialized(&self, params: InitializedParams) {}

    fn shutdown(&self, params: ()) -> LspResult<()> {
        Box::new(futures::finished(()))
    }

    fn exit(&self, params: ()) {}

    fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {}

    fn did_open(&self, params: DidOpenTextDocumentParams) {}

    fn did_change(&self, params: DidChangeTextDocumentParams) {}

    fn did_save(&self, params: DidSaveTextDocumentParams) {}

    fn did_close(&self, params: DidCloseTextDocumentParams) {}

    fn completion(&self, params: CompletionParams) -> LspResult<CompletionList> {
        Box::new(futures::finished(CompletionList::default()))
    }

    fn completion_resolve(&self, item: CompletionItem) -> LspResult<CompletionItem> {
        Box::new(futures::finished(item))
    }

    fn hover(&self, params: TextDocumentPositionParams) -> LspResult<Option<Hover>> {
        Box::new(futures::finished(None))
    }

    fn definition(&self, params: TextDocumentPositionParams) -> LspResult<Vec<Location>> {
        Box::new(futures::finished(Vec::new()))
    }

    fn references(&self, params: ReferenceParams) -> LspResult<Vec<Location>> {
        Box::new(futures::finished(Vec::new()))
    }

    fn document_highlight(
        &self,
        params: TextDocumentPositionParams,
    ) -> LspResult<Vec<DocumentHighlight>> {
        Box::new(futures::finished(Vec::new()))
    }

    fn document_symbol(&self, params: DocumentSymbolParams) -> LspResult<Vec<DocumentSymbol>> {
        Box::new(futures::finished(Vec::new()))
    }

    fn document_link(&self, params: DocumentLinkParams) -> LspResult<Vec<DocumentLink>> {
        Box::new(futures::finished(Vec::new()))
    }

    fn formatting(&self, params: DocumentFormattingParams) -> LspResult<Vec<TextEdit>> {
        Box::new(futures::finished(Vec::new()))
    }

    fn rename(&self, params: RenameParams) -> LspResult<Option<WorkspaceEdit>> {
        Box::new(futures::finished(None))
    }

    fn folding_range(&self, params: FoldingRangeParams) -> LspResult<Vec<FoldingRange>> {
        Box::new(futures::finished(Vec::new()))
    }
}
