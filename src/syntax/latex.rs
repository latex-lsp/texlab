use std::mem;

use logos::Logos;
use rowan::{GreenNode, GreenNodeBuilder};

pub use self::SyntaxKind::*;

use super::AstNode;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
    ERROR = 0,
    MISSING,

    WHITESPACE,
    COMMENT,
    L_BRACE,
    R_BRACE,
    L_BRACKET,
    R_BRACKET,
    L_PAREN,
    R_PAREN,
    PARAMETER,
    COMMA,
    EQUALITY_SIGN,
    WORD,
    DOLLAR,
    GENERIC_COMMAND_NAME,
    BEGIN_ENV,
    END_ENV,
    BEGIN_EQUATION,
    END_EQUATION,
    PART_COMMAND,
    CHAPTER_COMMAND,
    SECTION_COMMAND,
    SUBSECTION_COMMAND,
    SUBSUBSECTION_COMMAND,
    PARAGRAPH_COMMAND,
    SUBPARAGRAPH_COMMAND,
    ENUM_ITEM_COMMAND,
    CAPTION_COMMAND,
    CITATION_COMMAND,
    PACKAGE_INCLUDE_COMMAND,
    CLASS_INCLUDE_COMMAND,
    LATEX_INCLUDE_COMMAND,
    BIBLATEX_INCLUDE_COMMAND,
    BIBTEX_INCLUDE_COMMAND,
    GRAPHICS_INCLUDE_COMMAND,
    SVG_INCLUDE_COMMAND,
    INKSCAPE_INCLUDE_COMMAND,
    VERBATIM_INCLUDE_COMMAND,
    IMPORT_COMMAND,
    LABEL_DEFINITION_COMMAND,
    LABEL_REFERENCE_COMMAND,
    LABEL_REFERENCE_RANGE_COMMAND,
    LABEL_NUMBER_COMMAND,
    COMMAND_DEFINITION_COMMAND,
    MATH_OPERATOR_COMMAND,
    GLOSSARY_ENTRY_DEFINITION_COMMAND,
    GLOSSARY_ENTRY_REFERENCE_COMMAND,
    ACRONYM_DEFINITION_COMMAND,
    ACRONYM_REFERENCE_COMMAND,
    THEOREM_DEFINITION_COMMAND,
    COLOR_REFERENCE_COMMAND,
    COLOR_DEFINITION_COMMAND,
    COLOR_SET_DEFINITION_COMMAND,
    TIKZ_LIBRARY_IMPORT_COMMAND,

    PREAMBLE,
    TEXT,
    KEY,
    VALUE,
    KEY_VALUE_PAIR,
    KEY_VALUE_BODY,
    BRACE_GROUP,
    BRACE_GROUP_WORD,
    BRACE_GROUP_WORD_LIST,
    BRACE_GROUP_COMMAND,
    BRACE_GROUP_KEY_VALUE,
    BRACKET_GROUP,
    BRACKET_GROUP_WORD,
    BRACKET_GROUP_KEY_VALUE,
    PAREN_GROUP,
    MIXED_GROUP,
    GENERIC_COMMAND,
    ENVIRONMENT,
    BEGIN,
    END,
    EQUATION,
    PART,
    CHAPTER,
    SECTION,
    SUBSECTION,
    SUBSUBSECTION,
    PARAGRAPH,
    SUBPARAGRAPH,
    ENUM_ITEM,
    FORMULA,
    CAPTION,
    CITATION,
    PACKAGE_INCLUDE,
    CLASS_INCLUDE,
    LATEX_INCLUDE,
    BIBLATEX_INCLUDE,
    BIBTEX_INCLUDE,
    GRAPHICS_INCLUDE,
    SVG_INCLUDE,
    INKSCAPE_INCLUDE,
    VERBATIM_INCLUDE,
    IMPORT,
    LABEL_DEFINITION,
    LABEL_REFERENCE,
    LABEL_REFERENCE_RANGE,
    LABEL_NUMBER,
    COMMAND_DEFINITION,
    MATH_OPERATOR,
    GLOSSARY_ENTRY_DEFINITION,
    GLOSSARY_ENTRY_REFERENCE,
    ACRONYM_DEFINITION,
    ACRONYM_REFERENCE,
    THEOREM_DEFINITION,
    COLOR_REFERENCE,
    COLOR_DEFINITION,
    COLOR_SET_DEFINITION,
    TIKZ_LIBRARY_IMPORT,
    ROOT,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lang {}

impl rowan::Language for Lang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= ROOT as u16);
        unsafe { mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub struct Parse {
    pub green_node: GreenNode,
}

pub type SyntaxNode = rowan::SyntaxNode<Lang>;

pub type SyntaxToken = rowan::SyntaxToken<Lang>;

pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Logos)]
#[allow(non_camel_case_types)]
#[repr(u16)]
enum LogosToken {
    #[regex(r"\s+")]
    WHITESPACE = 2,

    #[regex(r"%[\r\n]*")]
    COMMENT,

    #[token("{")]
    L_BRACE,

    #[token("}")]
    R_BRACE,

    #[token("[")]
    L_BRACKET,

    #[token("]")]
    R_BRACKET,

    #[token("(")]
    L_PAREN,

    #[token(")")]
    R_PAREN,

    #[regex(r"#\d?")]
    PARAMETER,

    #[token(",")]
    COMMA,

    #[token("=")]
    EQUALITY_SIGN,

