use logos::Logos;

use super::kind::SyntaxKind;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Logos)]
#[allow(non_camel_case_types)]
#[repr(u16)]
enum RootToken {
    #[regex(r"[\r\n]+", priority = 2)]
    LineBreak = 2,

    #[regex(r"\s+", priority = 1)]
    Whitespace,

    #[regex(r"%[^\r\n]*")]
    LineComment,

    #[token("{")]
    LCurly,

    #[token("}")]
    RCurly,

    #[token("[")]
    LBrack,

    #[token("]")]
    RBrack,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token(",")]
    Comma,

    #[token("=")]
    Eq,

    #[regex(r"[^\s\\%\{\},\$\[\]\(\)=]+")]
    #[error]
    Word,

    #[regex(r"\$\$?")]
    Dollar,

    #[regex(r"\\([^\r\n]|[@a-zA-Z:_]+\*?)?")]
    CommandName,

    #[token("\\iffalse")]
    BeginBlockComment,

    #[token("\\begin{asy}")]
    #[token("\\begin{verbatim}")]
    #[token("\\begin{lstlisting}")]
    #[token("\\begin{minted}")]
    #[token("\\begin{pycode}")]
    BeginVerbatimEnvironment,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Logos)]
enum CommandNameToken {
    #[token("\\begin")]
    BeginEnvironment,

    #[token("\\end")]
    EndEnvironment,

    #[token("\\[")]
    BeginEquation,

    #[token("\\]")]
    EndEquation,

    #[token("\\part")]
    #[token("\\part*")]
    Part,

    #[token("\\chapter")]
    #[token("\\chapter*")]
    Chapter,

    #[token("\\section")]
    #[token("\\section*")]
    Section,

    #[token("\\subsection")]
    #[token("\\subsection*")]
    Subsection,

    #[token("\\subsubsection")]
    #[token("\\subsubsection*")]
    Subsubsection,

    #[token("\\paragraph")]
    #[token("\\paragraph*")]
    Paragraph,

    #[token("\\subparagraph")]
    #[token("\\subparagraph*")]
    Subparagraph,

    #[token("\\item")]
    EnumItem,

    #[token("\\caption")]
    Caption,

    #[token("\\cite")]
    #[token("\\cite*")]
    #[token("\\Cite")]
    #[token("\\nocite")]
    #[token("\\citet")]
    #[token("\\citet*")]
    #[token("\\citep")]
    #[token("\\citep*")]
    #[token("\\citeauthor")]
    #[token("\\citeauthor*")]
    #[token("\\Citeauthor")]
    #[token("\\Citeauthor*")]
    #[token("\\citetitle")]
    #[token("\\citetitle*")]
    #[token("\\citeyear")]
    #[token("\\citeyear*")]
    #[token("\\citedate")]
    #[token("\\citedate*")]
    #[token("\\citeurl")]
    #[token("\\fullcite")]
    #[token("\\citeyearpar")]
    #[token("\\citealt")]
    #[token("\\citealp")]
    #[token("\\citetext")]
    #[token("\\parencite")]
    #[token("\\parencite*")]
    #[token("\\Parencite")]
    #[token("\\footcite")]
    #[token("\\footfullcite")]
    #[token("\\footcitetext")]
    #[token("\\textcite")]
    #[token("\\Textcite")]
    #[token("\\smartcite")]
    #[token("\\supercite")]
    #[token("\\autocite")]
    #[token("\\autocite*")]
    #[token("\\Autocite")]
    #[token("\\Autocite*")]
    #[token("\\volcite")]
    #[token("\\Volcite")]
    #[token("\\pvolcite")]
    #[token("\\Pvolcite")]
    #[token("\\fvolcite")]
    #[token("\\ftvolcite")]
    #[token("\\svolcite")]
    #[token("\\Svolcite")]
    #[token("\\tvolcite")]
    #[token("\\Tvolcite")]
    #[token("\\avolcite")]
    #[token("\\Avolcite")]
    #[token("\\notecite")]
    #[token("\\pnotecite")]
    #[token("\\Pnotecite")]
    #[token("\\fnotecite")]
    #[token("\\citeA")]
    #[token("\\citeA*")]
    Citation,

    #[token("\\usepackage")]
    #[token("\\RequirePackage")]
    PackageInclude,

    #[token("\\documentclass")]
    ClassInclude,

    #[token("\\include")]
    #[token("\\subfileinclude")]
    #[token("\\input")]
    #[token("\\subfile")]
    LatexInclude,

    #[token("\\addbibresource")]
    BiblatexInclude,

