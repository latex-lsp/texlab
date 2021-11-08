use logos::Logos;

use super::kind::SyntaxKind;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Logos)]
#[allow(non_camel_case_types)]
#[repr(u16)]
enum Token {
    #[regex(r"\s+")]
    WHITESPACE = 2,

    #[regex(r"%[^\r\n]*")]
    COMMENT,

    #[token("{")]
    L_CURLY,

    #[token("}")]
    R_CURLY,

    #[token("[")]
    L_BRACK,

    #[token("]")]
    R_BRACK,

    #[token("(")]
    L_PAREN,

    #[token(")")]
    R_PAREN,

    #[token(",")]
    COMMA,

    #[token("=")]
    EQUALITY_SIGN,

    #[regex(r"[^\s\\%\{\},\$\[\]\(\)=]+")]
    #[error]
    WORD,

    #[regex(r"\$\$?")]
    DOLLAR,

    #[regex(r"\\([^\r\n]|[@a-zA-Z:_]+\*?)?")]
    GENERIC_COMMAND_NAME,

    #[regex(r"\\begin")]
    BEGIN_ENVIRONMENT_NAME,

    #[regex(r"\\end")]
    END_ENVIRONMENT_NAME,

    #[regex(r"\\\[")]
    BEGIN_EQUATION_NAME,

    #[regex(r"\\\]")]
    END_EQUATION_NAME,

    #[regex(r"\\part\*?")]
    PART_NAME,

    #[regex(r"\\chapter\*?")]
    CHAPTER_NAME,

    #[regex(r"\\section\*?")]
    SECTION_NAME,

    #[regex(r"\\subsection\*?")]
    SUBSECTION_NAME,

    #[regex(r"\\subsubsection\*?")]
    SUBSUBSECTION_NAME,

    #[regex(r"\\paragraph\*?")]
    PARAGRAPH_NAME,

    #[regex(r"\\subparagraph\*?")]
    SUBPARAGRAPH_NAME,

    #[regex(r"\\item")]
    ENUM_ITEM_NAME,

    #[regex(r"\\caption")]
    CAPTION_NAME,

    #[regex(r"\\cite|\\cite\*|\\Cite|\\nocite|\\citet|\\citep|\\citet\*|\\citep\*|\\citeauthor|\\citeauthor\*|\\Citeauthor|\\Citeauthor\*|\\citetitle|\\citetitle\*|\\citeyear|\\citeyear\*|\\citedate|\\citedate\*|\\citeurl|\\fullcite|\\citeyearpar|\\citealt|\\citealp|\\citetext|\\parencite|\\parencite\*|\\Parencite|\\footcite|\\footfullcite|\\footcitetext|\\textcite|\\Textcite|\\smartcite|\\Smartcite|\\supercite|\\autocite|\\Autocite|\\autocite\*|\\Autocite\*|\\volcite|\\Volcite|\\pvolcite|\\Pvolcite|\\fvolcite|\\ftvolcite|\\svolcite|\\Svolcite|\\tvolcite|\\Tvolcite|\\avolcite|\\Avolcite|\\notecite|\\notecite|\\pnotecite|\\Pnotecite|\\fnotecite|\\citeA|\\citeA\*")]
    CITATION_NAME,

    #[regex(r"\\usepackage|\\RequirePackage")]
    PACKAGE_INCLUDE_NAME,

    #[regex(r"\\documentclass")]
    CLASS_INCLUDE_NAME,

    #[regex(r"\\include|\\subfileinclude|\\input|\\subfile")]
    LATEX_INCLUDE_NAME,

    #[regex(r"\\addbibresource")]
    BIBLATEX_INCLUDE_NAME,

    #[regex(r"\\bibliography")]
    BIBTEX_INCLUDE_NAME,

    #[regex(r"\\includegraphics")]
    GRAPHICS_INCLUDE_NAME,

    #[regex(r"\\includesvg")]
    SVG_INCLUDE_NAME,

    #[regex(r"\\includeinkscape")]
    INKSCAPE_INCLUDE_NAME,

    #[regex(r"\\verbatiminput|\\VerbatimInput")]
    VERBATIM_INCLUDE_NAME,

    #[regex(r"\\import|\\subimport|\\inputfrom|\\subimportfrom|\\includefrom|\\subincludefrom")]
    IMPORT_NAME,

