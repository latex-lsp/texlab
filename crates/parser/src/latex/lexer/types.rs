use logos::Logos;

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

    #[regex(r"\\", lex_command_name)]
    CommandName(CommandName),
}

fn lex_command_name(lexer: &mut logos::Lexer<Token>) -> CommandName {
    let input = &lexer.source()[lexer.span().end..];

    let mut chars = input.chars().peekable();
    let Some(c) = chars.next() else {
        return CommandName::Generic;
    };

    if c.is_whitespace() {
        return CommandName::Generic;
    }

    lexer.bump(c.len_utf8());
    if !c.is_alphanumeric() && c != '@' {
        return CommandName::Generic;
    }

    while let Some(c) = chars.next() {
        match c {
            '*' => {
                lexer.bump(c.len_utf8());
                break;
            }
            c if c.is_alphanumeric() => {
                lexer.bump(c.len_utf8());
            }
            '_' => {
                if !matches!(chars.peek(), Some(c) if c.is_alphanumeric()) {
                    break;
                }

                lexer.bump(c.len_utf8());
            }
            '@' | ':' => {
                lexer.bump(c.len_utf8());
            }
            _ => {
                break;
            }
        }
    }

    CommandName::Generic
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