    #[regex(r"[^\s\\%\{\},\$\[\]\(\)=\#]+")]
    #[error]
    WORD,

    #[regex(r"\$\$?")]
    DOLLAR,

    #[regex(r"\\([^\r\n]|[@a-zA-Z]+\*?)?")]
    GENERIC_COMMAND_NAME,

    #[regex(r"\\begin")]
    BEGIN_ENV,

    #[regex(r"\\end")]
    END_ENV,

    #[regex(r"\\\[")]
    BEGIN_EQUATION,

    #[regex(r"\\\]")]
    END_EQUATION,

    #[regex(r"\\part\*?")]
    PART_COMMAND,

    #[regex(r"\\chapter\*?")]
    CHAPTER_COMMAND,

    #[regex(r"\\section\*?")]
    SECTION_COMMAND,

    #[regex(r"\\subsection\*?")]
    SUBSECTION_COMMAND,

    #[regex(r"\\subsubsection\*?")]
    SUBSUBSECTION_COMMAND,

    #[regex(r"\\paragraph\*?")]
    PARAGRAPH_COMMAND,

    #[regex(r"\\subparagraph\*?")]
    SUBPARAGRAPH_COMMAND,

    #[regex(r"\\item")]
    ENUM_ITEM_COMMAND,

    #[regex(r"\\caption")]
    CAPTION_COMMAND,

    #[regex(r"\\cite|\\cite\*|\\Cite|\\nocite|\\citet|\\citep|\\citet\*|\\citep\*|\\citeauthor|\\citeauthor\*|\\Citeauthor|\\Citeauthor\*|\\citetitle|\\citetitle\*|\\citeyear|\\citeyear\*|\\citedate|\\citedate\*|\\citeurl|\\fullcite|\\citeyearpar|\\citealt|\\citealp|\\citetext|\\parencite|\\parencite\*|\\Parencite|\\footcite|\\footfullcite|\\footcitetext|\\textcite|\\Textcite|\\smartcite|\\Smartcite|\\supercite|\\autocite|\\Autocite|\\autocite\*|\\Autocite\*|\\volcite|\\Volcite|\\pvolcite|\\Pvolcite|\\fvolcite|\\ftvolcite|\\svolcite|\\Svolcite|\\tvolcite|\\Tvolcite|\\avolcite|\\Avolcite|\\notecite|\\notecite|\\pnotecite|\\Pnotecite|\\fnotecite")]
    CITATION_COMMAND,

    #[regex(r"\\usepackage|\\RequirePackage")]
    PACKAGE_INCLUDE_COMMAND,

    #[regex(r"\\documentclass")]
    CLASS_INCLUDE_COMMAND,

    #[regex(r"\\include|\\subfileinclude|\\input|\\subfile")]
    LATEX_INCLUDE_COMMAND,

    #[regex(r"\\addbibresource")]
    BIBLATEX_INCLUDE_COMMAND,

    #[regex(r"\\bibliography")]
    BIBTEX_INCLUDE_COMMAND,

    #[regex(r"\\includegraphics")]
    GRAPHICS_INCLUDE_COMMAND,

    #[regex(r"\\includesvg")]
    SVG_INCLUDE_COMMAND,

    #[regex(r"\\includeinkscape")]
    INKSCAPE_INCLUDE_COMMAND,

    #[regex(r"\\verbatiminput|\\VerbatimInput")]
    VERBATIM_INCLUDE_COMMAND,

    #[regex(r"\\import|\\subimport|\\inputfrom|\\subimportfrom|\\includefrom|\\subincludefrom")]
    IMPORT_COMMAND,

    #[regex(r"\\label")]
    LABEL_DEFINITION_COMMAND,

    #[regex(r"\\ref|\\vref|\\Vref|\\autoref|\\pageref|\\cref|\\Cref|\\cref*|\\Cref*|\\namecref|\\nameCref|\\lcnamecref|\\namecrefs|\\nameCrefs|\\lcnamecrefs|\\labelcref|\\labelcpageref|\\eqref")]
    LABEL_REFERENCE_COMMAND,

    #[regex(r"\\crefrange\*?|\\Crefrange\*?")]
    LABEL_REFERENCE_RANGE_COMMAND,

    #[regex(r"\\newlabel")]
    LABEL_NUMBER_COMMAND,

    #[regex(r"\\newcommand|\\renewcommand|\\DeclareRobustCommand")]
    COMMAND_DEFINITION_COMMAND,

    #[regex(r"\\DeclareMathOperator\*?")]
    MATH_OPERATOR_COMMAND,

    #[regex(r"\\newglossaryentry")]
    GLOSSARY_ENTRY_DEFINITION_COMMAND,

    #[regex(r"\\gls|\\Gls|\\GLS|\\glspl|\\Glspl|\\GLSpl|\\glsdisp|\\glslink|\\glstext|\\Glstext|\\GLStext|\\glsfirst|\\Glsfirst|\\GLSfirst|\\glsplural|\\Glsplural|\\GLSplural|\\glsfirstplural|\\Glsfirstplural|\\GLSfirstplural|\\glsname|\\Glsname|\\GLSname|\\glssymbol|\\Glssymbol|\\glsdesc|\\Glsdesc|\\GLSdesc|\\glsuseri|\\Glsuseri|\\GLSuseri|\\glsuserii|\\Glsuserii|\\GLSuserii|\\glsuseriii|\\Glsuseriii|\\GLSuseriii|\\glsuseriv|\\Glsuseriv|\\GLSuseriv|\\glsuserv|\\Glsuserv|\\GLSuserv|\\glsuservi|\\Glsuservi|\\GLSuservi")]
    GLOSSARY_ENTRY_REFERENCE_COMMAND,

