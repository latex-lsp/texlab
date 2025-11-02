mod bib;
mod tex;

use base_db::{Document, DocumentData, SymbolConfig, Workspace, deps::Project, util};

use crate::Symbol;

pub fn document_symbols(workspace: &Workspace, document: &Document) -> Vec<Symbol> {
    let project = Project::from_child(workspace, document);
    let mut symbols = match &document.data {
        DocumentData::Tex(data) => {
            let builder = tex::SymbolBuilder::new(&project, workspace.config());
            builder.visit(&data.root_node())
        }
        DocumentData::Bib(data) => {
            let builder = bib::SymbolBuilder;
            data.root_node()
                .children()
                .filter_map(|node| builder.visit(&node))
                .collect()
        }
        DocumentData::Aux(_)
        | DocumentData::Log(_)
        | DocumentData::Root
        | DocumentData::Latexmkrc(_)
        | DocumentData::FileList(_)
        | DocumentData::Tectonic => Vec::new(),
    };

    filter_symbols(&mut symbols, &workspace.config().symbols);
    symbols
}

fn filter_symbols(container: &mut Vec<Symbol>, config: &SymbolConfig) {
    let allowed = &config.allowed_patterns;
    let ignored = &config.ignored_patterns;

    let mut i = 0;
    while i < container.len() {
        let symbol = &mut container[i];
        if symbol.name.is_empty() || !util::filter_regex_patterns(&symbol.name, allowed, ignored) {
            let mut symbol = container.remove(i);
            container.append(&mut symbol.children);
        } else {
            filter_symbols(&mut symbol.children, config);
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests;
