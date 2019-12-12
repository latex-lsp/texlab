use texlab_workspace::*;
use futures_boxed::boxed;
use texlab_protocol::RangeExt;
use texlab_protocol::{Location, ReferenceParams};
use texlab_syntax::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexEntryReferenceProvider;

impl FeatureProvider for BibtexEntryReferenceProvider {
    type Params = ReferenceParams;
    type Output = Vec<Location>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<ReferenceParams>) -> Vec<Location> {
        let mut references = Vec::new();
        if let Some(key) = Self::find_key(request) {
            for document in request.related_documents() {
                match &document.tree {
                    SyntaxTree::Latex(tree) => tree
                        .citations
                        .iter()
                        .flat_map(LatexCitation::keys)
                        .filter(|citation| citation.text() == key)
                        .map(|citation| {
                            Location::new(document.uri.clone().into(), citation.range())
                        })
                        .for_each(|location| references.push(location)),
                    SyntaxTree::Bibtex(tree) => {
                        if request.params.context.include_declaration {
                            for entry in tree.entries() {
                                if let Some(key_token) = &entry.key {
                                    if key_token.text() == key {
                                        let uri = document.uri.clone();
                                        let location = Location::new(uri.into(), key_token.range());
                                        references.push(location);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        references
    }
}

impl BibtexEntryReferenceProvider {
    fn find_key(request: &FeatureRequest<ReferenceParams>) -> Option<&str> {
        match &request.document().tree {
            SyntaxTree::Latex(tree) => tree
                .citations
                .iter()
                .flat_map(LatexCitation::keys)
                .find(|key| {
                    key.range()
                        .contains(request.params.text_document_position.position)
                })
                .map(LatexToken::text),
            SyntaxTree::Bibtex(tree) => {
                for entry in tree.entries() {
                    if let Some(key) = &entry.key {
                        if key
                            .range()
                            .contains(request.params.text_document_position.position)
                        {
                            return Some(key.text());
                        }
                    }
                }
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texlab_protocol::RangeExt;
    use texlab_protocol::{Position, Range};

    #[test]
    fn test_entry() {
        let references = test_feature(
            BibtexEntryReferenceProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.bib", "@article{foo, bar = {baz}}"),
                    FeatureSpec::file("bar.tex", "\\addbibresource{foo.bib}\n\\cite{foo}"),
                    FeatureSpec::file("baz.tex", "\\cite{foo}"),
                ],
                main_file: "foo.bib",
                position: Position::new(0, 9),
                include_declaration: false,
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            references,
            vec![Location::new(
                FeatureSpec::uri("bar.tex"),
                Range::new_simple(1, 6, 1, 9)
            )]
        );
    }

    #[test]
    fn test_entry_include_declaration() {
        let references = test_feature(
            BibtexEntryReferenceProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.bib", "@article{foo, bar = {baz}}"),
                    FeatureSpec::file("bar.tex", "\\addbibresource{foo.bib}\n\\cite{foo}"),
                    FeatureSpec::file("baz.tex", "\\cite{foo}"),
                ],
                main_file: "foo.bib",
                position: Position::new(0, 9),
                include_declaration: true,
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            references,
            vec![
                Location::new(FeatureSpec::uri("foo.bib"), Range::new_simple(0, 9, 0, 12)),
                Location::new(FeatureSpec::uri("bar.tex"), Range::new_simple(1, 6, 1, 9)),
            ]
        );
    }

    #[test]
    fn test_citation() {
        let references = test_feature(
            BibtexEntryReferenceProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.bib", "@article{foo, bar = {baz}}"),
                    FeatureSpec::file("bar.tex", "\\addbibresource{foo.bib}\n\\cite{foo}"),
                    FeatureSpec::file("baz.tex", "\\cite{foo}"),
                ],
                main_file: "bar.tex",
                position: Position::new(1, 8),
                include_declaration: false,
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            references,
            vec![Location::new(
                FeatureSpec::uri("bar.tex"),
                Range::new_simple(1, 6, 1, 9)
            )]
        );
    }

    #[test]
    fn test_citation_include_declaration() {
        let references = test_feature(
            BibtexEntryReferenceProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.bib", "@article{foo, bar = {baz}}"),
                    FeatureSpec::file("bar.tex", "\\addbibresource{foo.bib}\n\\cite{foo}"),
                    FeatureSpec::file("baz.tex", "\\cite{foo}"),
                ],
                main_file: "bar.tex",
                position: Position::new(1, 9),
                include_declaration: true,
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            references,
            vec![
                Location::new(FeatureSpec::uri("bar.tex"), Range::new_simple(1, 6, 1, 9)),
                Location::new(FeatureSpec::uri("foo.bib"), Range::new_simple(0, 9, 0, 12)),
            ]
        );
    }

    #[test]
    fn test_empty() {
        let references = test_feature(
            BibtexEntryReferenceProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "")],
                main_file: "foo.tex",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(references.is_empty());
    }
}