    #[regex(r"\\newacronym")]
    ACRONYM_DEFINITION_COMMAND,

    #[regex(r"\\acrshort|\\Acrshort|\\ACRshort|\\acrshortpl|\\Acrshortpl|\\ACRshortpl|\\acrlong|\\Acrlong|\\ACRlong|\\acrlongpl|\\Acrlongpl|\\ACRlongpl|\\acrfull|\\Acrfull|\\ACRfull|\\acrfullpl|\\Acrfullpl|\\ACRfullpl|\\acs|\\Acs|\\acsp|\\Acsp|\\acl|\\Acl|\\aclp|\\Aclp|\\acf|\\Acf|\\acfp|\\Acfp|\\ac|\\Ac|\\acp|\\glsentrylong|\\Glsentrylong|\\glsentrylongpl|\\Glsentrylongpl|\\glsentryshort|\\Glsentryshort|\\glsentryshortpl|\\Glsentryshortpl|\\glsentryfullpl|\\Glsentryfullpl")]
    ACRONYM_REFERENCE_COMMAND,

    #[regex(r"\\newtheorem|\\declaretheorem")]
    THEOREM_DEFINITION_COMMAND,

    #[regex(r"\\color|\\colorbox|\\textcolor|\\pagecolor")]
    COLOR_REFERENCE_COMMAND,

    #[regex(r"\\definecolor")]
    COLOR_DEFINITION_COMMAND,

    #[regex(r"\\definecolorset")]
    COLOR_SET_DEFINITION_COMMAND,

    #[regex(r"\\usepgflibrary|\\usetikzlibrary")]
    TIKZ_LIBRARY_IMPORT_COMMAND,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct TokenSource<'a> {
    tokens: Vec<(SyntaxKind, &'a str)>,
}

impl<'a> TokenSource<'a> {
    pub fn new(text: &'a str) -> Self {
        let mut tokens = Vec::new();
        let mut lexer = LogosToken::lexer(text);
        while let Some(kind) = lexer.next() {
            tokens.push((
                unsafe { mem::transmute::<LogosToken, SyntaxKind>(kind) },
                lexer.slice(),
            ));
        }
        tokens.reverse();
        Self { tokens }
    }

    pub fn peek(&self) -> Option<SyntaxKind> {
        self.tokens.last().map(|(kind, _)| *kind)
    }