    #[token("\\bibliography")]
    BibtexInclude,

    #[token("\\includegraphics")]
    GraphicsInclude,

    #[token("\\includesvg")]
    SvgInclude,

    #[token("\\includeinkscape")]
    InkscapeInclude,

    #[token("\\verbatiminput")]
    #[token("\\VerbatimInput")]
    VerbatimInclude,

    #[token("\\import")]
    #[token("\\subimport")]
    #[token("\\inputfrom")]
    #[token("\\subinputfrom")]
    #[token("\\includefrom")]
    #[token("\\subincludefrom")]
    Import,

    #[token("\\label")]
    LabelDefinition,

    #[token("\\ref")]
    #[token("\\vref")]
    #[token("\\Vref")]
    #[token("\\autoref")]
    #[token("\\pageref")]
    #[token("\\cref")]
    #[token("\\cref*")]
    #[token("\\Cref")]
    #[token("\\Cref*")]
    #[token("\\namecref")]
    #[token("\\nameCref")]
    #[token("\\lcnamecref")]
    #[token("\\namecrefs")]
    #[token("\\nameCrefs")]
    #[token("\\lcnamecrefs")]
    #[token("\\labelcref")]
    #[token("\\labelcpageref")]
    #[token("\\eqref")]
    LabelReference,

    #[token("\\crefrange")]
    #[token("\\crefrange*")]
    #[token("\\Crefrange")]
    #[token("\\Crefrange*")]
    LabelReferenceRange,

    #[token("\\newlabel")]
    LabelNumber,

    #[token("\\newcommand")]
    #[token("\\newcommand*")]
    #[token("\\renewcommand")]
    #[token("\\renewcommand*")]
    #[token("\\DeclareRobustCommand")]
    #[token("\\DeclareRobustCommand*")]
    CommandDefinition,

    #[token("\\DeclareMathOperator")]
    #[token("\\DeclareMathOperator*")]
    MathOperator,

    #[token("\\newglossaryentry")]
    GlossaryEntryDefinition,

    #[token("\\gls")]
    #[token("\\Gls")]
    #[token("\\GLS")]
    #[token("\\glspl")]
    #[token("\\Glspl")]
    #[token("\\GLSpl")]
    #[token("\\glsdisp")]
    #[token("\\glslink")]
    #[token("\\glstext")]
    #[token("\\Glstext")]
    #[token("\\GLStext")]
    #[token("\\glsfirst")]
    #[token("\\Glsfirst")]
    #[token("\\GLSfirst")]
    #[token("\\glsplural")]
    #[token("\\Glsplural")]
    #[token("\\GLSplural")]
    #[token("\\glsfirstplural")]
    #[token("\\Glsfirstplural")]
    #[token("\\GLSfirstplural")]
    #[token("\\glsname")]
    #[token("\\Glsname")]
    #[token("\\GLSname")]
    #[token("\\glssymbol")]
    #[token("\\Glssymbol")]
    #[token("\\glsdesc")]
    #[token("\\Glsdesc")]
    #[token("\\GLSdesc")]
    #[token("\\glsuseri")]
    #[token("\\Glsuseri")]
    #[token("\\GLSuseri")]
    #[token("\\glsuserii")]
    #[token("\\Glsuserii")]
    #[token("\\glsuseriii")]
    #[token("\\glsuseriv")]
    #[token("\\Glsuseriv")]
    #[token("\\GLSuseriv")]
    #[token("\\glsuserv")]
    #[token("\\Glsuserv")]
    #[token("\\GLSuserv")]
    #[token("\\glsuservi")]
    #[token("\\Glsuservi")]
    #[token("\\GLSuservi")]
    GlossaryEntryReference,

    #[token("\\newacronym")]
    AcronymDefinition,

    #[token("\\DeclareAcronym")]
    AcronymDeclaration,

