use logos::Logos;

use super::kind::SyntaxKind;

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

    #[regex(r"\\newcommand\*?|\\renewcommand|\\DeclareRobustCommand")]
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
pub struct Lexer<'a> {
    tokens: Vec<(SyntaxKind, &'a str)>,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        let mut tokens = Vec::new();
        let mut lexer = LogosToken::lexer(text);
        while let Some(kind) = lexer.next() {
            tokens.push((
                unsafe { std::mem::transmute::<LogosToken, SyntaxKind>(kind) },
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
