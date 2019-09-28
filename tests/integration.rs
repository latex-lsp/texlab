use lsp_types::*;
use std::collections::HashMap;
use texlab::definition::DefinitionResponse;
use texlab::formatting::bibtex::BibtexFormattingOptions;
use texlab::range::RangeExt;
use texlab::test_scenario::*;

async fn run_completion(
    scenario_short_name: &'static str,
    file: &'static str,
    line: u64,
    character: u64,
) -> (TestScenario, Vec<CompletionItem>) {
    let scenario_name = format!("completion/{}", scenario_short_name);
    let scenario = TestScenario::new(&scenario_name, &DEFAULT_CAPABILITIES).await;
    scenario.open(file).await;

    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier::new(scenario.uri(file).into()),
            position: Position::new(line, character),
        },
        context: None,
    };

    let items = scenario
        .server
        .execute_async(|svr| svr.completion(params))
        .await
        .unwrap()
        .items;

    (scenario, items)
}

async fn run_completion_empty(
    scenario_short_name: &'static str,
    file: &'static str,
    line: u64,
    character: u64,
) {
    assert!(run_completion(scenario_short_name, file, line, character)
        .await
        .1
        .is_empty());
}

async fn run_completion_item(
    scenario_short_name: &'static str,
    file: &'static str,
    line: u64,
    character: u64,
    item_name: &'static str,
) -> CompletionItem {
    let (scenario, items) = run_completion(scenario_short_name, file, line, character).await;

    let item = items
        .into_iter()
        .find(|item| item.label == item_name)
        .unwrap();

    scenario
        .server
        .execute_async(|svr| svr.completion_resolve(item))
        .await
        .unwrap()
}

async fn verify_labels(
    scenario_short_name: &'static str,
    file: &'static str,
    line: u64,
    character: u64,
    expected_labels: Vec<&'static str>,
) {
    let (_, items) = run_completion(scenario_short_name, file, line, character).await;
    let actual_labels: Vec<&str> = items.iter().map(|item| item.label.as_ref()).collect();
    assert_eq!(actual_labels, expected_labels);
}

fn verify_text_edit(
    item: &CompletionItem,
    start_line: u64,
    start_character: u64,
    end_line: u64,
    end_character: u64,
    text: &str,
) {
    assert_eq!(
        item.text_edit,
        Some(TextEdit::new(
            Range::new_simple(start_line, start_character, end_line, end_character),
            text.into()
        ))
    );
}

fn verify_detail(item: &CompletionItem, detail: &str) {
    assert_eq!(item.detail.as_ref().unwrap(), detail);
}

#[tokio::test]
async fn completion_bibtex_command() {
    let item = run_completion_item("bibtex/command", "foo.bib", 1, 15, "LaTeX").await;
    verify_text_edit(&item, 1, 15, 1, 18, "LaTeX");
    verify_detail(&item, "built-in");
}

#[tokio::test]
async fn completion_bibtex_field() {
    let item = run_completion_item("bibtex/field", "foo.bib", 1, 6, "title").await;
    assert!(item.documentation.is_some());
    verify_text_edit(&item, 1, 4, 1, 8, "title");
}

#[tokio::test]
async fn completion_bibtex_type() {
    let item = run_completion_item("bibtex/type", "foo.bib", 0, 1, "article").await;
    assert!(item.documentation.is_some());
    verify_text_edit(&item, 0, 1, 0, 1, "article");
}

#[tokio::test]
async fn completion_bibtex_word() {
    run_completion_empty("bibtex/word", "foo.bib", 0, 00).await;
    run_completion_empty("bibtex/word", "foo.bib", 2, 14).await;
    run_completion_empty("bibtex/word", "foo.bib", 6, 14).await;
}

#[tokio::test]
async fn completion_latex_citation() {
    let item = run_completion_item("latex/citation", "foo.tex", 5, 6, "foo:2019").await;
    verify_text_edit(&item, 5, 6, 5, 6, "foo:2019");
    assert_eq!(
        item.documentation.unwrap(),
        Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "Bar, F. (2019). Baz Qux.".into()
        })
    );
}

#[tokio::test]
async fn completion_latex_class() {
    let item = run_completion_item("latex/class", "foo.tex", 0, 18, "article").await;
    assert!(item.documentation.is_some());
    verify_text_edit(&item, 0, 15, 0, 18, "article");
}