    pub fn consume(&mut self) -> Option<(SyntaxKind, &'a str)> {
        self.tokens.pop()
    }
}

struct Parser<'a> {
    tokens: TokenSource<'a>,
    builder: GreenNodeBuilder<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: TokenSource<'a>) -> Self {
        Self {
            tokens,
            builder: GreenNodeBuilder::new(),
        }
    }

    fn consume(&mut self) {
        let (kind, text) = self.tokens.consume().unwrap();
        self.builder.token(kind.into(), text);
    }

    fn expect_or_missing(&mut self, kind: SyntaxKind) {
        if self.tokens.peek() == Some(kind) {
            self.consume();
            self.trivia();
        } else {
            self.builder.token(MISSING.into(), "");
        }
    }

    pub fn parse(mut self) -> Parse {
        self.builder.start_node(ROOT.into());
        self.preamble();
        while self.tokens.peek().is_some() {
            self.content();
        }
        self.builder.finish_node();
        let green_node = self.builder.finish();
        Parse { green_node }
    }

    fn content(&mut self) {
        match self.tokens.peek().unwrap() {
            WHITESPACE => self.consume(),
            COMMENT => self.consume(),
            L_BRACE => self.brace_group(),
            R_BRACE => {
                self.tokens.consume();
                self.builder.token(ERROR.into(), "}");
            }
            L_BRACKET | L_PAREN => self.mixed_group(),
            R_BRACKET => {
                self.tokens.consume();
                self.builder.token(ERROR.into(), "]");
            }
            R_PAREN => {
                self.tokens.consume();
                self.builder.token(ERROR.into(), ")");
            }
            PARAMETER => self.consume(),
            WORD | COMMA => self.text(),
            EQUALITY_SIGN => self.consume(),
            DOLLAR => self.formula(),
            GENERIC_COMMAND_NAME => self.generic_command(),
            BEGIN_ENV => self.environment(),
            END_ENV => self.generic_command(),
            BEGIN_EQUATION => self.equation(),
            END_EQUATION => self.generic_command(),
            PART_COMMAND => self.part(),
            CHAPTER_COMMAND => self.chapter(),
            SECTION_COMMAND => self.section(),
            SUBSECTION_COMMAND => self.subsection(),
            SUBSUBSECTION_COMMAND => self.subsubsection(),
            PARAGRAPH_COMMAND => self.paragraph(),
            SUBPARAGRAPH_COMMAND => self.subparagraph(),
            ENUM_ITEM_COMMAND => self.enum_item(),
            CAPTION_COMMAND => self.caption(),
            CITATION_COMMAND => self.citation(),
            PACKAGE_INCLUDE_COMMAND => self.package_include(),
            CLASS_INCLUDE_COMMAND => self.class_include(),
            LATEX_INCLUDE_COMMAND => self.latex_include(),
            BIBLATEX_INCLUDE_COMMAND => self.biblatex_include(),
            BIBTEX_INCLUDE_COMMAND => self.bibtex_include(),
            GRAPHICS_INCLUDE_COMMAND => self.graphics_include(),
            SVG_INCLUDE_COMMAND => self.svg_include(),
            INKSCAPE_INCLUDE_COMMAND => self.inkscape_include(),
            VERBATIM_INCLUDE_COMMAND => self.verbatim_include(),
            IMPORT_COMMAND => self.import(),
            LABEL_DEFINITION_COMMAND => self.label_definition(),
            LABEL_REFERENCE_COMMAND => self.label_reference(),
            LABEL_REFERENCE_RANGE_COMMAND => self.label_reference_range(),
            LABEL_NUMBER_COMMAND => self.label_number(),
            COMMAND_DEFINITION_COMMAND => self.command_definition(),
            MATH_OPERATOR_COMMAND => self.math_operator(),
            GLOSSARY_ENTRY_DEFINITION_COMMAND => self.glossary_entry_definition(),
            GLOSSARY_ENTRY_REFERENCE_COMMAND => self.glossary_entry_reference(),
            ACRONYM_DEFINITION_COMMAND => self.acronym_definition(),
            ACRONYM_REFERENCE_COMMAND => self.acronym_reference(),
            THEOREM_DEFINITION_COMMAND => self.theorem_definition(),
            COLOR_REFERENCE_COMMAND => self.color_reference(),
            COLOR_DEFINITION_COMMAND => self.color_definition(),
            COLOR_SET_DEFINITION_COMMAND => self.color_set_definition(),
            TIKZ_LIBRARY_IMPORT_COMMAND => self.tikz_library_import(),
            _ => unreachable!("{:#?}", self.tokens.peek().unwrap()),
        }
    }

    fn trivia(&mut self) {
        while self
            .tokens
            .peek()
            .filter(|&kind| matches!(kind, WHITESPACE | COMMENT))
            .is_some()
        {
            self.consume();
        }
    }

    fn preamble(&mut self) {
        self.builder.start_node(PREAMBLE.into());
        while self
            .tokens
            .peek()
            .filter(|&kind| !matches!(kind, BEGIN_ENV))
            .is_some()
        {
            self.content();
        }
        self.builder.finish_node();
    }

    fn text(&mut self) {
        self.builder.start_node(TEXT.into());
        self.consume();
        while self
            .tokens
            .peek()
            .filter(|&kind| matches!(kind, WHITESPACE | COMMENT | WORD | COMMA))
            .is_some()
        {
            self.consume();
        }
        self.builder.finish_node();
    }

    fn brace_group(&mut self) {
        self.builder.start_node(BRACE_GROUP.into());
        self.consume();
        while self
            .tokens
            .peek()
            .filter(|&kind| !matches!(kind, R_BRACE | END_ENV))
            .is_some()
        {
            self.content();
        }

        self.expect_or_missing(R_BRACE);
        self.builder.finish_node();
    }

    fn bracket_group(&mut self) {
        self.builder.start_node(BRACKET_GROUP.into());
        self.consume();
        while self
            .tokens
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    R_BRACE
                        | R_BRACKET
                        | END_ENV
                        | PART_COMMAND
                        | CHAPTER_COMMAND
                        | SECTION_COMMAND
                        | SUBSECTION_COMMAND
                        | SUBSUBSECTION_COMMAND
                        | ENUM_ITEM_COMMAND
                )
            })
            .is_some()
        {
            self.content();
        }

        self.expect_or_missing(R_BRACKET);
        self.builder.finish_node();
    }

    fn paren_group(&mut self) {
        self.builder.start_node(PAREN_GROUP.into());
        self.consume();
        while self
            .tokens
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    R_BRACE
                        | R_BRACKET
                        | R_PAREN
                        | END_ENV
                        | PART_COMMAND
                        | CHAPTER_COMMAND
                        | SECTION_COMMAND
                        | SUBSECTION_COMMAND
                        | SUBSUBSECTION_COMMAND
                        | ENUM_ITEM_COMMAND
                )
            })
            .is_some()
        {
            self.content();
        }

        self.expect_or_missing(R_PAREN);
        self.builder.finish_node();
    }

    fn mixed_group(&mut self) {
        self.builder.start_node(MIXED_GROUP.into());
        self.consume();
        while self
            .tokens
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    R_BRACE
                        | R_BRACKET
                        | R_PAREN
                        | END_ENV
                        | PART_COMMAND
                        | CHAPTER_COMMAND
                        | SECTION_COMMAND
                        | SUBSECTION_COMMAND
                        | SUBSUBSECTION_COMMAND
                        | ENUM_ITEM_COMMAND
                )
            })
            .is_some()
        {
            self.content();
        }

        if self
            .tokens
            .peek()
            .filter(|&kind| matches!(kind, R_BRACKET | R_PAREN))
            .is_some()
        {
            self.consume();
            self.trivia();
        } else {
            self.builder.token(MISSING.into(), "");
        }
        self.builder.finish_node();
    }

    fn brace_group_word(&mut self) {
        self.builder.start_node(BRACE_GROUP_WORD.into());
        self.consume();
        self.trivia();
        self.expect_or_missing(WORD);
        self.expect_or_missing(R_BRACE);
        self.builder.finish_node();
    }

    fn bracket_group_word(&mut self) {
        self.builder.start_node(BRACKET_GROUP_WORD.into());
        self.consume();
        self.trivia();
        self.expect_or_missing(WORD);
        self.expect_or_missing(R_BRACKET);
        self.builder.finish_node();
    }

    fn brace_group_word_list(&mut self) {
        self.builder.start_node(BRACE_GROUP_WORD_LIST.into());
        self.consume();
        while self
            .tokens
            .peek()
            .filter(|&kind| matches!(kind, WHITESPACE | COMMENT | WORD | COMMA))
            .is_some()
        {
            self.consume();
        }
        self.expect_or_missing(R_BRACE);
        self.builder.finish_node();
    }

    fn brace_group_command(&mut self) {
        self.builder.start_node(BRACE_GROUP_COMMAND.into());
        self.consume();
        self.trivia();
        self.expect_or_missing(GENERIC_COMMAND_NAME);
        self.expect_or_missing(R_BRACE);
        self.builder.finish_node();
    }

    fn key(&mut self) {
        self.builder.start_node(KEY.into());
        self.consume();
        while self
            .tokens
            .peek()
            .filter(|&kind| matches!(kind, WHITESPACE | COMMENT | WORD))
            .is_some()
        {
            self.consume();
        }
        self.builder.finish_node();
    }

    fn value(&mut self) {
        self.builder.start_node(VALUE.into());
        self.content();
        self.builder.finish_node();
    }

    fn key_value_pair(&mut self) {
        self.builder.start_node(KEY_VALUE_PAIR.into());
        self.key();
        if self.tokens.peek() == Some(EQUALITY_SIGN) {
            self.consume();
            self.trivia();
            if self
                .tokens
                .peek()
                .filter(|&kind| matches!(kind, END_ENV | R_BRACE | R_BRACKET | R_PAREN | COMMA))
                .is_some()
            {
                self.builder.token(MISSING.into(), "");
            } else {
                self.value();
            }
        }
        self.builder.finish_node();
    }

    fn key_value_body(&mut self) {
        self.builder.start_node(KEY_VALUE_BODY.into());
        while let Some(kind) = self.tokens.peek() {
            match kind {
                WHITESPACE | COMMENT => self.consume(),
                WORD => {
                    self.key_value_pair();

                    if self.tokens.peek() == Some(COMMA) {
                        self.consume();
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        self.builder.finish_node();
    }

    fn brace_group_key_value(&mut self) {
        self.builder.start_node(BRACE_GROUP_KEY_VALUE.into());
        self.consume();
        self.trivia();
        self.key_value_body();
        self.expect_or_missing(R_BRACE);
        self.builder.finish_node();
    }

    fn bracket_group_key_value(&mut self) {
        self.builder.start_node(BRACKET_GROUP_KEY_VALUE.into());
        self.consume();
        self.trivia();
        self.key_value_body();
        self.expect_or_missing(R_BRACKET);
        self.builder.finish_node();
    }

    fn formula(&mut self) {
        self.builder.start_node(FORMULA.into());
        self.consume();
        while self
            .tokens
            .peek()
            .filter(|&kind| !matches!(kind, R_BRACE | END_ENV | DOLLAR))
            .is_some()
        {
            self.content();
        }

        self.expect_or_missing(DOLLAR);
        self.builder.finish_node();
    }

    fn generic_command(&mut self) {
        self.builder.start_node(GENERIC_COMMAND.into());
        self.consume();
        while let Some(kind) = self.tokens.peek() {
            match kind {
                WHITESPACE | COMMENT => self.consume(),
                L_BRACE => self.brace_group(),
                L_BRACKET => self.bracket_group(),
                L_PAREN => self.paren_group(),
                _ => break,
            }
        }
        self.builder.finish_node();
    }

    fn equation(&mut self) {
        self.builder.start_node(EQUATION.into());
        self.consume();
        while self
            .tokens
            .peek()
            .filter(|&kind| !matches!(kind, END_ENV | R_BRACE | END_EQUATION))
            .is_some()
        {
            self.content();
        }

        self.expect_or_missing(END_EQUATION);
        self.builder.finish_node();
    }

    fn environment(&mut self) {
        self.builder.start_node(ENVIRONMENT.into());
        self.begin();
        while self
            .tokens
            .peek()
            .filter(|&kind| !matches!(kind, END_ENV))
            .is_some()
        {
            self.content();
        }

        if self.tokens.peek() == Some(END_ENV) {
            self.end();
        }

        self.builder.finish_node();
    }

    fn begin(&mut self) {
        self.builder.start_node(BEGIN.into());
        self.consume();
        self.trivia();
        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.tokens.peek() == Some(L_BRACKET) {
            self.bracket_group();
        }

        self.builder.finish_node();
    }

    fn end(&mut self) {
        self.builder.start_node(END.into());
        self.consume();
        self.trivia();
        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn part(&mut self) {
        self.builder.start_node(PART.into());
        self.consume();
        self.trivia();
        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        while self
            .tokens
            .peek()
            .filter(|&kind| !matches!(kind, END_ENV | R_BRACE | PART_COMMAND))
            .is_some()
        {
            self.content();
        }
        self.builder.finish_node();
    }

    fn chapter(&mut self) {
        self.builder.start_node(CHAPTER.into());
        self.consume();
        self.trivia();
        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        while self
            .tokens
            .peek()
            .filter(|&kind| !matches!(kind, END_ENV | R_BRACE | PART_COMMAND | CHAPTER_COMMAND))
            .is_some()
        {
            self.content();
        }
        self.builder.finish_node();
    }

    fn section(&mut self) {
        self.builder.start_node(SECTION.into());
        self.consume();
        self.trivia();
        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        while self
            .tokens
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    END_ENV | R_BRACE | PART_COMMAND | CHAPTER_COMMAND | SECTION_COMMAND
                )
            })
            .is_some()
        {
            self.content();
        }
        self.builder.finish_node();
    }

    fn subsection(&mut self) {
        self.builder.start_node(SUBSECTION.into());
        self.consume();
        self.trivia();
        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        while self
            .tokens
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    END_ENV | R_BRACE | PART_COMMAND | CHAPTER_COMMAND | SUBSECTION_COMMAND
                )
            })
            .is_some()
        {
            self.content();
        }
        self.builder.finish_node();
    }

    fn subsubsection(&mut self) {
        self.builder.start_node(SUBSUBSECTION.into());
        self.consume();
        self.trivia();
        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        while self
            .tokens
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    END_ENV
                        | R_BRACE
                        | PART_COMMAND
                        | CHAPTER_COMMAND
                        | SECTION_COMMAND
                        | SUBSECTION_COMMAND
                        | SUBSUBSECTION_COMMAND
                )
            })
            .is_some()
        {
            self.content();
        }
        self.builder.finish_node();
    }

    fn paragraph(&mut self) {
        self.builder.start_node(PARAGRAPH.into());
        self.consume();
        self.trivia();
        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        while self
            .tokens
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    END_ENV
                        | R_BRACE
                        | PART_COMMAND
                        | CHAPTER_COMMAND
                        | SECTION_COMMAND
                        | SUBSECTION_COMMAND
                        | SUBSUBSECTION_COMMAND
                        | PARAGRAPH_COMMAND
                )
            })
            .is_some()
        {
            self.content();
        }
        self.builder.finish_node();
    }

    fn subparagraph(&mut self) {
        self.builder.start_node(SUBPARAGRAPH.into());
        self.consume();
        self.trivia();
        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        while self
            .tokens
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    END_ENV
                        | R_BRACE
                        | PART_COMMAND
                        | CHAPTER_COMMAND
                        | SECTION_COMMAND
                        | SUBSECTION_COMMAND
                        | SUBSUBSECTION_COMMAND
                        | PARAGRAPH_COMMAND
                        | SUBPARAGRAPH_COMMAND
                )
            })
            .is_some()
        {
            self.content();
        }
        self.builder.finish_node();
    }

    fn enum_item(&mut self) {
        self.builder.start_node(ENUM_ITEM.into());
        self.consume();
        self.trivia();
        if self.tokens.peek() == Some(L_BRACKET) {
            self.bracket_group_word();
        }

        while self
            .tokens
            .peek()
            .filter(|&kind| {
                !matches!(
                    kind,
                    END_ENV
                        | R_BRACE
                        | PART_COMMAND
                        | CHAPTER_COMMAND
                        | SECTION_COMMAND
                        | SUBSECTION_COMMAND
                        | SUBSUBSECTION_COMMAND
                        | PARAGRAPH_COMMAND
                        | SUBPARAGRAPH_COMMAND
                        | ENUM_ITEM
                )
            })
            .is_some()
        {
            self.content();
        }
        self.builder.finish_node();
    }

    fn caption(&mut self) {
        self.builder.start_node(CAPTION.into());
        self.consume();
        self.trivia();
        if self.tokens.peek() == Some(L_BRACKET) {
            self.bracket_group();
        }

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn citation(&mut self) {
        self.builder.start_node(CITATION.into());
        self.consume();
        self.trivia();
        for _ in 0..2 {
            if self.tokens.peek() == Some(L_BRACKET) {
                self.bracket_group();
            }
        }

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word_list();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn generic_include(&mut self, kind: SyntaxKind, options: bool) {
        self.builder.start_node(kind.into());
        self.consume();
        self.trivia();
        if options && self.tokens.peek() == Some(L_BRACKET) {
            self.bracket_group_key_value();
        }

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word_list();
        } else {
            self.builder.token(MISSING.into(), "");
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
        self.consume();
        self.trivia();

        for _ in 0..2 {
            if self.tokens.peek() == Some(L_BRACE) {
                self.brace_group_word();
            } else {
                self.builder.token(MISSING.into(), "");
            }
        }

        self.builder.finish_node();
    }

    fn label_definition(&mut self) {
        self.builder.start_node(LABEL_DEFINITION.into());
        self.consume();
        self.trivia();
        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }
        self.builder.finish_node();
    }

    fn label_reference(&mut self) {
        self.builder.start_node(LABEL_REFERENCE.into());
        self.consume();
        self.trivia();
        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word_list();
        } else {
            self.builder.token(MISSING.into(), "");
        }
        self.builder.finish_node();
    }

    fn label_reference_range(&mut self) {
        self.builder.start_node(LABEL_REFERENCE_RANGE.into());
        self.consume();
        self.trivia();

        for _ in 0..2 {
            if self.tokens.peek() == Some(L_BRACE) {
                self.brace_group_word();
            } else {
                self.builder.token(MISSING.into(), "");
            }
        }

        self.builder.finish_node();
    }

    fn label_number(&mut self) {
        self.builder.start_node(LABEL_NUMBER.into());
        self.consume();
        self.trivia();
        if self.tokens.peek() == Some(L_BRACE) {
            self.bracket_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group();
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn command_definition(&mut self) {
        self.builder.start_node(COMMAND_DEFINITION.into());
        self.consume();
        self.trivia();

        if self.tokens.peek() == Some(L_BRACKET) {
            self.bracket_group_word();
        }

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_command();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group();
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn math_operator(&mut self) {
        self.builder.start_node(MATH_OPERATOR.into());
        self.consume();
        self.trivia();

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_command();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group();
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn glossary_entry_definition(&mut self) {
        self.builder.start_node(GLOSSARY_ENTRY_DEFINITION.into());
        self.consume();
        self.trivia();

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_key_value();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn glossary_entry_reference(&mut self) {
        self.builder.start_node(GLOSSARY_ENTRY_REFERENCE.into());
        self.consume();
        self.trivia();

        if self.tokens.peek() == Some(L_BRACKET) {
            self.bracket_group_key_value();
        }

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn acronym_definition(&mut self) {
        self.builder.start_node(ACRONYM_DEFINITION.into());
        self.consume();
        self.trivia();

        if self.tokens.peek() == Some(L_BRACKET) {
            self.bracket_group_key_value();
        }

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        for _ in 0..2 {
            if self.tokens.peek() == Some(L_BRACE) {
                self.brace_group();
            } else {
                self.builder.token(MISSING.into(), "");
            }
        }

        self.builder.finish_node();
    }

    fn acronym_reference(&mut self) {
        self.builder.start_node(ACRONYM_REFERENCE.into());
        self.consume();
        self.trivia();

        if self.tokens.peek() == Some(L_BRACKET) {
            self.bracket_group_key_value();
        }

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn theorem_definition(&mut self) {
        self.builder.start_node(THEOREM_DEFINITION.into());
        self.consume();
        self.trivia();

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.tokens.peek() == Some(L_BRACKET) {
            self.bracket_group_word();
        }

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.tokens.peek() == Some(L_BRACKET) {
            self.bracket_group_word();
        }

        self.builder.finish_node();
    }

    fn color_reference(&mut self) {
        self.builder.start_node(COLOR_REFERENCE.into());
        self.consume();
        self.trivia();

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn color_definition(&mut self) {
        self.builder.start_node(COLOR_DEFINITION.into());
        self.consume();
        self.trivia();

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }

    fn color_set_definition(&mut self) {
        self.builder.start_node(COLOR_SET_DEFINITION.into());
        self.consume();
        self.trivia();

        if self.tokens.peek() == Some(L_BRACKET) {
            self.brace_group_word();
        }

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word_list();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        for _ in 0..3 {
            if self.tokens.peek() == Some(L_BRACE) {
                self.brace_group_word();
            } else {
                self.builder.token(MISSING.into(), "");
            }
        }

        self.builder.finish_node();
    }

    fn tikz_library_import(&mut self) {
        self.builder.start_node(TIKZ_LIBRARY_IMPORT.into());
        self.consume();
        self.trivia();

        if self.tokens.peek() == Some(L_BRACE) {
            self.brace_group_word_list();
        } else {
            self.builder.token(MISSING.into(), "");
        }

        self.builder.finish_node();
    }
}

pub fn parse(text: &str) -> Parse {
    Parser::new(TokenSource::new(text)).parse()
}

macro_rules! ast_node {
    ($name:ident, $($kind:pat),+) => {
        #[derive(Debug, Clone)]
        #[repr(transparent)]
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            type Lang = Lang;

            fn cast(node: rowan::SyntaxNode<Self::Lang>) -> Option<Self>
            where
                Self: Sized,
            {
                match node.kind() {
                    $($kind => Some(Self(node)),)+
                    _ => None,
                }
            }

            fn syntax(&self) -> &rowan::SyntaxNode<Self::Lang> {
                &self.0
            }
        }
    };
}

ast_node!(Text, TEXT);

pub trait HasBraces: AstNode<Lang = Lang> {
    fn left_brace(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == L_BRACE.into())
    }

    fn right_brace(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == R_BRACE.into())
    }
}

ast_node!(BraceGroup, BRACE_GROUP);

impl HasBraces for BraceGroup {}

pub trait HasBrackets: AstNode<Lang = Lang> {
    fn left_bracket(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == L_BRACKET.into())
    }

    fn right_bracket(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == R_BRACKET.into())
    }
}

ast_node!(BracketGroup, BRACKET_GROUP);

impl HasBrackets for BracketGroup {}

pub trait HasParens: AstNode<Lang = Lang> {
    fn left_paren(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == L_PAREN.into())
    }

    fn right_paren(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == R_PAREN.into())
    }
}

