use insta::assert_debug_snapshot;
use lsp_types::{notification::DidOpenTextDocument, request::DocumentSymbolRequest, *};

use crate::fixture::TestBed;

#[test]
fn test_smoke() {
    let test_bed = TestBed::new("").unwrap();
    test_bed
        .initialize(ClientCapabilities {
            text_document: Some(TextDocumentClientCapabilities {
                document_symbol: Some(DocumentSymbolClientCapabilities {
                    hierarchical_document_symbol_support: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();

    let uri = Url::parse("file:///texlab/main.tex").unwrap();
    test_bed
        .client()
        .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
            text_document: TextDocumentItem::new(
                uri.clone(),
                "latex".into(),
                0,
                r#"\section{Foo} \subsection{Bar} \section{Baz}"#.into(),
            ),
        })
        .unwrap();

    let Some(DocumentSymbolResponse::Nested(symbols)) = test_bed
        .client()
        .send_request::<DocumentSymbolRequest>(DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            partial_result_params: Default::default(),
            work_done_progress_params: Default::default(),
        })
        .unwrap() else { unreachable!() };

    assert_debug_snapshot!(symbols);
}