#[tokio::test]
async fn completion_latex_class_command() {
    let item = run_completion_item("latex/class_command", "foo.tex", 1, 3, "chapter").await;
    verify_text_edit(&item, 1, 1, 1, 5, "chapter");
    assert_eq!(item.detail.unwrap(), "book.cls");
}

#[tokio::test]
async fn completion_latex_class_environment() {
    let item = run_completion_item("latex/class_environment", "foo.tex", 1, 11, "theindex").await;
    verify_text_edit(&item, 1, 7, 1, 11, "theindex");
    verify_detail(&item, "article.cls");
}

#[tokio::test]
async fn completion_latex_color() {
    let item = run_completion_item("latex/color", "foo.tex", 0, 9, "red").await;
    verify_text_edit(&item, 0, 7, 0, 9, "red");
}

#[tokio::test]
async fn completion_latex_color_model() {
    let item = run_completion_item("latex/color_model", "foo.tex", 0, 18, "rgb").await;
    verify_text_edit(&item, 0, 18, 0, 18, "rgb");

    let item = run_completion_item("latex/color_model", "foo.tex", 1, 17, "RGB").await;
    verify_text_edit(&item, 1, 16, 1, 17, "RGB");
}

#[tokio::test]
async fn completion_latex_glyph() {
    let item = run_completion_item("latex/glyph", "foo.tex", 0, 7, "varepsilon").await;
    verify_text_edit(&item, 0, 1, 0, 7, "varepsilon");
    verify_detail(&item, "ε, built-in");
}

#[tokio::test]
async fn completion_latex_include() {
    let name = "latex/include";
    verify_labels(name, "foo.tex", 2, 09, vec!["bar", "foo", "qux"]).await;
    verify_labels(name, "foo.tex", 3, 07, vec!["bar.tex", "foo.tex", "qux"]).await;
    verify_labels(name, "foo.tex", 4, 11, vec!["baz.tex"]).await;
    verify_labels(name, "foo.tex", 5, 16, vec!["bibliography.bib", "qux"]).await;
}

#[tokio::test]
async fn completion_latex_label() {
    let (_, items) = run_completion("latex/label", "bar.tex", 4, 5).await;
    assert_eq!(items.len(), 6);
    verify_text_edit(&items[0], 4, 5, 4, 5, "sec:bar");
    verify_text_edit(&items[1], 4, 5, 4, 5, "sec:foo");
    verify_text_edit(&items[2], 4, 5, 4, 5, "eq:foo");
    verify_text_edit(&items[3], 4, 5, 4, 5, "eq:bar");
    verify_text_edit(&items[4], 4, 5, 4, 5, "fig:baz");
    verify_text_edit(&items[5], 4, 5, 4, 5, "thm:foo");
    verify_detail(&items[0], "2 Bar");
    verify_detail(&items[1], "1 Foo");
    verify_detail(&items[2], "Equation (1)");
    verify_detail(&items[3], "Equation (2)");
    verify_detail(&items[4], "Figure 1");
    verify_detail(&items[5], "Lemma 1");
    assert_eq!(
        *items[4].documentation.as_ref().unwrap(),
        Documentation::String("Baz".into())
    );
    verify_labels("latex/label", "bar.tex", 5, 7, vec!["eq:foo", "eq:bar"]).await;
}

#[tokio::test]
async fn completion_latex_package() {
    let item = run_completion_item("latex/package", "foo.tex", 1, 15, "amsmath").await;
    assert!(item.documentation.is_some());
    verify_text_edit(&item, 1, 12, 1, 15, "amsmath");
}

#[tokio::test]
async fn completion_latex_package_command() {
    let item = run_completion_item("latex/package_command", "foo.tex", 1, 7, "varDelta").await;
    verify_text_edit(&item, 1, 1, 1, 7, "varDelta");
    verify_detail(&item, "amsmath.sty");
}

#[tokio::test]
async fn completion_latex_package_environment() {
    let item = run_completion_item("latex/package_environment", "foo.tex", 1, 9, "align").await;
    verify_text_edit(&item, 1, 5, 1, 9, "align");
    verify_detail(&item, "amsmath.sty");
}

#[tokio::test]
async fn completion_latex_preselect() {
    let item = run_completion_item("latex/preselect", "foo.tex", 2, 5, "document").await;
    assert_eq!(item.preselect, Some(true));
}

#[tokio::test]
async fn completion_latex_theorem() {
    let item = run_completion_item("latex/theorem", "foo.tex", 4, 7, "foo").await;
    verify_text_edit(&item, 4, 7, 4, 8, "foo");
    verify_detail(&item, "user-defined");
}

