package texlab

enum class Language {
    LATEX,
    BIBTEX;
}

fun getLanguageById(id: String): Language? {
    return when (id) {
        "latex" ->
            Language.LATEX
        "bibtex" ->
            Language.BIBTEX
        else ->
            null
    }
}

fun getLanguageByExtension(extension: String): Language? {
    return when (extension.toLowerCase()) {
        "tex" ->
            Language.LATEX
        "sty" ->
            Language.LATEX
        "cls" ->
            Language.LATEX
        "bib" ->
            Language.BIBTEX
        else ->
            null
    }
}
