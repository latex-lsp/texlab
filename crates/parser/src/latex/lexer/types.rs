use logos::Logos;

use crate::util::lex_command_name;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Logos)]
pub enum Token {
    #[regex(r"[\r\n]+", priority = 2)]
    LineBreak,

    #[regex(r"[^\S\r\n]+", priority = 1)]
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

    #[token("|")]
    Pipe,

    #[regex(r"[^\s\\%\{\},\$\[\]\(\)=\|]+")]
    Word,

    #[regex(r"\$\$?")]
    Dollar,

    #[regex(r"\\", |lexer| { lex_command_name(lexer); CommandName::Generic } )]
    CommandName(CommandName),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum CommandName {
    Generic,
    BeginEnvironment,
    EndEnvironment,
    BeginEquation,
    EndEquation,
    Section(SectionLevel),
    Paragraph(ParagraphLevel),
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
    OldCommandDefinition,
    NewCommandDefinition,
    MathOperator,
    GlossaryEntryDefinition,
    GlossaryEntryReference,
    AcronymDefinition,
    AcronymDeclaration,
    AcronymReference,
    TheoremDefinitionAmsThm,
    TheoremDefinitionThmTools,
    ColorReference,
    ColorDefinition,
    ColorSetDefinition,
    TikzLibraryImport,
    EnvironmentDefinition,
    GraphicsPath,
    BeginBlockComment,
    EndBlockComment,
    VerbatimBlock,
    BibItem,
    TocContentsLine,
    TocNumberLine,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum SectionLevel {
    Part,
    Chapter,
    Section,
    Subsection,
    Subsubsection,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum ParagraphLevel {
    Paragraph,
    Subparagraph,
}
