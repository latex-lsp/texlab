package texlab.syntax.bibtex

enum class BibtexTokenKind {
    PREAMBLE_TYPE,
    STRING_TYPE,
    ENTRY_TYPE,
    WORD,
    COMMAND,
    ASSIGN,
    COMMA,
    CONCAT,
    QUOTE,
    BEGIN_BRACE,
    END_BRACE,
    BEGIN_PAREN,
    END_PAREN;
}
