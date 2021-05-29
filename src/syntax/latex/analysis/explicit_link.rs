use std::sync::Arc;

use crate::syntax::{latex, CstNode};

use super::{
    distro_file::resolve_distro_file, ExplicitLink, ExplicitLinkKind, LatexAnalyzerContext,
};

pub fn analyze_include(context: &mut LatexAnalyzerContext, node: &latex::SyntaxNode) -> Option<()> {
    let include = latex::Include::cast(node)?;
    let kind = match include.syntax().kind() {
        latex::LATEX_INCLUDE => ExplicitLinkKind::Latex,
        latex::BIBLATEX_INCLUDE | latex::BIBTEX_INCLUDE => ExplicitLinkKind::Bibtex,
        latex::PACKAGE_INCLUDE => ExplicitLinkKind::Package,
        latex::CLASS_INCLUDE => ExplicitLinkKind::Class,
        _ => return None,
    };

    let extensions = match kind {
        ExplicitLinkKind::Latex => &["tex"],
        ExplicitLinkKind::Bibtex => &["bib"],
        ExplicitLinkKind::Package => &["sty"],
        ExplicitLinkKind::Class => &["cls"],
    };

    for path in include.path_list()?.keys() {
        let stem = path.to_string();
        let mut targets = vec![Arc::new(context.base_uri.join(&stem).ok()?.into())];
        for extension in extensions {
            let path = format!("{}.{}", stem, extension);
            targets.push(Arc::new(context.base_uri.join(&path).ok()?.into()));
        }

        resolve_distro_file(&context.inner.resolver.lock().unwrap(), &stem, extensions)
            .into_iter()
            .for_each(|target| targets.push(Arc::new(target)));

        context.extras.explicit_links.push(ExplicitLink {
            kind,
            stem: stem.into(),
            stem_range: path.small_range(),
            targets,
        });
    }

    Some(())
}

pub fn analyze_import(context: &mut LatexAnalyzerContext, node: &latex::SyntaxNode) -> Option<()> {
    let import = latex::Import::cast(node)?;

    let mut targets = Vec::new();
    let directory = context
        .base_uri
        .join(&import.directory()?.key()?.to_string())
        .ok()?;

    let file = import.file()?.key()?;
    let stem = file.to_string();
    targets.push(Arc::new(directory.join(&stem).ok()?.into()));
    targets.push(Arc::new(
        directory.join(&format!("{}.tex", stem)).ok()?.into(),
    ));

    context.extras.explicit_links.push(ExplicitLink {
        stem: stem.into(),
        stem_range: file.small_range(),
        targets,
        kind: ExplicitLinkKind::Latex,
    });
    Some(())
}
