package texlab.completion.bibtex

object BibtexFieldDocumentation {
    fun getDocumentation(name: String): String {
        return when (name) {
            "abstract" ->
                "This field is intended for recording abstracts in a `bib` file, to be printed by a special\n" +
                        "bibliography style. It is not used by all standard bibliography styles."
            else ->
                ""
        }
    }
}
