mod lexer;

use rowan::{GreenNode, GreenNodeBuilder};
use syntax::latex::SyntaxKind::{self, *};

use crate::SyntaxConfig;

use self::lexer::{
    types::{CommandName, ParagraphLevel, SectionLevel, Token},
    Lexer,
};

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

#[derive(Debug, Clone, Copy)]
struct KeyOptions {
    allow_eq: bool,
    allow_parens: bool,
    allow_bracks: bool,
}

#[derive(Debug)]
struct Parser<'a> {
    lexer: Lexer<'a>,
    builder: GreenNodeBuilder<'static>,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str, config: &SyntaxConfig) -> Self {
        Self {
            lexer: Lexer::new(text, config),
            builder: GreenNodeBuilder::new(),
        }
    }

    fn eat(&mut self) {
        let (kind, text) = self.lexer.eat().unwrap();
        self.builder.token(kind.into(), text);
    }

    fn eat_remap(&mut self, kind: SyntaxKind) {
        let (_, text) = self.lexer.eat().unwrap();
        self.builder.token(kind.into(), text);
    }

    fn peek(&self) -> Option<Token> {
        self.lexer.peek()
    }

    fn expect(&mut self, kind: Token) {
        if self.peek() == Some(kind) {
            self.eat();
            self.trivia();
        }
    }

    fn expect2(&mut self, kind1: Token, kind2: Token) {
        if self
            .peek()
            .filter(|&kind| kind == kind1 || kind == kind2)
            .is_some()
        {
            self.eat();
            self.trivia();
        }
    }

    fn trivia(&mut self) {
        while self.peek().map_or(false, |kind| {
            matches!(
                kind,
                Token::LineBreak | Token::Whitespace | Token::LineComment
            )
        }) {
            self.eat();
        }
    }

    pub fn parse(mut self) -> GreenNode {
        self.builder.start_node(ROOT.into());
        self.preamble();
        while self.peek().is_some() {
            self.content(ParserContext::default());
        }
        self.builder.finish_node();
        self.builder.finish()
    }

    fn content(&mut self, context: ParserContext) {
        match self.peek().unwrap() {
            Token::LineBreak | Token::Whitespace | Token::LineComment => self.eat(),
            Token::LCurly if context.allow_environment => self.curly_group(),
            Token::LCurly => self.curly_group_without_environments(),
            Token::LBrack | Token::LParen => self.mixed_group(),
            Token::RCurly | Token::RBrack | Token::RParen => {
                self.builder.start_node(ERROR.into());
                self.eat();
                self.builder.finish_node();
            }
            Token::Pipe | Token::Word | Token::Comma => self.text(context),
            Token::Eq => self.eat(),
            Token::Dollar => self.formula(),
            Token::Href => self.eat(),
            Token::CommandName(name) => match name {
                CommandName::Generic => self.generic_command(),
                CommandName::BeginEnvironment if context.allow_environment => self.environment(),
                CommandName::BeginEnvironment => self.generic_command(),
                CommandName::EndEnvironment => self.generic_command(),
                CommandName::BeginEquation => self.equation(),
                CommandName::EndEquation => self.generic_command(),
                CommandName::Section(level) => self.section(level),
                CommandName::Paragraph(level) => self.paragraph(level),
                CommandName::EnumItem => self.enum_item(),
                CommandName::Caption => self.caption(),
                CommandName::Citation => self.citation(),
                CommandName::PackageInclude => self.package_include(),
                CommandName::ClassInclude => self.class_include(),
                CommandName::LatexInclude => self.latex_include(),
                CommandName::BiblatexInclude => self.biblatex_include(),
                CommandName::BibtexInclude => self.bibtex_include(),
                CommandName::GraphicsInclude => self.graphics_include(),
                CommandName::SvgInclude => self.svg_include(),
                CommandName::InkscapeInclude => self.inkscape_include(),
                CommandName::VerbatimInclude => self.verbatim_include(),
                CommandName::Import => self.import(),
                CommandName::LabelDefinition => self.label_definition(),
                CommandName::LabelReference => self.label_reference(),
                CommandName::LabelReferenceRange => self.label_reference_range(),
                CommandName::LabelNumber => self.label_number(),
                CommandName::OldCommandDefinition => self.old_command_definition(),
                CommandName::NewCommandDefinition => self.new_command_definition(),
                CommandName::MathOperator => self.math_operator(),
                CommandName::GlossaryEntryDefinition => self.glossary_entry_definition(),
                CommandName::GlossaryEntryReference => self.glossary_entry_reference(),
                CommandName::AcronymDefinition => self.acronym_definition(),
                CommandName::AcronymDeclaration => self.acronym_declaration(),
                CommandName::AcronymReference => self.acronym_reference(),
                CommandName::TheoremDefinitionAmsThm => self.theorem_definition_amsthm(),
                CommandName::TheoremDefinitionThmTools => self.theorem_definition_thmtools(),
                CommandName::ColorReference => self.color_reference(),
                CommandName::ColorDefinition => self.color_definition(),
                CommandName::ColorSetDefinition => self.color_set_definition(),
                CommandName::TikzLibraryImport => self.tikz_library_import(),
                CommandName::EnvironmentDefinition => self.environment_definition(),
                CommandName::BeginBlockComment => self.block_comment(),
                CommandName::EndBlockComment => self.generic_command(),
                CommandName::VerbatimBlock => self.verbatim_block(),
                CommandName::GraphicsPath => self.graphics_path(),
                CommandName::BibItem => self.bibitem(),
                CommandName::TocContentsLine => self.toc_contents_line(),
                CommandName::TocNumberLine => self.toc_number_line(),
            },
        }
    }

    fn text(&mut self, context: ParserContext) {
        self.builder.start_node(TEXT.into());
        self.eat();
        while self
            .peek()
            .filter(|&kind| {
                matches!(
                    kind,
                    Token::LineBreak
                        | Token::Whitespace
                        | Token::LineComment
                        | Token::Word
                        | Token::Pipe
                        | Token::Comma
                ) && (context.allow_comma || kind != Token::Comma)
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
            .filter(|&kind| !matches!(kind, Token::RCurly))
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.expect(Token::RCurly);
        self.builder.finish_node();
    }

    fn curly_group_impl(&mut self) {
        self.builder.start_node(CURLY_GROUP.into());
        self.eat();
        while let Some(kind) = self.peek() {
            match kind {
                Token::RCurly => break,
                Token::CommandName(CommandName::BeginEnvironment) => self.begin(),
                Token::CommandName(CommandName::EndEnvironment) => self.end(),
                _ => self.content(ParserContext::default()),
            };
        }
        self.expect(Token::RCurly);
        self.builder.finish_node();
    }

    fn curly_group_without_environments(&mut self) {
        self.builder.start_node(CURLY_GROUP.into());
        self.eat();
        while self
            .peek()
            .filter(|&kind| !matches!(kind, Token::RCurly))
            .is_some()
        {
            self.content(ParserContext {
                allow_environment: false,
                allow_comma: true,
            });
        }
        self.expect(Token::RCurly);
        self.builder.finish_node();
    }

    fn curly_group_word(&mut self) {
        self.builder.start_node(CURLY_GROUP_WORD.into());
        self.eat();
        self.trivia();
        match self.peek() {
            Some(Token::Word | Token::Pipe) => {
                self.key();
            }
            Some(Token::CommandName(_)) => {
                self.content(ParserContext::default());
            }
            Some(_) | None => {}
        }
        self.expect(Token::RCurly);
        self.builder.finish_node();
    }

    fn curly_group_word_list(&mut self) {
        self.builder.start_node(CURLY_GROUP_WORD_LIST.into());
        self.eat();

        while self
            .peek()
            .filter(|&kind| {
                matches!(
                    kind,
                    Token::LineBreak
                        | Token::Whitespace
                        | Token::LineComment
                        | Token::Word
                        | Token::Pipe
                        | Token::Comma
                )
            })
            .is_some()
        {
            if self.peek() == Some(Token::Word) {
                self.key();
            } else {
                self.eat();
            }
        }

        self.expect(Token::RCurly);
        self.builder.finish_node();
    }

    fn curly_group_command(&mut self) {
        self.builder.start_node(CURLY_GROUP_COMMAND.into());
        self.eat();
        self.trivia();
        if matches!(self.peek(), Some(Token::CommandName(_))) {
            self.eat();
            self.trivia();
        }

        self.expect(Token::RCurly);
        self.builder.finish_node();
    }

    fn brack_group(&mut self) {
        self.builder.start_node(BRACK_GROUP.into());
        self.eat();
        while self.peek().map_or(false, |kind| {
            !matches!(
                kind,
                Token::RCurly
                    | Token::RBrack
                    | Token::CommandName(CommandName::Section(_))
                    | Token::CommandName(CommandName::EnumItem)
                    | Token::CommandName(CommandName::EndEnvironment)
            )
        }) {
            self.content(ParserContext::default());
        }

        self.expect(Token::RBrack);
        self.builder.finish_node();
    }

    fn brack_group_word(&mut self) {
        self.builder.start_node(BRACK_GROUP_WORD.into());
        self.eat();
        self.trivia();
        match self.peek() {
            Some(Token::Word | Token::Pipe) => {
                self.key_with_opts(KeyOptions {
                    allow_eq: true,
                    allow_bracks: false,
                    allow_parens: true,
                });
            }
            Some(_) | None => {}
        }
        self.expect(Token::RBrack);
        self.builder.finish_node();
    }

    fn mixed_group(&mut self) {
        self.builder.start_node(MIXED_GROUP.into());
        self.eat();
        self.trivia();
        while self.peek().map_or(false, |kind| {
            !matches!(
                kind,
                Token::RCurly
                    | Token::RBrack
                    | Token::RParen
                    | Token::CommandName(CommandName::Section(_))
                    | Token::CommandName(CommandName::EnumItem)
                    | Token::CommandName(CommandName::EndEnvironment)
            )
        }) {
            self.content(ParserContext::default());
        }

        self.expect2(Token::RBrack, Token::RParen);
        self.builder.finish_node();
    }

    fn key(&mut self) {
        self.key_with_opts(KeyOptions {
            allow_eq: true,
            allow_parens: true,
            allow_bracks: true,
        });
    }

    fn key_with_opts(&mut self, options: KeyOptions) {
        self.builder.start_node(KEY.into());
        self.eat();
        while let Some(kind) = self.peek() {
            match kind {
                Token::Whitespace | Token::LineComment | Token::Word | Token::Pipe => self.eat(),
                Token::LBrack | Token::RBrack if options.allow_bracks => self.eat(),
                Token::LParen | Token::RParen if options.allow_parens => self.eat(),
                Token::Eq if options.allow_eq => self.eat(),
                _ => break,
            }
        }

        self.trivia();
        self.builder.finish_node();
    }

    fn value(&mut self) {
        self.builder.start_node(VALUE.into());
        while let Some(kind) = self.lexer.peek() {
            match kind {
                Token::Comma | Token::RBrack | Token::RCurly => break,
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
        self.key_with_opts(KeyOptions {
            allow_eq: false,
            allow_parens: true,
            allow_bracks: false,
        });

        if self.peek() == Some(Token::Eq) {
            self.eat();
            self.trivia();
            if self
                .peek()
                .filter(|&kind| {
                    !matches!(
                        kind,
                        Token::CommandName(CommandName::EndEnvironment)
                            | Token::RCurly
                            | Token::RBrack
                            | Token::RParen
                            | Token::Comma
                    )
                })
                .is_some()
            {
                self.value();
            }
        }

        self.builder.finish_node();
    }

    fn key_value_body(&mut self) {
        self.builder.start_node(KEY_VALUE_BODY.into());
        while let Some(kind) = self.peek() {
            match kind {
                Token::LineBreak | Token::Whitespace | Token::LineComment => self.eat(),
                Token::Word | Token::Pipe => {
                    self.key_value_pair();
                    if self.peek() == Some(Token::Comma) {
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

    fn group_key_value(&mut self, node_kind: SyntaxKind, right_kind: Token) {
        self.builder.start_node(node_kind.into());
        self.eat();
        self.trivia();
        self.key_value_body();
        self.expect(right_kind);
        self.builder.finish_node();
    }

    fn curly_group_key_value(&mut self) {
        self.group_key_value(CURLY_GROUP_KEY_VALUE, Token::RCurly);
    }

    fn brack_group_key_value(&mut self) {
        self.group_key_value(BRACK_GROUP_KEY_VALUE, Token::RBrack);
    }

    fn formula(&mut self) {
        self.builder.start_node(FORMULA.into());
        self.eat();
        self.trivia();
        while self
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    Token::RCurly | Token::CommandName(CommandName::EndEnvironment) | Token::Dollar
                )
            })
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.expect(Token::Dollar);
        self.builder.finish_node();
    }

    fn generic_command(&mut self) {
        self.builder.start_node(GENERIC_COMMAND.into());
        self.eat();
        while let Some(kind) = self.peek() {
            match kind {
                Token::LineBreak | Token::Whitespace | Token::LineComment => self.eat(),
                Token::LCurly => self.curly_group(),
                Token::LBrack | Token::LParen => self.mixed_group(),
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
            .filter(|&kind| {
                !matches!(
                    kind,
                    Token::CommandName(CommandName::EndEnvironment)
                        | Token::RCurly
                        | Token::CommandName(CommandName::EndEquation)
                )
            })
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.expect(Token::CommandName(CommandName::EndEquation));
        self.builder.finish_node();
    }

    fn begin(&mut self) {
        self.builder.start_node(BEGIN.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        if self.peek() == Some(Token::LBrack) {
            self.brack_group();
        }
        self.builder.finish_node();
    }

    fn end(&mut self) {
        self.builder.start_node(END.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        self.builder.finish_node();
    }

    fn environment(&mut self) {
        self.builder.start_node(ENVIRONMENT.into());
        self.begin();

        while self
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    Token::RCurly | Token::CommandName(CommandName::EndEnvironment)
                )
            })
            .is_some()
        {
            self.content(ParserContext::default());
        }

        if self.peek() == Some(Token::CommandName(CommandName::EndEnvironment)) {
            self.end();
        }

        self.builder.finish_node();
    }

    fn preamble(&mut self) {
        self.builder.start_node(PREAMBLE.into());
        while self
            .peek()
            .filter(|&kind| kind != Token::CommandName(CommandName::EndEnvironment))
            .is_some()
        {
            self.content(ParserContext::default());
        }
        self.builder.finish_node();
    }

    fn section(&mut self, level: SectionLevel) {
        let node_kind = match level {
            SectionLevel::Part => PART,
            SectionLevel::Chapter => CHAPTER,
            SectionLevel::Section => SECTION,
            SectionLevel::Subsection => SUBSECTION,
            SectionLevel::Subsubsection => SUBSUBSECTION,
        };

        self.builder.start_node(node_kind.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(Token::LCurly) {
            self.curly_group();
        }

        while let Some(kind) = self.peek() {
            match kind {
                Token::RCurly => break,
                Token::CommandName(CommandName::EndEnvironment) => break,
                Token::CommandName(CommandName::Section(nested)) if nested <= level => break,
                _ => self.content(ParserContext::default()),
            };
        }

        self.builder.finish_node();
    }

    fn paragraph(&mut self, level: ParagraphLevel) {
        let node_kind = match level {
            ParagraphLevel::Paragraph => PARAGRAPH,
            ParagraphLevel::Subparagraph => SUBPARAGRAPH,
        };

        self.builder.start_node(node_kind.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(Token::LCurly) {
            self.curly_group();
        }

        while let Some(kind) = self.peek() {
            match kind {
                Token::RCurly => break,
                Token::CommandName(CommandName::EndEnvironment) => break,
                Token::CommandName(CommandName::Section(_)) => break,
                Token::CommandName(CommandName::Paragraph(nested)) if nested <= level => break,
                _ => self.content(ParserContext::default()),
            }
        }

        self.builder.finish_node();
    }

    fn enum_item(&mut self) {
        self.builder.start_node(ENUM_ITEM.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(Token::LBrack) {
            self.brack_group();
        }

        while let Some(kind) = self.peek() {
            match kind {
                Token::CommandName(CommandName::EndEnvironment)
                | Token::RCurly
                | Token::CommandName(CommandName::Section(_))
                | Token::CommandName(CommandName::EnumItem) => break,
                _ => self.content(ParserContext::default()),
            }
        }

        self.builder.finish_node();
    }

    fn block_comment(&mut self) {
        self.builder.start_node(BLOCK_COMMENT.into());
        self.eat();

        while let Some(kind) = self.peek() {
            match kind {
                Token::CommandName(CommandName::BeginBlockComment) => {
                    self.block_comment();
                }
                Token::CommandName(CommandName::EndBlockComment) => {
                    self.eat();
                    break;
                }
                _ => {
                    self.eat();
                }
            }
        }

        self.builder.finish_node();
    }

    fn caption(&mut self) {
        self.builder.start_node(CAPTION.into());
        self.eat();
        self.trivia();

        if self.peek() == Some(Token::LBrack) {
            self.brack_group();
        }

        if self.peek() == Some(Token::LCurly) {
            self.curly_group();
        }

        self.builder.finish_node();
    }

    fn citation(&mut self) {
        self.builder.start_node(CITATION.into());
        self.eat();
        self.trivia();
        for _ in 0..2 {
            if self.lexer.peek() == Some(Token::LBrack) {
                self.brack_group();
            }
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word_list();
        }

        self.builder.finish_node();
    }

    fn generic_include(&mut self, kind: SyntaxKind, options: bool) {
        self.builder.start_node(kind.into());
        self.eat();
        self.trivia();
        if options && self.lexer.peek() == Some(Token::LBrack) {
            self.brack_group_key_value();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_path_list();
        }

        self.builder.finish_node();
    }

    fn curly_group_path(&mut self) {
        self.builder.start_node(CURLY_GROUP_WORD.into());
        self.eat();
        self.trivia();

        while let Some(kind) = self.lexer.peek() {
            match kind {
                Token::LineComment
                | Token::Word
                | Token::Eq
                | Token::Comma
                | Token::LBrack
                | Token::RBrack
                | Token::Dollar
                | Token::CommandName(CommandName::Generic) => self.path(),
                Token::LCurly => self.curly_group_path(),
                Token::Whitespace | Token::Pipe => self.eat(),
                _ => break,
            };
        }

        self.expect(Token::RCurly);
        self.builder.finish_node();
    }

    fn curly_group_path_list(&mut self) {
        self.builder.start_node(CURLY_GROUP_WORD_LIST.into());
        self.eat();
        self.trivia();

        while let Some(kind) = self.peek() {
            match kind {
                Token::Word
                | Token::Eq
                | Token::LBrack
                | Token::RBrack
                | Token::LParen
                | Token::RParen
                | Token::Dollar
                | Token::CommandName(CommandName::Generic) => self.path(),
                Token::Whitespace
                | Token::LineBreak
                | Token::LineComment
                | Token::Comma
                | Token::Pipe => self.eat(),
                Token::LCurly => self.curly_group_path(),
                _ => break,
            };
        }

        self.expect(Token::RCurly);
        self.builder.finish_node();
    }

    fn path(&mut self) {
        self.builder.start_node(KEY.into());
        self.eat();

        while let Some(kind) = self.peek() {
            match kind {
                Token::Whitespace
                | Token::LineComment
                | Token::Word
                | Token::Eq
                | Token::LBrack
                | Token::RBrack
                | Token::LParen
                | Token::RParen
                | Token::Dollar
                | Token::CommandName(CommandName::Generic) => self.eat(),
                Token::LCurly => self.curly_group_path(),
                _ => break,
            };
        }

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
            if self.lexer.peek() == Some(Token::LCurly) {
                self.curly_group_word();
            }
        }

        self.builder.finish_node();
    }

    fn label_definition(&mut self) {
        self.builder.start_node(LABEL_DEFINITION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LBrack) {
            self.brack_group();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.builder.start_node(CURLY_GROUP_WORD.into());
            self.eat();
            self.trivia();

            if self.peek() == Some(Token::Word) || self.peek() == Some(Token::Pipe) {
                self.key();
            }

            if let Some(Token::CommandName(_)) = self.peek() {
                self.content(ParserContext::default());
            }

            self.expect(Token::RCurly);
            self.builder.finish_node();
        }

        self.builder.finish_node();
    }

    fn label_reference(&mut self) {
        self.builder.start_node(LABEL_REFERENCE.into());
        self.eat();
        self.trivia();
        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word_list();
        }

        self.builder.finish_node();
    }

    fn label_reference_range(&mut self) {
        self.builder.start_node(LABEL_REFERENCE_RANGE.into());
        self.eat();
        self.trivia();

        for _ in 0..2 {
            if self.lexer.peek() == Some(Token::LCurly) {
                self.curly_group_word();
            }
        }

        self.builder.finish_node();
    }

    fn label_number(&mut self) {
        self.builder.start_node(LABEL_NUMBER.into());
        self.eat();
        self.trivia();
        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group();
        }

        self.builder.finish_node();
    }

    fn old_command_definition(&mut self) {
        self.builder.start_node(OLD_COMMAND_DEFINITION.into());
        self.eat();
        self.trivia();

        if let Some(Token::CommandName(_)) = self.lexer.peek() {
            self.eat();
            self.trivia();
        }

        self.builder.finish_node();
    }

    fn new_command_definition(&mut self) {
        self.builder.start_node(NEW_COMMAND_DEFINITION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_command();
        }

        if self.lexer.peek() == Some(Token::LBrack) {
            self.brack_group_word();

            if self.lexer.peek() == Some(Token::LBrack) {
                self.brack_group();
            }
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_impl();
        }

        self.builder.finish_node();
    }

    fn math_operator(&mut self) {
        self.builder.start_node(MATH_OPERATOR.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_command();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_impl();
        }

        self.builder.finish_node();
    }

    fn glossary_entry_definition(&mut self) {
        self.builder.start_node(GLOSSARY_ENTRY_DEFINITION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_key_value();
        }

        self.builder.finish_node();
    }

    fn glossary_entry_reference(&mut self) {
        self.builder.start_node(GLOSSARY_ENTRY_REFERENCE.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LBrack) {
            self.brack_group_key_value();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        self.builder.finish_node();
    }

    fn acronym_definition(&mut self) {
        self.builder.start_node(ACRONYM_DEFINITION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LBrack) {
            self.brack_group_key_value();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        if self.lexer.peek() == Some(Token::LBrack) {
            self.brack_group();
        }

        for _ in 0..2 {
            if self.lexer.peek() == Some(Token::LCurly) {
                self.curly_group();
            }
        }

        self.builder.finish_node();
    }

    fn acronym_declaration(&mut self) {
        self.builder.start_node(ACRONYM_DECLARATION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_key_value();
        }

        self.builder.finish_node();
    }

    fn acronym_reference(&mut self) {
        self.builder.start_node(ACRONYM_REFERENCE.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LBrack) {
            self.brack_group_key_value();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        self.builder.finish_node();
    }

    fn theorem_definition_amsthm(&mut self) {
        self.builder.start_node(THEOREM_DEFINITION_AMSTHM.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        if self.lexer.peek() == Some(Token::LBrack) {
            self.brack_group_word();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group();
        }

        if self.lexer.peek() == Some(Token::LBrack) {
            self.brack_group_word();
        }

        self.builder.finish_node();
    }

    fn theorem_definition_thmtools(&mut self) {
        self.builder.start_node(THEOREM_DEFINITION_THMTOOLS.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LBrack) {
            self.brack_group_key_value();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word_list();
        }

        self.builder.finish_node();
    }

    fn color_reference(&mut self) {
        self.builder.start_node(COLOR_REFERENCE.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        self.builder.finish_node();
    }

    fn color_definition(&mut self) {
        self.builder.start_node(COLOR_DEFINITION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group();
        }

        self.builder.finish_node();
    }

    fn color_set_definition(&mut self) {
        self.builder.start_node(COLOR_SET_DEFINITION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LBrack) {
            self.brack_group_word();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word_list();
        }

        for _ in 0..3 {
            if self.lexer.peek() == Some(Token::LCurly) {
                self.curly_group_word();
            }
        }

        self.builder.finish_node();
    }

    fn tikz_library_import(&mut self) {
        self.builder.start_node(TIKZ_LIBRARY_IMPORT.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word_list();
        }

        self.builder.finish_node();
    }

    fn environment_definition(&mut self) {
        self.builder.start_node(ENVIRONMENT_DEFINITION.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        if self.lexer.peek() == Some(Token::LBrack) {
            self.brack_group_word();
            if self.lexer.peek() == Some(Token::LBrack) {
                self.brack_group();
            }
        }

        for _ in 0..2 {
            if self.lexer.peek() == Some(Token::LCurly) {
                self.curly_group_without_environments();
            }
        }

        self.builder.finish_node();
    }

    fn graphics_path(&mut self) {
        self.builder.start_node(GRAPHICS_PATH.into());
        self.eat();
        self.trivia();

        let checkpoint = self.builder.checkpoint();
        if self.lexer.peek() == Some(Token::LCurly) {
            self.eat();
            self.trivia();

            if matches!(
                self.lexer.peek(),
                Some(
                    Token::Word
                        | Token::Eq
                        | Token::LBrack
                        | Token::RBrack
                        | Token::CommandName(CommandName::Generic)
                )
            ) {
                self.builder
                    .start_node_at(checkpoint, CURLY_GROUP_WORD.into());
                self.path();
            } else {
                self.builder.start_node_at(checkpoint, CURLY_GROUP.into());
                while matches!(self.lexer.peek(), Some(Token::LCurly)) {
                    self.curly_group_path();
                }
            }

            self.expect(Token::RCurly);
            self.builder.finish_node();
        }

        self.builder.finish_node();
    }

    fn verbatim_block(&mut self) {
        self.builder.start_node(GENERIC_COMMAND.into());
        self.eat();
        self.builder.finish_node();
        self.trivia();

        if self.peek() == Some(Token::Pipe) {
            self.eat_remap(SyntaxKind::VERBATIM);
            while let Some(kind) = self.peek() {
                self.eat_remap(SyntaxKind::VERBATIM);
                if kind == Token::Pipe {
                    break;
                }
            }
        }
    }

    fn bibitem(&mut self) {
        self.builder.start_node(BIBITEM.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group_word();
        }

        self.builder.finish_node();
    }

    fn toc_contents_line(&mut self) {
        self.builder.start_node(TOC_CONTENTS_LINE.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group();
        }

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group();
        }

        self.builder.finish_node();
    }

    fn toc_number_line(&mut self) {
        self.builder.start_node(TOC_NUMBER_LINE.into());
        self.eat();
        self.trivia();

        if self.lexer.peek() == Some(Token::LCurly) {
            self.curly_group();
        }

        self.builder.finish_node();
    }

}

pub fn parse_latex(text: &str, config: &SyntaxConfig) -> GreenNode {
    Parser::new(text, config).parse()
}

#[cfg(test)]
mod tests;