    #[token("\\acrshort")]
    #[token("\\Acrshort")]
    #[token("\\ACRshort")]
    #[token("\\acrshortpl")]
    #[token("\\Acrshortpl")]
    #[token("\\ACRshortpl")]
    #[token("\\acrlong")]
    #[token("\\Acrlong")]
    #[token("\\ACRlong")]
    #[token("\\acrlongpl")]
    #[token("\\Acrlongpl")]
    #[token("\\ACRlongpl")]
    #[token("\\acrfull")]
    #[token("\\Acrfull")]
    #[token("\\ACRfull")]
    #[token("\\acrfullpl")]
    #[token("\\Acrfullpl")]
    #[token("\\ACRfullpl")]
    #[token("\\acs")]
    #[token("\\Acs")]
    #[token("\\acsp")]
    #[token("\\Acsp")]
    #[token("\\acl")]
    #[token("\\Acl")]
    #[token("\\aclp")]
    #[token("\\Aclp")]
    #[token("\\acf")]
    #[token("\\Acf")]
    #[token("\\acfp")]
    #[token("\\Acfp")]
    #[token("\\ac")]
    #[token("\\Ac")]
    #[token("\\acp")]
    #[token("\\glsentrylong")]
    #[token("\\Glsentrylong")]
    #[token("\\glsentrylongpl")]
    #[token("\\Glsentrylongpl")]
    #[token("\\glsentryshort")]
    #[token("\\Glsentryshort")]
    #[token("\\glsentryshortpl")]
    #[token("\\Glsentryshortpl")]
    #[token("\\glsentryfullpl")]
    #[token("\\Glsentryfullpl")]
    AcronymReference,

    #[token("\\newtheorem")]
    #[token("\\newtheorem*")]
    #[token("\\declaretheorem")]
    #[token("\\declaretheorem*")]
    TheoremDefinition,

    #[token("\\color")]
    #[token("\\colorbox")]
    #[token("\\textcolor")]
    #[token("\\pagecolor")]
    ColorReference,

    #[token("\\definecolor")]
    ColorDefinition,

    #[token("\\definecolorset")]
    ColorSetDefinition,

    #[token("\\usepgflibrary")]
    #[token("\\usetikzlibrary")]
    TikzLibraryImport,

    #[token("\\newenvironment")]
    #[token("\\newenvironment*")]
    #[token("\\renewenvironment")]
    #[token("\\renewenvironment*")]
    EnvironmentDefinition,

    #[token("\\graphicspath")]
    GraphicsPath,

    #[token("\\fi")]
    EndBlockComment,

    #[regex(r"\\([^\r\n]|[@a-zA-Z:_]+\*?)?")]
    #[error]
    Generic,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Logos)]
enum BlockCommentToken {
    #[token("\\fi")]
    End,

    #[regex(r"\\([^\r\n]|[@a-zA-Z:_]+\*?)?")]
    #[regex(r"[^\\]+")]
    #[error]
    Verbatim,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Logos)]
enum VerbatimEnvironmentToken {
    #[token("\\end{asy}")]
    #[token("\\end{verbatim}")]
    #[token("\\end{lstlisting}")]
    #[token("\\end{minted}")]
    #[token("\\end{pycode}")]
    End,

    #[regex(r"\\([^\r\n]|[@a-zA-Z:_]+\*?)?")]
    #[regex(r"[^\\]+")]
    #[error]
    Verbatim,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Lexer<'a> {
    tokens: Vec<(SyntaxKind, &'a str)>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut tokens = Vec::new();
        tokenize(input, &mut tokens);
        tokens.reverse();
        Self { tokens }
    }

    pub fn peek(&self) -> Option<SyntaxKind> {
        self.tokens.last().map(|(kind, _)| *kind)
    }

    pub fn eat(&mut self) -> Option<(SyntaxKind, &'a str)> {
        self.tokens.pop()
    }
}

fn tokenize<'a>(input: &'a str, tokens: &mut Vec<(SyntaxKind, &'a str)>) {
    let mut lexer = RootToken::lexer(input);
    while let Some(kind) = lexer.next() {
        let text = lexer.slice();
        match kind {
            RootToken::LineBreak => {
                tokens.push((SyntaxKind::LINE_BREAK, text));
            }
            RootToken::Whitespace => {
                tokens.push((SyntaxKind::WHITESPACE, text));
            }
            RootToken::LineComment => {
                tokens.push((SyntaxKind::COMMENT, text));
            }
            RootToken::LCurly => {
                tokens.push((SyntaxKind::L_CURLY, text));
            }
            RootToken::RCurly => {
                tokens.push((SyntaxKind::R_CURLY, text));
            }
            RootToken::LBrack => {
                tokens.push((SyntaxKind::L_BRACK, text));
            }
            RootToken::RBrack => {
                tokens.push((SyntaxKind::R_BRACK, text));
            }
            RootToken::LParen => {
                tokens.push((SyntaxKind::L_PAREN, text));
            }
            RootToken::RParen => {
                tokens.push((SyntaxKind::R_PAREN, text));
            }
            RootToken::Comma => {
                tokens.push((SyntaxKind::COMMA, text));
            }
            RootToken::Eq => {
                tokens.push((SyntaxKind::EQUALITY_SIGN, text));
            }
            RootToken::Word => {
                tokens.push((SyntaxKind::WORD, text));
            }
            RootToken::Dollar => {
                tokens.push((SyntaxKind::DOLLAR, text));
            }
            RootToken::CommandName => {
                let kind = tokenize_command_name(text);
                tokens.push((kind, text));
            }
            RootToken::BeginBlockComment => {
                tokens.push((SyntaxKind::BEGIN_BLOCK_COMMENT_NAME, text));
                let end = lexer.span().end;
                lexer = RootToken::lexer(tokenize_block_comment(&lexer.source()[end..], tokens));
            }
            RootToken::BeginVerbatimEnvironment => {
                tokens.push((SyntaxKind::BEGIN_ENVIRONMENT_NAME, "\\begin"));
                tokens.push((SyntaxKind::L_CURLY, "{"));
                tokens.push((SyntaxKind::WORD, &text["\\begin{".len()..text.len() - 1]));
                tokens.push((SyntaxKind::R_CURLY, "}"));
                let end = lexer.span().end;
                lexer = RootToken::lexer(tokenize_verbatim_environment(
                    &lexer.source()[end..],
                    tokens,
                ));
            }
        }
    }
}