#[tokio::test]
async fn completion_latex_tikz() {
    let item = run_completion_item("latex/tikz", "foo.tex", 1, 15, "arrows").await;
    verify_text_edit(&item, 1, 15, 1, 15, "arrows");
    let item = run_completion_item("latex/tikz", "foo.tex", 2, 16, "arrows").await;
    verify_text_edit(&item, 2, 16, 2, 16, "arrows");
}

#[tokio::test]
async fn completion_latex_user_command() {
    let item = run_completion_item("latex/user_command", "foo.tex", 1, 3, "foo").await;
    verify_text_edit(&item, 1, 1, 1, 3, "foo");
    verify_detail(&item, "user-defined");
}

#[tokio::test]
async fn completion_latex_user_environment() {
    let item = run_completion_item("latex/user_environment", "foo.tex", 2, 7, "foo").await;
    verify_text_edit(&item, 2, 7, 2, 9, "foo");
    verify_detail(&item, "user-defined");
}

async fn run_definition(
    scenario_short_name: &'static str,
    file: &'static str,
    line: u64,
    character: u64,
    capabilities: &ClientCapabilities,
) -> (TestScenario, DefinitionResponse) {
    let scenario_name = format!("definition/{}", scenario_short_name);
    let scenario = TestScenario::new(&scenario_name, capabilities).await;
    scenario.open(file).await;

    let params = TextDocumentPositionParams {
        text_document: TextDocumentIdentifier::new(scenario.uri(file).into()),
        position: Position::new(line, character),
    };

    let response = scenario
        .server
        .execute_async(|svr| svr.definition(params))
        .await
        .unwrap();

    (scenario, response)
}

async fn run_definition_link(
    scenario_short_name: &'static str,
    file: &'static str,
    line: u64,
    character: u64,
) -> (TestScenario, Vec<LocationLink>) {
    let (scenario, response) = run_definition(
        scenario_short_name,
        file,
        line,
        character,
        &DEFAULT_CAPABILITIES,
    )
    .await;
    match response {
        DefinitionResponse::LocationLinks(links) => (scenario, links),
        DefinitionResponse::Locations(_) => unreachable!(),
    }
}

async fn run_definition_location(
    scenario_short_name: &'static str,
    file: &'static str,
    line: u64,
    character: u64,
) -> (TestScenario, Vec<Location>) {
    let (scenario, response) = run_definition(
        scenario_short_name,
        file,
        line,
        character,
        &NO_LINK_SUPPORT_CAPABILITIES,
    )
    .await;
    match response {
        DefinitionResponse::LocationLinks(_) => unreachable!(),
        DefinitionResponse::Locations(locations) => (scenario, locations),
    }
}

fn verify_origin_selection_range(
    link: &LocationLink,
    start_line: u64,
    start_character: u64,
    end_line: u64,
    end_character: u64,
) {
    assert_eq!(
        link.origin_selection_range,
        Some(Range::new_simple(
            start_line,
            start_character,
            end_line,
            end_character
        ))
    );
}

#[tokio::test]
async fn definition_latex_citation() {
    let (scenario, mut links) = run_definition_link("latex/citation", "foo.tex", 1, 7).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    assert_eq!(
        link.origin_selection_range.unwrap(),
        Range::new_simple(1, 6, 1, 9)
    );
    assert_eq!(link.target_uri, scenario.uri("bar.bib").into());
    assert_eq!(link.target_range, Range::new_simple(2, 0, 2, 14));
    assert_eq!(link.target_selection_range, Range::new_simple(2, 9, 2, 12));

    let (scenario, mut locations) =
        run_definition_location("latex/citation", "foo.tex", 1, 7).await;
    assert_eq!(locations.len(), 1);
    let location = locations.pop().unwrap();
    assert_eq!(location.uri, scenario.uri("bar.bib").into());
    assert_eq!(location.range, Range::new_simple(2, 9, 2, 12));
}

