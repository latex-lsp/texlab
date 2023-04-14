use base_db::Document;
use rowan::{ast::AstNode, TextRange, TextSize};
use syntax::latex;

#[derive(Debug)]
pub struct ChangeEnvironmentResult<'a> {
    pub begin: TextRange,
    pub end: TextRange,
    pub old_name: String,
    pub new_name: &'a str,
}

pub fn change_environment<'a>(
    document: &'a Document,
    position: TextSize,
    new_name: &'a str,
) -> Option<ChangeEnvironmentResult<'a>> {
    let root = document.data.as_tex()?.root_node();

    let environment = root
        .token_at_offset(position)
        .right_biased()?
        .parent_ancestors()
        .find_map(latex::Environment::cast)?;

    let begin = environment.begin()?.name()?.key()?;
    let end = environment.end()?.name()?.key()?;

    let old_name = begin.to_string();
    if old_name != end.to_string() {
        return None;
    }

    Some(ChangeEnvironmentResult {
        begin: latex::small_range(&begin),
        end: latex::small_range(&end),
        old_name,
        new_name,
    })
}
