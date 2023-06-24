use base_db::DocumentData;
use lsp_types::ReferenceContext;

use crate::util::cursor::CursorContext;

use super::ReferenceResult;

pub(super) fn find_all_references<'a>(
    context: &CursorContext<'a, &ReferenceContext>,
    results: &mut Vec<ReferenceResult<'a>>,
) -> Option<()> {
    let (key, _) = context
        .find_citation_key_word()
        .or_else(|| context.find_citation_key_command())
        .or_else(|| context.find_entry_key())?;

    for document in &context.project.documents {
        if let DocumentData::Tex(data) = &document.data {
            for citation in data
                .semantics
                .citations
                .iter()
                .filter(|citation| citation.name.text == key)
            {
                results.push(ReferenceResult {
                    document,
                    range: citation.name.range,
                });
            }
        } else if let DocumentData::Bib(data) = &document.data {
            if context.params.include_declaration {
                for entry in data
                    .semantics
                    .entries
                    .iter()
                    .filter(|entry| entry.name.text == key)
                {
                    results.push(ReferenceResult {
                        document,
                        range: entry.name.range,
                    });
                }
            }
        }
    }

    Some(())
}