ast_node!(ParenGroup, PAREN_GROUP);

impl HasParens for ParenGroup {}

ast_node!(MixedGroup, MIXED_GROUP);

impl MixedGroup {
    pub fn left_delim(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| matches!(node.kind().into(), L_BRACKET | L_PAREN))
    }

    pub fn right_delim(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| matches!(node.kind().into(), R_BRACKET | R_PAREN))
    }
}

ast_node!(BraceGroupWord, BRACE_GROUP_WORD);

impl HasBraces for BraceGroupWord {}

impl BraceGroupWord {
    pub fn word(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == WORD)
    }
}

ast_node!(BracketGroupWord, BRACKET_GROUP_WORD);

impl HasBrackets for BracketGroupWord {}

impl BracketGroupWord {
    pub fn word(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == WORD)
    }
}

ast_node!(BraceGroupWordList, BRACE_GROUP_WORD_LIST);

impl HasBraces for BraceGroupWordList {}

impl BraceGroupWordList {
    pub fn words(&self) -> impl Iterator<Item = SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .filter(|node| node.kind() == WORD.into())
    }
}

ast_node!(BraceGroupCommand, BRACE_GROUP_COMMAND);

impl HasBraces for BraceGroupCommand {}

impl BraceGroupCommand {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .find(|node| node.kind() == GENERIC_COMMAND_NAME)
    }
}

