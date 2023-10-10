use base_db::{semantics::Span, DocumentData};
use rowan::{TextRange, TextSize};
use syntax::latex;

use crate::{RenameBuilder, RenameParams};

pub(super) fn prepare_rename(params: &RenameParams) -> Option<Span> {
    let data = params.inner.document.data.as_tex()?;
    let token = data
        .root_node()
        .token_at_offset(params.offset)
        .find(|token| token.kind() == latex::COMMAND_NAME)?;

    let range = token.text_range();
    let text = token.text()[1..].into();
    Some(Span::new(
        text,
        TextRange::new(range.start() + TextSize::of('\\'), range.end()),
    ))
}

pub(super) fn rename<'a>(builder: &mut RenameBuilder) -> Option<()> {
    let name = prepare_rename(&builder.params)?;

    for document in &builder.params.inner.project.documents {
        let DocumentData::Tex(data) = &document.data else {
            continue;
        };

        let mut edits = Vec::new();
        for command in &data.semantics.commands {
            if command.text == name.text {
                let range = TextRange::new(command.range.start(), command.range.end());
                edits.push(range);
            }
        }

        builder.result.changes.insert(*document, edits);
    }

    Some(())
}
