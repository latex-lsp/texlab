use crate::lsp::*;
use futures::prelude::*;
use lsp_types::*;

pub struct LatexLspServer;

impl LspServer for LatexLspServer {
    fn initialize(&self, params: InitializeParams) -> LspFuture<InitializeResult> {
        Box::pin(async { Ok(InitializeResult::default()) })
    }

    fn initialized(&self, params: InitializedParams) {}

    fn shutdown(&self, params: ()) -> LspFuture<()> {
        Box::pin(async { Ok(()) })
    }

    fn exit(&self, params: ()) {}

    fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {}

    fn did_open(&self, params: DidOpenTextDocumentParams) {}

    fn did_change(&self, params: DidChangeTextDocumentParams) {}

    fn did_save(&self, params: DidSaveTextDocumentParams) {}

    fn did_close(&self, params: DidCloseTextDocumentParams) {}

    fn completion(&self, params: CompletionParams) -> LspFuture<CompletionList> {
        Box::pin(async { Ok(CompletionList::default()) })
    }

    fn completion_resolve(&self, item: CompletionItem) -> LspFuture<CompletionItem> {
        Box::pin(async { Ok(item) })
    }

    fn hover(&self, params: TextDocumentPositionParams) -> LspFuture<Option<Hover>> {
        Box::pin(async { Ok(None) })
    }

    fn definition(&self, params: TextDocumentPositionParams) -> LspFuture<Vec<Location>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn references(&self, params: ReferenceParams) -> LspFuture<Vec<Location>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn document_highlight(
        &self,
        params: TextDocumentPositionParams,
    ) -> LspFuture<Vec<DocumentHighlight>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn document_symbol(&self, params: DocumentSymbolParams) -> LspFuture<Vec<DocumentSymbol>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn document_link(&self, params: DocumentLinkParams) -> LspFuture<Vec<DocumentLink>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn formatting(&self, params: DocumentFormattingParams) -> LspFuture<Vec<TextEdit>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn rename(&self, params: RenameParams) -> LspFuture<Option<WorkspaceEdit>> {
        Box::pin(async { Ok(None) })
    }

    fn folding_range(&self, params: FoldingRangeParams) -> LspFuture<Vec<FoldingRange>> {
        Box::pin(async { Ok(Vec::new()) })
    }
}