fn tokenize_command_name(text: &str) -> SyntaxKind {
    let mut lexer = CommandNameToken::lexer(text);
    match lexer.next().unwrap() {
        CommandNameToken::BeginEnvironment => SyntaxKind::BEGIN_ENVIRONMENT_NAME,
        CommandNameToken::EndEnvironment => SyntaxKind::END_ENVIRONMENT_NAME,
        CommandNameToken::BeginEquation => SyntaxKind::BEGIN_EQUATION_NAME,
        CommandNameToken::EndEquation => SyntaxKind::END_EQUATION_NAME,
        CommandNameToken::Part => SyntaxKind::PART_NAME,
        CommandNameToken::Chapter => SyntaxKind::CHAPTER_NAME,
        CommandNameToken::Section => SyntaxKind::SECTION_NAME,
        CommandNameToken::Subsection => SyntaxKind::SUBSECTION_NAME,
        CommandNameToken::Subsubsection => SyntaxKind::SUBSUBSECTION_NAME,
        CommandNameToken::Paragraph => SyntaxKind::PARAGRAPH_NAME,
        CommandNameToken::Subparagraph => SyntaxKind::SUBPARAGRAPH_NAME,
        CommandNameToken::EnumItem => SyntaxKind::ENUM_ITEM_NAME,
        CommandNameToken::Caption => SyntaxKind::CAPTION_NAME,
        CommandNameToken::Citation => SyntaxKind::CITATION_NAME,
        CommandNameToken::PackageInclude => SyntaxKind::PACKAGE_INCLUDE_NAME,
        CommandNameToken::ClassInclude => SyntaxKind::CLASS_INCLUDE_NAME,
        CommandNameToken::LatexInclude => SyntaxKind::LATEX_INCLUDE_NAME,
        CommandNameToken::BiblatexInclude => SyntaxKind::BIBLATEX_INCLUDE_NAME,
        CommandNameToken::BibtexInclude => SyntaxKind::BIBTEX_INCLUDE_NAME,
        CommandNameToken::GraphicsInclude => SyntaxKind::GRAPHICS_INCLUDE_NAME,
        CommandNameToken::SvgInclude => SyntaxKind::SVG_INCLUDE_NAME,
        CommandNameToken::InkscapeInclude => SyntaxKind::INKSCAPE_INCLUDE_NAME,
        CommandNameToken::VerbatimInclude => SyntaxKind::VERBATIM_INCLUDE_NAME,
        CommandNameToken::Import => SyntaxKind::IMPORT_NAME,
        CommandNameToken::LabelDefinition => SyntaxKind::LABEL_DEFINITION_NAME,
        CommandNameToken::LabelReference => SyntaxKind::LABEL_REFERENCE_NAME,
        CommandNameToken::LabelReferenceRange => SyntaxKind::LABEL_REFERENCE_RANGE_NAME,
        CommandNameToken::LabelNumber => SyntaxKind::LABEL_NUMBER_NAME,
        CommandNameToken::CommandDefinition => SyntaxKind::COMMAND_DEFINITION_NAME,
        CommandNameToken::MathOperator => SyntaxKind::MATH_OPERATOR_NAME,
        CommandNameToken::GlossaryEntryDefinition => SyntaxKind::GLOSSARY_ENTRY_DEFINITION_NAME,
        CommandNameToken::GlossaryEntryReference => SyntaxKind::GLOSSARY_ENTRY_REFERENCE_NAME,
        CommandNameToken::AcronymDefinition => SyntaxKind::ACRONYM_DEFINITION_NAME,
        CommandNameToken::AcronymDeclaration => SyntaxKind::ACRONYM_DECLARATION_NAME,
        CommandNameToken::AcronymReference => SyntaxKind::ACRONYM_REFERENCE_NAME,
        CommandNameToken::TheoremDefinition => SyntaxKind::THEOREM_DEFINITION_NAME,
        CommandNameToken::ColorReference => SyntaxKind::COLOR_REFERENCE_NAME,
        CommandNameToken::ColorDefinition => SyntaxKind::COLOR_DEFINITION_NAME,
        CommandNameToken::ColorSetDefinition => SyntaxKind::COLOR_SET_DEFINITION_NAME,
        CommandNameToken::TikzLibraryImport => SyntaxKind::TIKZ_LIBRARY_IMPORT_NAME,
        CommandNameToken::EnvironmentDefinition => SyntaxKind::ENVIRONMENT_DEFINITION_NAME,
        CommandNameToken::EndBlockComment => SyntaxKind::END_BLOCK_COMMENT_NAME,
        CommandNameToken::GraphicsPath => SyntaxKind::GRAPHICS_PATH_NAME,
        CommandNameToken::Generic => SyntaxKind::GENERIC_COMMAND_NAME,
    }
}

