use rustc_hash::FxHashMap;

use base_db::{Config, FeatureParams};
use parser::SyntaxConfig;

use crate::{RenameInformation, RenameParams};

fn check_with_syntax_config(config: SyntaxConfig, input: &str) {
    let mut fixture = test_utils::fixture::Fixture::parse(input);
    fixture.workspace.set_config(Config {
        syntax: config,
        ..Config::default()
    });
    let fixture = fixture;

    let mut expected: FxHashMap<_, Vec<RenameInformation>> = FxHashMap::default();
    for spec in &fixture.documents {
        if !spec.ranges.is_empty() {
            let document = fixture.workspace.lookup(&spec.uri).unwrap();
            expected.insert(
                document,
                spec.ranges.iter().map(|r| r.clone().into()).collect(),
            );
        }
    }
    let (feature, offset) = fixture.make_params().unwrap();
    let actual = crate::rename(RenameParams { feature, offset });
    assert_eq!(actual.changes, expected);
}

fn check(input: &str) {
    check_with_syntax_config(SyntaxConfig::default(), input)
}

#[test]
fn test_command() {
    check(
        r#"
%! foo.tex
\baz
  |
 ^^^
\include{bar.tex}

%! bar.tex
\baz
 ^^^
"#,
    )
}

#[test]
fn test_entry() {
    check(
        r#"
%! main.bib
@article{foo, bar = baz}
         |
         ^^^

%! main.tex
\addbibresource{main.bib}
\cite{foo}
      ^^^
"#,
    )
}

#[test]
fn test_citation() {
    check(
        r#"
%! main.bib
@article{foo, bar = baz}
         ^^^

%! main.tex
\addbibresource{main.bib}
\cite{foo}
       |
      ^^^
"#,
    )
}

#[test]
fn test_label() {
    check(
        r#"
%! foo.tex
\label{foo}\include{bar}
       |
       ^^^

%! baz.tex
\ref{foo}

%! bar.tex
\ref{foo}
     ^^^
"#,
    )
}

#[test]
fn test_custom_label_ref() {
    let mut config = SyntaxConfig::default();
    config
        .label_definition_commands
        .extend(vec!["asm", "goal"].into_iter().map(String::from));
    config.label_definition_prefixes.extend(
        vec![("asm", "asm:"), ("goal", "goal:")]
            .into_iter()
            .map(|(x, y)| (String::from(x), String::from(y))),
    );
    config
        .label_reference_commands
        .extend(vec!["asmref", "goalref"].into_iter().map(String::from));
    config.label_reference_prefixes.extend(
        vec![("asmref", "asm:"), ("goalref", "goal:")]
            .into_iter()
            .map(|(x, y)| (String::from(x), String::from(y))),
    );
    check_with_syntax_config(
        config,
        r#"
%! foo.tex
\goal{foo}

\asm{foo}\include{bar}\include{baz}
     |
     ^^^

%! bar.tex
\asmref{foo}
        ^^^

%! baz.tex
\ref{foo}

\ref{asm:foo}
     ^^^^^^^

"#,
    )
}

#[test]
fn test_custom_label_def() {
    let mut config = SyntaxConfig::default();
    config
        .label_definition_commands
        .extend(vec!["asm", "goal"].into_iter().map(String::from));
    config.label_definition_prefixes.extend(
        vec![("asm", "asm:"), ("goal", "goal:")]
            .into_iter()
            .map(|(x, y)| (String::from(x), String::from(y))),
    );
    config
        .label_reference_commands
        .extend(vec!["asmref", "goalref"].into_iter().map(String::from));
    config.label_reference_prefixes.extend(
        vec![("asmref", "asm:"), ("goalref", "goal:")]
            .into_iter()
            .map(|(x, y)| (String::from(x), String::from(y))),
    );
    check_with_syntax_config(
        config,
        r#"
%! foo.tex
\goal{foo}

\label{asm:foo}\include{bar}\include{baz}
       |
       ^^^^^^^

%! bar.tex
\asmref{foo}
        ^^^

%! baz.tex
\ref{foo}

\ref{asm:foo}
     ^^^^^^^
"#,
    )
}
