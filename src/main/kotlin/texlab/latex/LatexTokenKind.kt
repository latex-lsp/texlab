package texlab.latex

enum class LatexTokenKind {
    COMMAND,
    WORD,
    BEGIN_GROUP,
    END_GROUP,
    BEGIN_OPTIONS,
    END_OPTIONS
}