    #[regex(r"\\label")]
    LABEL_DEFINITION_NAME,

    #[regex(r"\\ref|\\vref|\\Vref|\\autoref|\\pageref|\\cref|\\Cref|\\cref*|\\Cref*|\\namecref|\\nameCref|\\lcnamecref|\\namecrefs|\\nameCrefs|\\lcnamecrefs|\\labelcref|\\labelcpageref|\\eqref")]
    LABEL_REFERENCE_NAME,

    #[regex(r"\\crefrange\*?|\\Crefrange\*?")]
    LABEL_REFERENCE_RANGE_NAME,

    #[regex(r"\\newlabel")]
    LABEL_NUMBER_NAME,

    #[regex(r"\\newcommand\*?|\\renewcommand|\\DeclareRobustCommand")]
    COMMAND_DEFINITION_NAME,

    #[regex(r"\\DeclareMathOperator\*?")]
    MATH_OPERATOR_NAME,

    #[regex(r"\\newglossaryentry")]
    GLOSSARY_ENTRY_DEFINITION_NAME,

    #[regex(r"\\gls|\\Gls|\\GLS|\\glspl|\\Glspl|\\GLSpl|\\glsdisp|\\glslink|\\glstext|\\Glstext|\\GLStext|\\glsfirst|\\Glsfirst|\\GLSfirst|\\glsplural|\\Glsplural|\\GLSplural|\\glsfirstplural|\\Glsfirstplural|\\GLSfirstplural|\\glsname|\\Glsname|\\GLSname|\\glssymbol|\\Glssymbol|\\glsdesc|\\Glsdesc|\\GLSdesc|\\glsuseri|\\Glsuseri|\\GLSuseri|\\glsuserii|\\Glsuserii|\\GLSuserii|\\glsuseriii|\\Glsuseriii|\\GLSuseriii|\\glsuseriv|\\Glsuseriv|\\GLSuseriv|\\glsuserv|\\Glsuserv|\\GLSuserv|\\glsuservi|\\Glsuservi|\\GLSuservi")]
    GLOSSARY_ENTRY_REFERENCE_NAME,

    #[regex(r"\\newacronym")]
    ACRONYM_DEFINITION_NAME,

    #[regex(r"\\DeclareAcronym")]
    ACRONYM_DECLARATION_NAME,

    #[regex(r"\\acrshort|\\Acrshort|\\ACRshort|\\acrshortpl|\\Acrshortpl|\\ACRshortpl|\\acrlong|\\Acrlong|\\ACRlong|\\acrlongpl|\\Acrlongpl|\\ACRlongpl|\\acrfull|\\Acrfull|\\ACRfull|\\acrfullpl|\\Acrfullpl|\\ACRfullpl|\\acs|\\Acs|\\acsp|\\Acsp|\\acl|\\Acl|\\aclp|\\Aclp|\\acf|\\Acf|\\acfp|\\Acfp|\\ac|\\Ac|\\acp|\\glsentrylong|\\Glsentrylong|\\glsentrylongpl|\\Glsentrylongpl|\\glsentryshort|\\Glsentryshort|\\glsentryshortpl|\\Glsentryshortpl|\\glsentryfullpl|\\Glsentryfullpl")]
    ACRONYM_REFERENCE_NAME,

    #[regex(r"\\newtheorem|\\declaretheorem")]
    THEOREM_DEFINITION_NAME,

    #[regex(r"\\color|\\colorbox|\\textcolor|\\pagecolor")]
    COLOR_REFERENCE_NAME,

    #[regex(r"\\definecolor")]
    COLOR_DEFINITION_NAME,

    #[regex(r"\\definecolorset")]
    COLOR_SET_DEFINITION_NAME,

    #[regex(r"\\usepgflibrary|\\usetikzlibrary")]
    TIKZ_LIBRARY_IMPORT_NAME,

    #[regex(r"\\newenvironment|\\newenvironment*")]
    ENVIRONMENT_DEFINITION_NAME,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Lexer<'a> {
    tokens: Vec<(SyntaxKind, &'a str)>,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        let mut tokens = Vec::new();
        let mut lexer = Token::lexer(text);
        while let Some(kind) = lexer.next() {
            tokens.push((
                unsafe { std::mem::transmute::<Token, SyntaxKind>(kind) },
                lexer.slice(),
            ));
        }
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
}
