use logos::Logos;

use super::kind::SyntaxKind;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Logos)]
#[allow(non_camel_case_types)]
#[repr(u16)]
enum Token {
    #[regex(r"[\r\n]+", priority = 2)]
    LINE_BREAK = 2,

    #[regex(r"\s+", priority = 1)]
    WHITESPACE,

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

    #[token("\\begin")]
    BEGIN_ENVIRONMENT_NAME,

    #[token("\\end")]
    END_ENVIRONMENT_NAME,

    #[token("\\[")]
    BEGIN_EQUATION_NAME,

    #[token("\\]")]
    END_EQUATION_NAME,

    #[token("\\part")]
    #[token("\\part*")]
    PART_NAME,

    #[token("\\chapter")]
    #[token("\\chapter*")]
    CHAPTER_NAME,

    #[token("\\section")]
    #[token("\\section*")]
    SECTION_NAME,

    #[token("\\subsection")]
    #[token("\\subsection*")]
    SUBSECTION_NAME,

    #[token("\\subsubsection")]
    #[token("\\subsubsection*")]
    SUBSUBSECTION_NAME,

    #[token("\\paragraph")]
    #[token("\\paragraph*")]
    PARAGRAPH_NAME,

    #[token("\\subparagraph")]
    #[token("\\subparagraph*")]
    SUBPARAGRAPH_NAME,

    #[token("\\item")]
    ENUM_ITEM_NAME,

    #[token("\\caption")]
    CAPTION_NAME,

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
    CITATION_NAME,

    #[token("\\usepackage")]
    #[token("\\RequirePackage")]
    PACKAGE_INCLUDE_NAME,

    #[token("\\documentclass")]
    CLASS_INCLUDE_NAME,

    #[token("\\include")]
    #[token("\\subfileinclude")]
    #[token("\\input")]
    #[token("\\subfile")]
    LATEX_INCLUDE_NAME,

    #[token("\\addbibresource")]
    BIBLATEX_INCLUDE_NAME,

    #[token("\\bibliography")]
    BIBTEX_INCLUDE_NAME,

    #[token("\\includegraphics")]
    GRAPHICS_INCLUDE_NAME,

    #[token("\\includesvg")]
    SVG_INCLUDE_NAME,

    #[token("\\includeinkscape")]
    INKSCAPE_INCLUDE_NAME,

    #[token("\\verbatiminput")]
    #[token("\\VerbatimInput")]
    VERBATIM_INCLUDE_NAME,

    #[token("\\import")]
    #[token("\\subimport")]
    #[token("\\inputfrom")]
    #[token("\\subimportfrom")]
    #[token("\\includefrom")]
    #[token("\\subincludefrom")]
    IMPORT_NAME,

    #[token("\\label")]
    LABEL_DEFINITION_NAME,

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
    LABEL_REFERENCE_NAME,

    #[token("\\crefrange")]
    #[token("\\crefrange*")]
    #[token("\\Crefrange")]
    #[token("\\Crefrange*")]
    LABEL_REFERENCE_RANGE_NAME,

    #[token("\\newlabel")]
    LABEL_NUMBER_NAME,

    #[token("\\newcommand")]
    #[token("\\newcommand*")]
    #[token("\\renewcommand")]
    #[token("\\renewcommand*")]
    #[token("\\DeclareRobustCommand")]
    #[token("\\DeclareRobustCommand*")]
    COMMAND_DEFINITION_NAME,

    #[token("\\DeclareMathOperator")]
    #[token("\\DeclareMathOperator*")]
    MATH_OPERATOR_NAME,

    #[token("\\newglossaryentry")]
    GLOSSARY_ENTRY_DEFINITION_NAME,

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
    GLOSSARY_ENTRY_REFERENCE_NAME,

    #[token("\\newacronym")]
    ACRONYM_DEFINITION_NAME,

    #[token("\\DeclareAcronym")]
    ACRONYM_DECLARATION_NAME,

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
    ACRONYM_REFERENCE_NAME,

    #[token("\\newtheorem")]
    #[token("\\newtheorem*")]
    #[token("\\declaretheorem")]
    #[token("\\declaretheorem*")]
    THEOREM_DEFINITION_NAME,

    #[token("\\color")]
    #[token("\\colorbox")]
    #[token("\\textcolor")]
    #[token("\\pagecolor")]
    COLOR_REFERENCE_NAME,

    #[token("\\definecolor")]
    COLOR_DEFINITION_NAME,

    #[token("\\definecolorset")]
    COLOR_SET_DEFINITION_NAME,

    #[token("\\usepgflibrary")]
    #[token("\\usetikzlibrary")]
    TIKZ_LIBRARY_IMPORT_NAME,

    #[token("\\newenvironment")]
    #[token("\\newenvironment*")]
    #[token("\\renewenvironment")]
    #[token("\\renewenvironment*")]
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

    #[test]
    fn test_line_break() {
        assert_debug_snapshot!(verify("hello\nworld"));
    }
}