ast_node!(Key, KEY);

impl Key {
    pub fn words(&self) -> impl Iterator<Item = SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|node| node.into_token())
            .filter(|node| node.kind() == WORD.into())
    }
}

ast_node!(Value, VALUE);

ast_node!(KeyValuePair, KEY_VALUE_PAIR);

impl KeyValuePair {
    pub fn key(&self) -> Option<Key> {
        self.syntax().children().find_map(Key::cast)
    }

    pub fn value(&self) -> Option<Value> {
        self.syntax().children().find_map(Value::cast)
    }
}

ast_node!(KeyValueBody, KEY_VALUE_BODY);

impl KeyValueBody {
    pub fn pairs(&self) -> impl Iterator<Item = KeyValuePair> {
        self.syntax().children().filter_map(KeyValuePair::cast)
    }
}

pub trait HasKeyValueBody: AstNode<Lang = Lang> {
    fn body(&self) -> Option<KeyValueBody> {
        self.syntax().children().find_map(KeyValueBody::cast)
    }
}

ast_node!(BraceGroupKeyValue, BRACE_GROUP_KEY_VALUE);

impl HasBraces for BraceGroupKeyValue {}

impl HasKeyValueBody for BraceGroupKeyValue {}

ast_node!(BracketGroupKeyValue, BRACKET_GROUP_KEY_VALUE);

