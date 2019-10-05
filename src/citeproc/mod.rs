use crate::formatting::bibtex::{format_entry, BibtexFormattingParams};
use crate::syntax::*;
use ducc::{Ducc, ExecSettings, Function};
use lsp_types::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;

const JS_CODE: &str = include_str!("js/dist/citeproc.js");
const WRAPPER: &str = "(function execute(code) { return citeproc.default(code); })";

pub struct JavaScriptEngine {
    ducc: Ducc,
}

impl JavaScriptEngine {
    fn new() -> Self {
        let ducc = Ducc::new();
        ducc.exec::<()>(JS_CODE, None, ExecSettings::default()).unwrap();
        Self { ducc }
    }

    pub fn initialize() {
        Lazy::force(&ENGINE);
    }
}

unsafe impl Send for JavaScriptEngine {}

static ENGINE: Lazy<Mutex<JavaScriptEngine>> = Lazy::new(|| Mutex::new(JavaScriptEngine::new()));

pub fn render_citation(tree: &BibtexSyntaxTree, key: &str) -> Option<MarkupContent> {
    let entries = tree.entries();
    let entry = entries
        .iter()
        .find(|entry| entry.key.as_ref().map(BibtexToken::text) == Some(key))?;
    if entry.fields.is_empty() {
        return None;
    }

    let entry = replace_strings(tree, &entry);
    let entry_code = format_entry(&entry, &BibtexFormattingParams::default());

    let engine = ENGINE.lock().unwrap();
    let entry_code = engine.ducc.create_string(&entry_code).unwrap();
    let func: Function = engine
        .ducc
        .compile(WRAPPER, None)
        .unwrap()
        .call(())
        .unwrap();
    let html: String = func.call((entry_code, ())).ok()?;
    let markdown = html2md::parse_html(&html);
    Some(MarkupContent {
        kind: MarkupKind::Markdown,
        value: markdown.trim().to_owned(),
    })
}

fn replace_strings(tree: &BibtexSyntaxTree, old_entry: &BibtexEntry) -> BibtexEntry {
    let mut strings = Vec::new();
    for child in &tree.root.children {
        if let BibtexDeclaration::String(string) = &child {
            if string.value.is_some() {
                strings.push(string);
            }
        }
    }

    let mut new_entry = old_entry.clone();
    for field in &mut new_entry.fields {
        if let Some(BibtexContent::Word(reference)) = &field.content {
            if let Some(string) = strings
                .iter()
                .find(|string| string.name.as_ref().unwrap().text() == reference.token.text())
            {
                field.content = string.value.clone();
            }
        }
    }
    new_entry
}