#[tokio::test]
async fn definition_latex_command() {
    let (scenario, mut links) = run_definition_link("latex/command", "foo.tex", 2, 2).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify_origin_selection_range(&link, 2, 0, 2, 4);
    assert_eq!(link.target_uri, scenario.uri("foo.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 0, 22));
    assert_eq!(link.target_selection_range, Range::new_simple(0, 0, 0, 22));
}

#[tokio::test]
async fn definition_latex_math_operator() {
    let (scenario, mut links) = run_definition_link("latex/math_operator", "foo.tex", 2, 2).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify_origin_selection_range(&link, 2, 0, 2, 4);
    assert_eq!(link.target_uri, scenario.uri("foo.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 0, 31));
    assert_eq!(link.target_selection_range, Range::new_simple(0, 0, 0, 31));
}

#[tokio::test]
async fn definition_latex_label_default() {
    let (scenario, mut links) = run_definition_link("latex/label", "default.tex", 1, 7).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify_origin_selection_range(&link, 1, 5, 1, 8);
    assert_eq!(link.target_uri, scenario.uri("default.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 0, 11));
    assert_eq!(link.target_selection_range, Range::new_simple(0, 0, 0, 11));
}

#[tokio::test]
async fn definition_latex_label_equation() {
    let (scenario, mut links) = run_definition_link("latex/label", "equation.tex", 5, 8).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify_origin_selection_range(&link, 5, 5, 5, 11);
    assert_eq!(link.target_uri, scenario.uri("equation.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 3, 14));
    assert_eq!(link.target_selection_range, Range::new_simple(1, 0, 1, 14));
}

#[tokio::test]
async fn definition_latex_label_float() {
    let (scenario, mut links) = run_definition_link("latex/label", "float.tex", 6, 6).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify_origin_selection_range(&link, 6, 5, 6, 8);
    assert_eq!(link.target_uri, scenario.uri("float.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 4, 12));
    assert_eq!(link.target_selection_range, Range::new_simple(3, 0, 3, 11));
}

#[tokio::test]
async fn definition_latex_label_item() {
    let (scenario, mut links) = run_definition_link("latex/label", "item.tex", 6, 6).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify_origin_selection_range(&link, 6, 5, 6, 8);
    assert_eq!(link.target_uri, scenario.uri("item.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 4, 15));
    assert_eq!(link.target_selection_range, Range::new_simple(2, 9, 2, 20));
}

#[tokio::test]
async fn definition_latex_label_section() {
    let (scenario, mut links) = run_definition_link("latex/label", "section.tex", 6, 6).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify_origin_selection_range(&link, 6, 5, 6, 12);
    assert_eq!(link.target_uri, scenario.uri("section.tex").into());
    assert_eq!(link.target_range, Range::new_simple(0, 0, 3, 0));
    assert_eq!(link.target_selection_range, Range::new_simple(1, 0, 1, 15));
}

#[tokio::test]
async fn definition_latex_label_theorem() {
    let (scenario, mut links) = run_definition_link("latex/label", "theorem.tex", 8, 7).await;
    assert_eq!(links.len(), 1);
    let link = links.pop().unwrap();
    verify_origin_selection_range(&link, 8, 5, 8, 12);
    assert_eq!(link.target_uri, scenario.uri("theorem.tex").into());
    assert_eq!(link.target_range, Range::new_simple(3, 0, 6, 11));
    assert_eq!(link.target_selection_range, Range::new_simple(4, 0, 4, 15));
}

async fn run_bibtex_formatting(
    scenario_short_name: &'static str,
    file: &'static str,
    options: Option<BibtexFormattingOptions>,
) -> (TestScenario, Vec<TextEdit>) {
    let scenario_name = format!("formatting/bibtex/{}", scenario_short_name);
    let scenario = TestScenario::new(&scenario_name, &DEFAULT_CAPABILITIES).await;
    scenario.open(file).await;
    {
        scenario.client.options.lock().await.bibtex_formatting = options;
    }

    let params = DocumentFormattingParams {
        text_document: TextDocumentIdentifier::new(scenario.uri(file).into()),
        options: FormattingOptions {
            tab_size: 4,
            insert_spaces: true,
            properties: HashMap::new(),
        },
    };

    let edits = scenario
        .server
        .execute_async(|svr| svr.formatting(params))
        .await
        .unwrap();
    (scenario, edits)
}

#[tokio::test]
async fn formatting_bibtex_default_settings() {
    let (scenario, edits) = run_bibtex_formatting("default", "unformatted.bib", None).await;
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, scenario.read("formatted.bib").await);
    assert_eq!(edits[0].range, Range::new_simple(0, 0, 0, 52));
}

#[tokio::test]
async fn formatting_bibtex_infinite_line_length() {
    let (scenario, edits) = run_bibtex_formatting(
        "infinite_line_length",
        "unformatted.bib",
        Some(BibtexFormattingOptions {
            line_length: Some(0),
        }),
    )
    .await;
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, scenario.read("formatted.bib").await);
    assert_eq!(edits[0].range, Range::new_simple(0, 0, 0, 149));
}