impl HasBrackets for BracketGroupKeyValue {}

impl HasKeyValueBody for BracketGroupKeyValue {}

ast_node!(Formula, FORMULA);

ast_node!(GenericCommand, GENERIC_COMMAND);

ast_node!(Equation, EQUATION);

ast_node!(Begin, BEGIN);

impl Begin {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<BraceGroupWord> {
        self.syntax().children().find_map(BraceGroupWord::cast)
    }

    pub fn options(&self) -> Option<BracketGroupKeyValue> {
        self.syntax()
            .children()
            .find_map(BracketGroupKeyValue::cast)
    }
}

ast_node!(End, END);

impl End {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<BraceGroupWord> {
        self.syntax().children().find_map(BraceGroupWord::cast)
    }
}

ast_node!(Environment, ENVIRONMENT);

impl Environment {
    pub fn begin(&self) -> Option<Begin> {
        self.syntax().children().find_map(Begin::cast)
    }

    pub fn end(&self) -> Option<End> {
        self.syntax().children().find_map(End::cast)
    }
}

ast_node!(
    Section,
    PART,
    CHAPTER,
    SECTION,
    SUBSECTION,
    SUBSUBSECTION,
    PARAGRAPH,
    SUBPARAGRAPH
);

impl Section {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn name(&self) -> Option<BraceGroup> {
        self.syntax().children().find_map(BraceGroup::cast)
    }
}

