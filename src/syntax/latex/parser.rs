use rowan::{GreenNode, GreenNodeBuilder};

use super::{
    lexer::Lexer,
    SyntaxKind::{self, *},
};

#[derive(Clone)]
pub struct Parse {
    pub green: GreenNode,
}

#[derive(Debug, Clone, Copy)]
struct ParserContext {
    allow_environment: bool,
    allow_comma: bool,
}

impl Default for ParserContext {
    fn default() -> Self {
        Self {
            allow_environment: true,
            allow_comma: true,
        }
    }
}

#[derive(Debug)]
struct Parser<'a> {
    lexer: Lexer<'a>,
    builder: GreenNodeBuilder<'static>,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            lexer: Lexer::new(text),
            builder: GreenNodeBuilder::new(),
        }
    }

    fn eat(&mut self) {
        let (kind, text) = self.lexer.eat().unwrap();
        self.builder.token(kind.into(), text);
    }

    fn peek(&self) -> Option<SyntaxKind> {
        self.lexer.peek()
    }

    fn expect(&mut self, kind: SyntaxKind) {
        if self.peek() == Some(kind) {
            self.eat();
            self.trivia();
        } else {
            self.builder.token(MISSING.into(), "");
        }
    }

    fn expect2(&mut self, kind1: SyntaxKind, kind2: SyntaxKind) {
        if self
            .peek()
            .filter(|&kind| kind == kind1 || kind == kind2)
            .is_some()
        {
            self.eat();
            self.trivia();
        } else {
            self.builder.token(MISSING.into(), "");
        }
    }

    fn trivia(&mut self) {
        while self
            .peek()
            .filter(|&kind| matches!(kind, LINE_BREAK | WHITESPACE | COMMENT))
            .is_some()
        {
            self.eat();
        }
    }

    pub fn parse(mut self) -> Parse {
        self.builder.start_node(ROOT.into());
        self.preamble();
        while self.peek().is_some() {
            self.content(ParserContext::default());
        }
        self.builder.finish_node();
        let green = self.builder.finish();
        Parse { green }
    }

    fn content(&mut self, context: ParserContext) {
        match self.peek().unwrap() {
            LINE_BREAK | WHITESPACE | COMMENT | VERBATIM => self.eat(),
            L_CURLY if context.allow_environment => self.curly_group(),
            L_CURLY => self.curly_group_without_environments(),
            L_BRACK | L_PAREN => self.mixed_group(),
            R_CURLY | R_BRACK | R_PAREN => {
                self.builder.start_node(ERROR.into());
                self.eat();
                self.builder.finish_node();
            }
            WORD | COMMA => self.text(context),
            EQUALITY_SIGN => self.eat(),
            DOLLAR => self.formula(),
            GENERIC_COMMAND_NAME => self.generic_command(),
            BEGIN_ENVIRONMENT_NAME if context.allow_environment => self.environment(),
            BEGIN_ENVIRONMENT_NAME => self.generic_command(),
            END_ENVIRONMENT_NAME => self.generic_command(),
            BEGIN_EQUATION_NAME => self.equation(),
            END_EQUATION_NAME => self.generic_command(),
            MISSING | ERROR => self.eat(),
            PART_NAME => self.part(),
            CHAPTER_NAME => self.chapter(),
            SECTION_NAME => self.section(),
            SUBSECTION_NAME => self.subsection(),
            SUBSUBSECTION_NAME => self.subsubsection(),
            PARAGRAPH_NAME => self.paragraph(),
            SUBPARAGRAPH_NAME => self.subparagraph(),
            ENUM_ITEM_NAME => self.enum_item(),
            CAPTION_NAME => self.caption(),
            CITATION_NAME => self.citation(),
            PACKAGE_INCLUDE_NAME => self.package_include(),
            CLASS_INCLUDE_NAME => self.class_include(),
            LATEX_INCLUDE_NAME => self.latex_include(),
            BIBLATEX_INCLUDE_NAME => self.biblatex_include(),
            BIBTEX_INCLUDE_NAME => self.bibtex_include(),
            GRAPHICS_INCLUDE_NAME => self.graphics_include(),
            SVG_INCLUDE_NAME => self.svg_include(),
            INKSCAPE_INCLUDE_NAME => self.inkscape_include(),
            VERBATIM_INCLUDE_NAME => self.verbatim_include(),
            IMPORT_NAME => self.import(),
            LABEL_DEFINITION_NAME => self.label_definition(),
            LABEL_REFERENCE_NAME => self.label_reference(),
            LABEL_REFERENCE_RANGE_NAME => self.label_reference_range(),
            LABEL_NUMBER_NAME => self.label_number(),
            COMMAND_DEFINITION_NAME => self.command_definition(),
            MATH_OPERATOR_NAME => self.math_operator(),
            GLOSSARY_ENTRY_DEFINITION_NAME => self.glossary_entry_definition(),
            GLOSSARY_ENTRY_REFERENCE_NAME => self.glossary_entry_reference(),
            ACRONYM_DEFINITION_NAME => self.acronym_definition(),
            ACRONYM_DECLARATION_NAME => self.acronym_declaration(),
            ACRONYM_REFERENCE_NAME => self.acronym_reference(),
            THEOREM_DEFINITION_NAME => self.theorem_definition(),
            COLOR_REFERENCE_NAME => self.color_reference(),
            COLOR_DEFINITION_NAME => self.color_definition(),
            COLOR_SET_DEFINITION_NAME => self.color_set_definition(),
            TIKZ_LIBRARY_IMPORT_NAME => self.tikz_library_import(),
            ENVIRONMENT_DEFINITION_NAME => self.environment_definition(),
            BEGIN_BLOCK_COMMENT_NAME => self.block_comment(),
            END_BLOCK_COMMENT_NAME => self.generic_command(),
            GRAPHICS_PATH_NAME => self.graphics_path(),
            _ => unreachable!(),
        }
    }

    fn text(&mut self, context: ParserContext) {
        self.builder.start_node(TEXT.into());
        self.eat();
        while self
            .peek()
            .filter(|&kind| {
                matches!(kind, LINE_BREAK | WHITESPACE | COMMENT | WORD | COMMA)
                    && (context.allow_comma || kind != COMMA)
            })
            .is_some()
        {
            self.eat();
        }
        self.builder.finish_node();
    }

    fn curly_group(&mut self) {
        self.builder.start_node(CURLY_GROUP.into());
        self.eat();
        while self
            .peek()
            .filter(|&kind| !matches!(kind, R_CURLY | END_ENVIRONMENT_NAME))
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.expect(R_CURLY);
        self.builder.finish_node();
    }

    fn curly_group_impl(&mut self) {
        self.builder.start_node(CURLY_GROUP.into());
        self.eat();
        while let Some(kind) = self.peek() {
            match kind {
                R_CURLY => break,
                BEGIN_ENVIRONMENT_NAME => self.begin(),
                END_ENVIRONMENT_NAME => self.end(),
                _ => self.content(ParserContext::default()),
            };
        }
        self.expect(R_CURLY);
        self.builder.finish_node();
    }

    fn curly_group_without_environments(&mut self) {
        self.builder.start_node(CURLY_GROUP.into());
        self.eat();
        while self
            .peek()
            .filter(|&kind| !matches!(kind, R_CURLY))
            .is_some()
        {
            self.content(ParserContext {
                allow_environment: false,
                allow_comma: true,
            });
        }
        self.expect(R_CURLY);
        self.builder.finish_node();
    }

    fn curly_group_word(&mut self) {
        self.builder.start_node(CURLY_GROUP_WORD.into());
        self.eat();
        self.trivia();
        match self.peek() {
            Some(WORD) => {
                self.key();
            }
            Some(kind) if kind.is_command_name() => {
                self.content(ParserContext::default());
            }
            Some(_) | None => {
                self.builder.token(MISSING.into(), "");
            }
        }
        self.expect(R_CURLY);
        self.builder.finish_node();
    }

    fn curly_group_word_list(&mut self) {
        self.builder.start_node(CURLY_GROUP_WORD_LIST.into());
        self.eat();

        while self
            .peek()
            .filter(|&kind| matches!(kind, LINE_BREAK | WHITESPACE | COMMENT | WORD | COMMA))
            .is_some()
        {
            if self.peek() == Some(WORD) {
                self.key();
            } else {
                self.eat();
            }
        }

        self.expect(R_CURLY);
        self.builder.finish_node();
    }

    fn curly_group_command(&mut self) {
        self.builder.start_node(CURLY_GROUP_COMMAND.into());
        self.eat();
        self.trivia();
        match self.peek() {
            Some(kind) if kind.is_command_name() => {
                self.eat();
                self.trivia();
            }
            Some(_) | None => {
                self.builder.token(MISSING.into(), "");
            }
        }
        self.expect(R_CURLY);
        self.builder.finish_node();
    }

    fn brack_group(&mut self) {
        self.builder.start_node(BRACK_GROUP.into());
        self.eat();
        while self
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    R_CURLY
                        | R_BRACK
                        | PART_NAME
                        | CHAPTER_NAME
                        | SECTION_NAME
                        | SUBSECTION_NAME
                        | PARAGRAPH_NAME
                        | SUBPARAGRAPH_NAME
                        | ENUM_ITEM_NAME
                        | END_ENVIRONMENT_NAME
                )
            })
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.expect(R_BRACK);
        self.builder.finish_node();
    }

    fn brack_group_word(&mut self) {
        self.builder.start_node(BRACK_GROUP_WORD.into());
        self.eat();
        self.trivia();
        match self.peek() {
            Some(WORD) => {
                self.key();
            }
            Some(_) | None => {
                self.builder.token(MISSING.into(), "");
            }
        }
        self.expect(R_BRACK);
        self.builder.finish_node();
    }

    fn mixed_group(&mut self) {
        self.builder.start_node(MIXED_GROUP.into());
        self.eat();
        self.trivia();
        while self
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    R_CURLY
                        | R_BRACK
                        | R_PAREN
                        | PART_NAME
                        | CHAPTER_NAME
                        | SECTION_NAME
                        | SUBSECTION_NAME
                        | PARAGRAPH_NAME
                        | SUBPARAGRAPH_NAME
                        | ENUM_ITEM_NAME
                        | END_ENVIRONMENT_NAME
                )
            })
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.expect2(R_BRACK, R_PAREN);
        self.builder.finish_node();
    }

    fn key(&mut self) {
        self.builder.start_node(KEY.into());
        self.eat();
        while self
            .peek()
            .filter(|&kind| matches!(kind, WHITESPACE | COMMENT | WORD))
            .is_some()
        {
            self.eat();
        }

        self.trivia();
        self.builder.finish_node();
    }

    fn value(&mut self) {
        self.builder.start_node(VALUE.into());
        while let Some(kind) = self.lexer.peek() {
            match kind {
                COMMA | R_BRACK | R_CURLY => break,
                _ => self.content(ParserContext {
                    allow_environment: true,
                    allow_comma: false,
                }),
            };
        }
        self.builder.finish_node();
    }

    fn key_value_pair(&mut self) {
        self.builder.start_node(KEY_VALUE_PAIR.into());
        self.key();
        if self.peek() == Some(EQUALITY_SIGN) {
            self.eat();
            self.trivia();
            if self
                .peek()
                .filter(|&kind| {
                    !matches!(
                        kind,
                        END_ENVIRONMENT_NAME | R_CURLY | R_BRACK | R_PAREN | COMMA
                    )
                })
                .is_some()
            {
                self.value();
            } else {
                self.builder.token(MISSING.into(), "");
            }
        }

        self.builder.finish_node();
    }

    fn key_value_body(&mut self) {
        self.builder.start_node(KEY_VALUE_BODY.into());
        while let Some(kind) = self.peek() {
            match kind {
                LINE_BREAK | WHITESPACE | COMMENT => self.eat(),
                WORD => {
                    self.key_value_pair();
                    if self.peek() == Some(COMMA) {
                        self.eat();
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        self.builder.finish_node();
    }

    fn group_key_value(&mut self, node_kind: SyntaxKind, right_kind: SyntaxKind) {
        self.builder.start_node(node_kind.into());
        self.eat();
        self.trivia();
        self.key_value_body();
        self.expect(right_kind);
        self.builder.finish_node();
    }

    fn curly_group_key_value(&mut self) {
        self.group_key_value(CURLY_GROUP_KEY_VALUE, R_CURLY);
    }

    fn brack_group_key_value(&mut self) {
        self.group_key_value(BRACK_GROUP_KEY_VALUE, R_BRACK);
    }

    fn formula(&mut self) {
        self.builder.start_node(FORMULA.into());
        self.eat();
        self.trivia();
        while self
            .peek()
            .filter(|&kind| !matches!(kind, R_CURLY | END_ENVIRONMENT_NAME | DOLLAR))
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.expect(DOLLAR);
        self.builder.finish_node();
    }

    fn generic_command(&mut self) {
        self.builder.start_node(GENERIC_COMMAND.into());
        self.eat();
        while let Some(kind) = self.peek() {
            match kind {
                LINE_BREAK | WHITESPACE | COMMENT => self.eat(),
                L_CURLY => self.curly_group(),
                L_BRACK | L_PAREN => self.mixed_group(),
                _ => break,
            }
        }
        self.builder.finish_node();
    }

    fn equation(&mut self) {
        self.builder.start_node(EQUATION.into());
        self.eat();
        while self
            .peek()
            .filter(|&kind| !matches!(kind, END_ENVIRONMENT_NAME | R_CURLY | END_EQUATION_NAME))
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.expect(END_EQUATION_NAME);
        self.builder.finish_node();
    }

    fn begin(&mut self) {
        self.builder.start_node(BEGIN.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(L_CURLY) {
            self.curly_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.peek() == Some(L_BRACK) {
            self.brack_group();
        }
        self.builder.finish_node();
    }

    fn end(&mut self) {
        self.builder.start_node(END.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(L_CURLY) {
            self.curly_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn environment(&mut self) {
        self.builder.start_node(ENVIRONMENT.into());
        self.begin();

        while self
            .peek()
            .filter(|&kind| kind != END_ENVIRONMENT_NAME)
            .is_some()
        {
            self.content(ParserContext::default());
        }

        if self.peek() == Some(END_ENVIRONMENT_NAME) {
            self.end();
        } else {
            self.builder.token(MISSING.into(), "");
        }
        self.builder.finish_node();
    }

    fn preamble(&mut self) {
        self.builder.start_node(PREAMBLE.into());
        while self
            .peek()
            .filter(|&kind| kind != END_ENVIRONMENT_NAME)
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.builder.finish_node();
    }

    fn part(&mut self) {
        self.builder.start_node(PART.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(L_CURLY) {
            self.curly_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        while self
            .peek()
            .filter(|&kind| !matches!(kind, END_ENVIRONMENT_NAME | R_CURLY | PART_NAME))
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.builder.finish_node();
    }

    fn chapter(&mut self) {
        self.builder.start_node(CHAPTER.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(L_CURLY) {
            self.curly_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        while self
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    END_ENVIRONMENT_NAME | R_CURLY | PART_NAME | CHAPTER_NAME
                )
            })
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.builder.finish_node();
    }

    fn section(&mut self) {
        self.builder.start_node(SECTION.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(L_CURLY) {
            self.curly_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        while self
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    END_ENVIRONMENT_NAME | R_CURLY | PART_NAME | CHAPTER_NAME | SECTION_NAME
                )
            })
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.builder.finish_node();
    }

    fn subsection(&mut self) {
        self.builder.start_node(SUBSECTION.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(L_CURLY) {
            self.curly_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        while self
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    END_ENVIRONMENT_NAME
                        | R_CURLY
                        | PART_NAME
                        | CHAPTER_NAME
                        | SECTION_NAME
                        | SUBSECTION_NAME
                )
            })
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.builder.finish_node();
    }

    fn subsubsection(&mut self) {
        self.builder.start_node(SUBSUBSECTION.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(L_CURLY) {
            self.curly_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        while self
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    END_ENVIRONMENT_NAME
                        | R_CURLY
                        | PART_NAME
                        | CHAPTER_NAME
                        | SECTION_NAME
                        | SUBSECTION_NAME
                        | SUBSUBSECTION_NAME
                )
            })
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.builder.finish_node();
    }

    fn paragraph(&mut self) {
        self.builder.start_node(PARAGRAPH.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(L_CURLY) {
            self.curly_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        while self
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    END_ENVIRONMENT_NAME
                        | R_CURLY
                        | PART_NAME
                        | CHAPTER_NAME
                        | SECTION_NAME
                        | SUBSECTION_NAME
                        | SUBSUBSECTION_NAME
                        | PARAGRAPH_NAME
                )
            })
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.builder.finish_node();
    }

    fn subparagraph(&mut self) {
        self.builder.start_node(SUBPARAGRAPH.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(L_CURLY) {
            self.curly_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        while self
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    END_ENVIRONMENT_NAME
                        | R_CURLY
                        | PART_NAME
                        | CHAPTER_NAME
                        | SECTION_NAME
                        | SUBSECTION_NAME
                        | SUBSUBSECTION_NAME
                        | PARAGRAPH_NAME
                        | SUBPARAGRAPH_NAME
                )
            })
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.builder.finish_node();
    }

    fn enum_item(&mut self) {
        self.builder.start_node(ENUM_ITEM.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(L_BRACK) {
            self.brack_group();
        }

        while self
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    END_ENVIRONMENT_NAME
                        | R_CURLY
                        | PART_NAME
                        | CHAPTER_NAME
                        | SECTION_NAME
                        | SUBSECTION_NAME
                        | SUBSUBSECTION_NAME
                        | PARAGRAPH_NAME
                        | SUBPARAGRAPH_NAME
                        | ENUM_ITEM_NAME
                )
            })
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.builder.finish_node();
    }

    fn block_comment(&mut self) {
        self.builder.start_node(BLOCK_COMMENT.into());
        self.eat();

        if self.peek() == Some(VERBATIM) {
            self.eat();
        }

        if self.peek() == Some(END_BLOCK_COMMENT_NAME) {
            self.eat();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn caption(&mut self) {
        self.builder.start_node(CAPTION.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(L_BRACK) {
            self.brack_group();
        }

        if self.peek() == Some(L_CURLY) {
            self.curly_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn citation(&mut self) {
        self.builder.start_node(CITATION.into());
        self.eat();
        self.trivia();
        for _ in 0..2 {
            if self.lexer.peek() == Some(L_BRACK) {
                self.brack_group();
            }
        }

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word_list();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn generic_include(&mut self, kind: SyntaxKind, options: bool) {
        self.builder.start_node(kind.into());
        self.eat();
        self.trivia();
        if options && self.lexer.peek() == Some(L_BRACK) {
            self.brack_group_key_value();
        }

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_path_list();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn curly_group_path(&mut self) {
        self.builder.start_node(CURLY_GROUP_WORD.into());
        self.eat();
        while matches!(self.peek(), Some(WORD)) {
            self.path();
        }

        self.expect(R_CURLY);
        self.builder.finish_node();
    }

    fn curly_group_path_list(&mut self) {
        self.builder.start_node(CURLY_GROUP_WORD_LIST.into());
        self.eat();

        while self
            .peek()
            .filter(|&kind| {
                matches!(
                    kind,
                    LINE_BREAK | WHITESPACE | COMMENT | WORD | COMMA | EQUALITY_SIGN
                )
            })
            .is_some()
        {
            if self.peek() == Some(WORD) {
                self.path();
            } else {
                self.eat();
            }
        }

        self.expect(R_CURLY);
        self.builder.finish_node();
    }

    fn path(&mut self) {
        self.builder.start_node(KEY.into());
        self.eat();
        while self
            .peek()
            .filter(|&kind| matches!(kind, WHITESPACE | COMMENT | WORD | EQUALITY_SIGN))
            .is_some()
        {
            self.eat();
        }

        self.trivia();
        self.builder.finish_node();
    }

    fn package_include(&mut self) {
        self.generic_include(PACKAGE_INCLUDE, true);
    }

    fn class_include(&mut self) {
        self.generic_include(CLASS_INCLUDE, true);
    }

    fn latex_include(&mut self) {
        self.generic_include(LATEX_INCLUDE, false);
    }

    fn biblatex_include(&mut self) {
        self.generic_include(BIBLATEX_INCLUDE, true);
    }

    fn bibtex_include(&mut self) {
        self.generic_include(BIBTEX_INCLUDE, false);
    }

    fn graphics_include(&mut self) {
        self.generic_include(GRAPHICS_INCLUDE, true);
    }

    fn svg_include(&mut self) {
        self.generic_include(SVG_INCLUDE, true);
    }

    fn inkscape_include(&mut self) {
        self.generic_include(INKSCAPE_INCLUDE, true);
    }

    fn verbatim_include(&mut self) {
        self.generic_include(VERBATIM_INCLUDE, false);
    }

    fn import(&mut self) {
        self.builder.start_node(IMPORT.into());
        self.eat();
        self.trivia();

        for _ in 0..2 {
            if self.lexer.peek() == Some(L_CURLY) {
                self.curly_group_word();
            } else {
                self.builder.token(MISSING.into(), "");
            }
        }

        self.builder.finish_node();
    }

    fn label_definition(&mut self) {
        self.builder.start_node(LABEL_DEFINITION.into());
        self.eat();
        self.trivia();
        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }
        self.builder.finish_node();
    }

    fn label_reference(&mut self) {
        self.builder.start_node(LABEL_REFERENCE.into());
        self.eat();
        self.trivia();
        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word_list();
        } else {
            self.builder.token(MISSING.into(), "");
        }
        self.builder.finish_node();
    }

    fn label_reference_range(&mut self) {
        self.builder.start_node(LABEL_REFERENCE_RANGE.into());
        self.eat();
        self.trivia();

        for _ in 0..2 {
            if self.lexer.peek() == Some(L_CURLY) {
                self.curly_group_word();
            } else {
                self.builder.token(MISSING.into(), "");
            }
        }

        self.builder.finish_node();
    }

    fn label_number(&mut self) {
        self.builder.start_node(LABEL_NUMBER.into());
        self.eat();
        self.trivia();
        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group();
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn command_definition(&mut self) {
        self.builder.start_node(COMMAND_DEFINITION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_command();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.lexer.peek() == Some(L_BRACK) {
            self.brack_group_word();

            if self.lexer.peek() == Some(L_BRACK) {
                self.brack_group();
            }
        }

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_impl();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn math_operator(&mut self) {
        self.builder.start_node(MATH_OPERATOR.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_command();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_impl();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn glossary_entry_definition(&mut self) {
        self.builder.start_node(GLOSSARY_ENTRY_DEFINITION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_key_value();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn glossary_entry_reference(&mut self) {
        self.builder.start_node(GLOSSARY_ENTRY_REFERENCE.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(L_BRACK) {
            self.brack_group_key_value();
        }

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn acronym_definition(&mut self) {
        self.builder.start_node(ACRONYM_DEFINITION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(L_BRACK) {
            self.brack_group_key_value();
        }

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        for _ in 0..2 {
            if self.lexer.peek() == Some(L_CURLY) {
                self.curly_group();
            } else {
                self.builder.token(MISSING.into(), "");
            }
        }

        self.builder.finish_node();
    }

    fn acronym_declaration(&mut self) {
        self.builder.start_node(ACRONYM_DECLARATION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_key_value();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn acronym_reference(&mut self) {
        self.builder.start_node(ACRONYM_REFERENCE.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(L_BRACK) {
            self.brack_group_key_value();
        }

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn theorem_definition(&mut self) {
        self.builder.start_node(THEOREM_DEFINITION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.lexer.peek() == Some(L_BRACK) {
            self.brack_group_word();
        }

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.lexer.peek() == Some(L_BRACK) {
            self.brack_group_word();
        }

        self.builder.finish_node();
    }

    fn color_reference(&mut self) {
        self.builder.start_node(COLOR_REFERENCE.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn color_definition(&mut self) {
        self.builder.start_node(COLOR_DEFINITION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn color_set_definition(&mut self) {
        self.builder.start_node(COLOR_SET_DEFINITION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(L_BRACK) {
            self.brack_group_word();
        }

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word_list();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        for _ in 0..3 {
            if self.lexer.peek() == Some(L_CURLY) {
                self.curly_group_word();
            } else {
                self.builder.token(MISSING.into(), "");
            }
        }

        self.builder.finish_node();
    }

    fn tikz_library_import(&mut self) {
        self.builder.start_node(TIKZ_LIBRARY_IMPORT.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word_list();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn environment_definition(&mut self) {
        self.builder.start_node(ENVIRONMENT_DEFINITION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(L_CURLY) {
            self.curly_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.lexer.peek() == Some(L_BRACK) {
            self.brack_group_word();
            if self.lexer.peek() == Some(L_BRACK) {
                self.brack_group();
            }
        }

        for _ in 0..2 {
            if self.lexer.peek() == Some(L_CURLY) {
                self.curly_group_without_environments();
            } else {
                self.builder.token(MISSING.into(), "");
            }
        }

        self.builder.finish_node();
    }

    fn graphics_path(&mut self) {
        self.builder.start_node(GRAPHICS_PATH.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(L_CURLY) {
            self.eat();
            self.trivia();

            while matches!(self.lexer.peek(), Some(L_CURLY)) {
                self.curly_group_path();
            }
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.expect(R_CURLY);
        self.builder.finish_node();
    }
}

pub fn parse(text: &str) -> Parse {
    Parser::new(text).parse()
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use crate::syntax::latex;

    use super::*;

    fn setup(text: &str) -> latex::SyntaxNode {
        latex::SyntaxNode::new_root(parse(&text.trim().replace("\r", "")).green)
    }

    #[test]
    fn test_empty() {
        assert_debug_snapshot!(setup(r#""#));
    }

    #[test]
    fn test_hello_world() {
        assert_debug_snapshot!(setup(r#"Hello World!"#));
    }

    #[test]
    fn test_generic_command_empty() {
        assert_debug_snapshot!(setup(r#"\foo"#));
    }

    #[test]
    fn test_generic_command_escape() {
        assert_debug_snapshot!(setup(r#"\#"#));
    }

    #[test]
    fn test_generic_command_args() {
        assert_debug_snapshot!(setup(r#"\foo{bar}[qux]"#));
    }

    #[test]
    fn test_inline() {
        assert_debug_snapshot!(setup(r#"$x \in [0, \infty)$"#));
    }

    #[test]
    fn test_inline_double_dollar() {
        assert_debug_snapshot!(setup(r#"$$x \in [0, \infty)$$"#));
    }

    #[test]
    fn test_brace_group_simple() {
        assert_debug_snapshot!(setup(r#"{hello world}"#));
    }

    #[test]
    fn test_brace_group_missing_end() {
        assert_debug_snapshot!(setup(r#"{hello world"#));
    }

    #[test]
    fn test_unmatched_braces() {
        assert_debug_snapshot!(setup(r#"}{"#));
    }

    #[test]
    fn test_unmatched_brackets() {
        assert_debug_snapshot!(setup(r#"]["#));
    }

    #[test]
    fn test_unmatched_brackets_with_group() {
        assert_debug_snapshot!(setup(r#"{][}"#));
    }

    #[test]
    fn test_escaped_brackets() {
        assert_debug_snapshot!(setup(r#"{[}{]}"#));
    }

    #[test]
    fn test_parameter() {
        assert_debug_snapshot!(setup(r#"#1"#));
    }

    #[test]
    fn test_parameter_error() {
        assert_debug_snapshot!(setup(r#"#"#));
    }

    #[test]
    fn test_environment_simple() {
        assert_debug_snapshot!(setup(r#"\begin{foo} Hello World \end{bar}"#));
    }

    #[test]
    fn test_environment_nested() {
        assert_debug_snapshot!(setup(r#"\begin{foo} \begin{qux} \end{baz} \end{bar}"#));
    }

    #[test]
    fn test_environment_nested_missing_braces() {
        assert_debug_snapshot!(setup(
            r#"\begin{foo \begin{qux Hello World \end{baz} \end{bar"#
        ));
    }

    #[test]
    fn test_structure_siblings() {
        assert_debug_snapshot!(setup(r#"\section{Foo} Foo \section{Bar} Bar"#));
    }

    #[test]
    fn test_structure_nested() {
        assert_debug_snapshot!(setup(
            r#"\part{1}\chapter{2}\section{3}\subsection{4}\subsubsection{5}\paragraph{6}\subparagraph{7}"#
        ));
    }

    #[test]
    fn test_structure_enum_item() {
        assert_debug_snapshot!(setup(
            r#"\begin{enumerate} \item 1 \item[2] 2 \item 3 \end{enumerate}"#
        ));
    }

    #[test]
    fn test_structure_invalid_nesting() {
        assert_debug_snapshot!(setup(r#"\section{Foo} \chapter{Bar}"#));
    }

    #[test]
    fn test_equation() {
        assert_debug_snapshot!(setup(r#"\[ foo bar \]"#));
    }

    #[test]
    fn test_equation_missing_end() {
        assert_debug_snapshot!(setup(r#"\begin{a} \[ foo bar \end{b}"#));
    }

    #[test]
    fn test_equation_missing_begin() {
        assert_debug_snapshot!(setup(r#"\begin{a} foo bar \] \end{b}"#));
    }

    #[test]
    fn test_caption_minimal() {
        assert_debug_snapshot!(setup(r#"\caption{Foo \Bar Baz}"#));
    }

    #[test]
    fn test_caption_minimal_error() {
        assert_debug_snapshot!(setup(r#"\caption{Foo \Bar Baz"#));
    }

    #[test]
    fn test_caption() {
        assert_debug_snapshot!(setup(r#"\caption[qux]{Foo \Bar Baz}"#));
    }

    #[test]
    fn test_caption_error() {
        assert_debug_snapshot!(setup(r#"\caption[qux]{Foo \Bar Baz"#));
    }

    #[test]
    fn test_caption_figure() {
        assert_debug_snapshot!(setup(r#"\begin{figure}\caption{Foo}\end{figure}"#));
    }

    #[test]
    fn test_citation_empty() {
        assert_debug_snapshot!(setup(r#"\cite{}"#));
    }

    #[test]
    fn test_citation_simple() {
        assert_debug_snapshot!(setup(r#"\cite{foo}"#));
    }

    #[test]
    fn test_citation_multiple_keys() {
        assert_debug_snapshot!(setup(r#"\cite{foo, bar}"#));
    }

    #[test]
    fn test_citation_star() {
        assert_debug_snapshot!(setup(r#"\nocite{*}"#));
    }

    #[test]
    fn test_citation_prenote() {
        assert_debug_snapshot!(setup(r#"\cite[foo]{bar}"#));
    }

    #[test]
    fn test_citation_prenote_postnote() {
        assert_debug_snapshot!(setup(r#"\cite[foo][bar]{baz}"#));
    }

    #[test]
    fn test_citation_missing_brace() {
        assert_debug_snapshot!(setup(r#"\cite{foo"#));
    }

    #[test]
    fn test_citation_redundant_comma() {
        assert_debug_snapshot!(setup(r#"\cite{,foo,}"#));
    }

    #[test]
    fn test_package_include_empty() {
        assert_debug_snapshot!(setup(r#"\usepackage{}"#));
    }

    #[test]
    fn test_package_include_simple() {
        assert_debug_snapshot!(setup(r#"\usepackage{amsmath}"#));
    }

    #[test]
    fn test_package_include_multiple() {
        assert_debug_snapshot!(setup(r#"\usepackage{amsmath, lipsum}"#));
    }

    #[test]
    fn test_package_include_options() {
        assert_debug_snapshot!(setup(r#"\usepackage[foo = bar, baz, qux]{amsmath}"#));
    }

    #[test]
    fn test_class_include_empty() {
        assert_debug_snapshot!(setup(r#"\documentclass{}"#));
    }

    #[test]
    fn test_class_include_simple() {
        assert_debug_snapshot!(setup(r#"\documentclass{article}"#));
    }

    #[test]
    fn test_class_include_options() {
        assert_debug_snapshot!(setup(r#"\documentclass[foo = bar, baz, qux]{article}"#));
    }

    #[test]
    fn test_latex_include_simple() {
        assert_debug_snapshot!(setup(r#"\include{foo/bar}"#));
    }

    #[test]
    fn test_latex_include_equality_sign() {
        assert_debug_snapshot!(setup(r#"\include{foo=bar}"#));
    }

    #[test]
    fn test_latex_input_simple() {
        assert_debug_snapshot!(setup(r#"\input{foo/bar.tex}"#));
    }

    #[test]
    fn test_biblatex_include_simple() {
        assert_debug_snapshot!(setup(r#"\addbibresource{foo/bar.bib}"#));
    }

    #[test]
    fn test_biblatex_include_options() {
        assert_debug_snapshot!(setup(r#"\addbibresource[foo=bar, baz]{foo/bar.bib}"#));
    }

    #[test]
    fn test_bibtex_include_simple() {
        assert_debug_snapshot!(setup(r#"\bibliography{foo/bar}"#));
    }

    #[test]
    fn test_graphics_include_simple() {
        assert_debug_snapshot!(setup(r#"\includegraphics{foo/bar.pdf}"#));
    }

    #[test]
    fn test_graphics_include_options() {
        assert_debug_snapshot!(setup(r#"\includegraphics[scale=.5]{foo/bar.pdf}"#));
    }

    #[test]
    fn test_graphics_include_complicated_options() {
        assert_debug_snapshot!(setup(r#"\includegraphics[width=0.5\textwidth]{}"#));
    }

    #[test]
    fn test_svg_include_simple() {
        assert_debug_snapshot!(setup(r#"\includesvg{foo/bar.svg}"#));
    }

    #[test]
    fn test_svg_include_options() {
        assert_debug_snapshot!(setup(r#"\includesvg[scale=.5]{foo/bar.svg}"#));
    }

    #[test]
    fn test_inkscape_include_simple() {
        assert_debug_snapshot!(setup(r#"\includesvg{foo/bar}"#));
    }

    #[test]
    fn test_inkscape_include_options() {
        assert_debug_snapshot!(setup(r#"\includesvg[scale=.5]{foo/bar}"#));
    }

    #[test]
    fn test_verbatim_include_simple() {
        assert_debug_snapshot!(setup(r#"\verbatiminput{foo/bar.txt}"#));
    }

    #[test]
    fn test_import_simple() {
        assert_debug_snapshot!(setup(r#"\import{foo}{bar}"#));
    }

    #[test]
    fn test_import_incomplete() {
        assert_debug_snapshot!(setup(r#"\import{foo"#));
    }

    #[test]
    fn test_label_definition_simple() {
        assert_debug_snapshot!(setup(r#"\label{foo}"#));
    }

    #[test]
    fn test_label_reference_simple() {
        assert_debug_snapshot!(setup(r#"\ref{foo}"#));
    }

    #[test]
    fn test_label_reference_multiple() {
        assert_debug_snapshot!(setup(r#"\ref{foo, bar}"#));
    }

    #[test]
    fn test_label_reference_incomplete() {
        assert_debug_snapshot!(setup(r#"Equation \eqref{eq is a \emph{useful} identity."#));
    }

    #[test]
    fn test_equation_label_reference_simple() {
        assert_debug_snapshot!(setup(r#"\eqref{foo}"#));
    }

    #[test]
    fn test_label_reference_range_simple() {
        assert_debug_snapshot!(setup(r#"\crefrange{foo}{bar}"#));
    }

    #[test]
    fn test_label_reference_range_incomplete() {
        assert_debug_snapshot!(setup(r#"\crefrange{foo}"#));
    }

    #[test]
    fn test_label_reference_range_error() {
        assert_debug_snapshot!(setup(r#"\crefrange{foo{bar}"#));
    }

    #[test]
    fn test_label_number() {
        assert_debug_snapshot!(setup(r#"\newlabel{foo}{{1.1}}"#));
    }

    #[test]
    fn test_command_definition_simple() {
        assert_debug_snapshot!(setup(r#"\newcommand[1]{\id}{#1}"#));
    }

    #[test]
    fn test_command_definition_optional() {
        assert_debug_snapshot!(setup(r#"\newcommand{\foo}[1][def]{#1}"#));
    }

    #[test]
    fn test_command_definition_no_argc() {
        assert_debug_snapshot!(setup(r#"\newcommand{\foo}{foo}"#));
    }

    #[test]
    fn test_command_definition_no_impl() {
        assert_debug_snapshot!(setup(r#"\newcommand{\foo}"#));
    }

    #[test]
    fn test_command_definition_no_impl_error() {
        assert_debug_snapshot!(setup(r#"\newcommand{\foo"#));
    }

    #[test]
    fn test_command_definition_with_begin() {
        assert_debug_snapshot!(setup(
            r#"\newcommand{\CVSubHeadingListStart}{\begin{itemize}[leftmargin=0.5cm, label={}]}"#
        ));
    }

    #[test]
    fn test_math_operator_simple() {
        assert_debug_snapshot!(setup(r#"\DeclareMathOperator{\foo}{foo}"#));
    }

    #[test]
    fn test_math_operator_no_impl() {
        assert_debug_snapshot!(setup(r#"\DeclareMathOperator{\foo}"#));
    }

    #[test]
    fn test_glossary_entry_definition_simple() {
        assert_debug_snapshot!(setup(r#"\newglossaryentry{foo}{bar = baz, qux,}"#));
    }

    #[test]
    fn test_glossary_entry_reference_simple() {
        assert_debug_snapshot!(setup(r#"\gls{foo}"#));
    }

    #[test]
    fn test_glossary_entry_reference_options() {
        assert_debug_snapshot!(setup(r#"\gls[foo = bar, qux]{baz}"#));
    }

    #[test]
    fn test_acroynm_definition_simple() {
        assert_debug_snapshot!(setup(r#"\newacronym{fpsLabel}{FPS}{Frame per Second}"#));
    }

    #[test]
    fn test_acroynm_definition_options() {
        assert_debug_snapshot!(setup(
            r#"\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}"#
        ));
    }

    #[test]
    fn test_acroynm_reference_simple() {
        assert_debug_snapshot!(setup(r#"\acrshort{fpsLabel}"#));
    }

    #[test]
    fn test_acroynm_reference_options() {
        assert_debug_snapshot!(setup(r#"\acrshort[foo=bar,baz]{fpsLabel}"#));
    }

    #[test]
    fn test_theorem_definition_only_name() {
        assert_debug_snapshot!(setup(r#"\newtheorem{foo}"#));
    }

    #[test]
    fn test_theorem_definition_name_with_description() {
        assert_debug_snapshot!(setup(r#"\newtheorem{foo}{Foo}"#));
    }

    #[test]
    fn test_theorem_definition_name_with_description_and_counter() {
        assert_debug_snapshot!(setup(r#"\newtheorem{foo}[bar]{Foo}"#));
    }

    #[test]
    fn test_theorem_definition_name_with_counter() {
        assert_debug_snapshot!(setup(r#"\newtheorem{foo}[bar]"#));
    }

    #[test]
    fn test_theorem_definition_full() {
        assert_debug_snapshot!(setup(r#"\newtheorem{foo}[bar]{Foo}[baz]"#));
    }

    #[test]
    fn test_color_reference_simple() {
        assert_debug_snapshot!(setup(r#"\color{black}"#));
    }

    #[test]
    fn test_color_definition_simple() {
        assert_debug_snapshot!(setup(r#"\definecolor{foo}{rgb}{255,168,0}"#));
    }

    #[test]
    fn test_color_set_definition_simple() {
        assert_debug_snapshot!(setup(r#"\definecolorset[ty]{rgb,HTML}{foo}{bar}{baz}"#));
    }

    #[test]
    fn test_color_set_definition_error1() {
        assert_debug_snapshot!(setup(r#"\definecolorset[ty]{rgb,HTML}{foo}{bar}"#));
    }

    #[test]
    fn test_color_set_definition_error2() {
        assert_debug_snapshot!(setup(r#"\definecolorset{rgb,HTML}{foo}"#));
    }

    #[test]
    fn test_color_set_definition_error3() {
        assert_debug_snapshot!(setup(r#"\definecolorset{rgb,HTML}"#));
    }

    #[test]
    fn test_color_set_definition_error4() {
        assert_debug_snapshot!(setup(r#"\definecolorset"#));
    }

    #[test]
    fn test_pgf_library_import_simple() {
        assert_debug_snapshot!(setup(r#"\usepgflibrary{foo}"#));
    }

    #[test]
    fn test_tikz_library_import_simple() {
        assert_debug_snapshot!(setup(r#"\usetikzlibrary{foo}"#));
    }

    #[test]
    fn test_environment_definition() {
        assert_debug_snapshot!(setup(r#"\newenvironment{bar}[1]{\begin{foo}}{\end{foo}}"#));
    }

    #[test]
    fn test_environment_definition_optional_arg() {
        assert_debug_snapshot!(setup(r#"\newenvironment{foo}[1][default]{begin}{end}"#));
    }

    #[test]
    fn test_acronym_declaration() {
        assert_debug_snapshot!(setup(
            r#"\DeclareAcronym{eg}{short = e.g,long = for example,tag = abbrev}"#
        ));
    }

    #[test]
    fn test_label_definition_line_break() {
        assert_debug_snapshot!(setup("\\label{hello\nworld}"));
    }

    #[test]
    fn test_block_comments() {
        assert_debug_snapshot!(setup(
            r#"Foo
\iffalse
Test1
\fi
Bar
\iffalse
\fii
\fi
Baz"#
        ));
    }

    #[test]
    fn test_asymptote() {
        assert_debug_snapshot!(setup(
            r#"\begin{asy}
    printf("Hello World\n");
\end{asy}"#
        ));
    }

    #[test]
    fn test_graphics_path() {
        assert_debug_snapshot!(setup(r#"\graphicspath{{../figures/}}"#));
    }
}
