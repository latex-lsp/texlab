use std::sync::Arc;

use crate::Uri;

use super::LatexAnalyzerContext;

pub fn analyze_implicit_links(context: &mut LatexAnalyzerContext) {
    context.extras.implicit_links.aux = find_by_extension(context, "aux").unwrap_or_default();
    context.extras.implicit_links.log = find_by_extension(context, "log").unwrap_or_default();
    context.extras.implicit_links.pdf = find_by_extension(context, "pdf").unwrap_or_default();
}

fn find_by_extension(context: &LatexAnalyzerContext, extension: &str) -> Option<Vec<Arc<Uri>>> {
    let mut targets = vec![Arc::new(context.document_uri.with_extension(extension)?)];
    if context.document_uri.scheme() == "file" {
        let file_path = context.document_uri.to_file_path().ok()?;
        let file_stem = file_path.file_stem()?;
        let aux_name = format!("{}.{}", file_stem.to_str()?, extension);

        let options = &context.workspace.options;
        if let Some(root_dir) = options.root_directory.as_ref() {
            let path = context
                .workspace
                .current_directory
                .join(root_dir)
                .join(&aux_name);
            targets.push(Arc::new(Uri::from_file_path(path).ok()?));
        }

        if let Some(build_dir) = options.aux_directory.as_ref() {
            let path = context
                .workspace
                .current_directory
                .join(build_dir)
                .join(&aux_name);
            targets.push(Arc::new(Uri::from_file_path(path).ok()?));
        }
    }
    Some(targets)
}