ast_node!(EnumItem, ENUM_ITEM);

impl EnumItem {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn label(&self) -> Option<BracketGroupWord> {
        self.syntax().children().find_map(BracketGroupWord::cast)
    }
}

ast_node!(Caption, CAPTION);

impl Caption {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn short(&self) -> Option<BracketGroup> {
        self.syntax().children().find_map(BracketGroup::cast)
    }

    pub fn long(&self) -> Option<BraceGroup> {
        self.syntax().children().find_map(BraceGroup::cast)
    }
}

ast_node!(Citation, CITATION);

impl Citation {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn prenote(&self) -> Option<BracketGroup> {
        self.syntax().children().find_map(BracketGroup::cast)
    }

    pub fn postnote(&self) -> Option<BracketGroup> {
        self.syntax()
            .children()
            .filter_map(BracketGroup::cast)
            .skip(1)
            .next()
    }

    pub fn key_list(&self) -> Option<BraceGroupWordList> {
        self.syntax().children().find_map(BraceGroupWordList::cast)
    }
}

ast_node!(
    Include,
    PACKAGE_INCLUDE,
    CLASS_INCLUDE,
    LATEX_INCLUDE,
    BIBLATEX_INCLUDE,
    BIBTEX_INCLUDE,
    GRAPHICS_INCLUDE,
    SVG_INCLUDE,
    INKSCAPE_INCLUDE,
    VERBATIM_INCLUDE
);

impl Include {
    pub fn path_list(&self) -> Option<BraceGroupWordList> {
        self.syntax().children().find_map(BraceGroupWordList::cast)
    }
}

ast_node!(Import, IMPORT);

impl Import {
    pub fn command(&self) -> Option<SyntaxToken> {
        self.syntax().first_token()
    }

    pub fn directory(&self) -> Option<BraceGroupWord> {
        self.syntax().children().find_map(BraceGroupWord::cast)
    }

    pub fn file(&self) -> Option<BraceGroupWord> {
        self.syntax()
            .children()
            .filter_map(BraceGroupWord::cast)
            .skip(1)
            .next()
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;

    fn setup(text: &str) -> SyntaxNode {
        SyntaxNode::new_root(parse(&text.trim().replace("\r", "")).green_node)
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
}
