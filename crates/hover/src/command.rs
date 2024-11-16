use syntax::latex;

use crate::{Hover, HoverData, HoverParams};

pub(super) fn find_hover<'a>(params: &HoverParams<'a>) -> Option<Hover<'a>> {
    let data = params.feature.document.data.as_tex()?;
    let name = data
        .root_node()
        .token_at_offset(params.offset)
        .find(|token| token.kind() == latex::COMMAND_NAME)?;

    let command = completion_data::included_packages(&params.feature)
        .flat_map(|package| package.commands.iter())
        .find(|command| command.name == &name.text()[1..])?;

    let range = name.text_range();
    let data = HoverData::Command(command);
    Some(Hover { range, data })
}
