use logos::Logos;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Logos)]
pub enum Token {
    #[regex(r"[\r\n]+", priority = 2)]
    LineBreak,

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

    #[regex(r"\\([^\r\n]|[@a-zA-Z:_]+\*?)?", |_| CommandName::Generic)]
    CommandName(CommandName),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum CommandName {
    Generic,
    BeginEnvironment,
    EndEnvironment,
    BeginEquation,
    EndEquation,
    Part,
    Chapter,
    Section,
    Subsection,
    Subsubsection,
    Paragraph,
    Subparagraph,
    EnumItem,
    Caption,
    Citation,
    PackageInclude,
    ClassInclude,
    LatexInclude,
    BiblatexInclude,
    BibtexInclude,
    GraphicsInclude,
    SvgInclude,
    InkscapeInclude,
    VerbatimInclude,
    Import,
    LabelDefinition,
    LabelReference,
    LabelReferenceRange,
    LabelNumber,
    CommandDefinition,
    MathOperator,
    GlossaryEntryDefinition,
    GlossaryEntryReference,
    AcronymDefinition,
    AcronymDeclaration,
    AcronymReference,
    TheoremDefinition,
    ColorReference,
    ColorDefinition,
    ColorSetDefinition,
    TikzLibraryImport,
    EnvironmentDefinition,
    GraphicsPath,
    BeginBlockComment,
    EndBlockComment,
}