fn tokenize_block_comment<'a>(input: &'a str, tokens: &mut Vec<(SyntaxKind, &'a str)>) -> &'a str {
    let mut lexer = BlockCommentToken::lexer(input);
    let mut end = 0;
    while let Some(kind) = lexer.next() {
        match kind {
            BlockCommentToken::Verbatim => {
                end = lexer.span().end;
            }
            BlockCommentToken::End => {
                end = lexer.span().start;
                break;
            }
        };
    }

    if end > 0 {
        tokens.push((SyntaxKind::VERBATIM, &input[..end]));
    }

    &input[end..]
}

fn tokenize_verbatim_environment<'a>(
    input: &'a str,
    tokens: &mut Vec<(SyntaxKind, &'a str)>,
) -> &'a str {
    let mut lexer = VerbatimEnvironmentToken::lexer(input);
    let mut end = 0;
    while let Some(kind) = lexer.next() {
        match kind {
            VerbatimEnvironmentToken::Verbatim => {
                end = lexer.span().end;
            }
            VerbatimEnvironmentToken::End => {
                end = lexer.span().start;
                break;
            }
        };
    }

    if end > 0 {
        tokens.push((SyntaxKind::VERBATIM, &input[..end]));
    }

    &input[end..]
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;

    fn verify(text: &str) -> Vec<(SyntaxKind, &str)> {
        let mut tokens = Lexer::new(text).tokens;
        tokens.reverse();
        tokens
    }

    #[test]
    fn test_empty() {
        assert_debug_snapshot!(verify(r#""#));
    }

    #[test]
    fn test_delimiters() {
        assert_debug_snapshot!(verify(r#"{foo} (bar) [baz, qux = foo-bar]"#));
    }

    #[test]
    fn test_command_with_parameter() {
        assert_debug_snapshot!(verify(r#"\newcommand{\id}[1]{#1}"#));
    }

    #[test]
    fn test_command_with_star() {
        assert_debug_snapshot!(verify(r#"\section*{Foo}"#));
    }

    #[test]
    fn test_escape_sequence() {
        assert_debug_snapshot!(verify(r#"\% hello"#));
    }

    #[test]
    fn test_formula() {
        assert_debug_snapshot!(verify(r#"$ f(x) = y $$"#));
    }

    #[test]
    fn test_comment() {
        assert_debug_snapshot!(verify("hello %world\r\ntest %test"));
    }

    #[test]
    fn test_invalid_parameter() {
        assert_debug_snapshot!(verify(r#"#"#))
    }

    #[test]
    fn test_line_break() {
        assert_debug_snapshot!(verify("hello\nworld"));
    }

    #[test]
    fn test_block_comment() {
        assert_debug_snapshot!(verify("Foo\\iffalse\n\\Bar{Baz}\n\\fi\\Qux"));
    }

    #[test]
    fn test_asymptote() {
        assert_debug_snapshot!(verify(
            r#"\begin{asy}
    printf("Hello World\n");
\end{asy}"#
        ));
    }
}
