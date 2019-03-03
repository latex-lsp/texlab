package texlab.syntax.latex

enum class LatexTokenKind {
    COMMAND,
    WORD,
    MATH,
    BEGIN_GROUP,
    END_GROUP,
    BEGIN_OPTIONS,
    END_OPTIONS
}
