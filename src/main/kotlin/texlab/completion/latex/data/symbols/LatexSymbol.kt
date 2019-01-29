package texlab.completion.latex.data.symbols

sealed class LatexSymbol {
    abstract val command: String?
    abstract val component: String?
    abstract val imageId: Int
}

data class LatexCommandSymbol(override val command: String,
                              override val component: String?,
                              override val imageId: Int) : LatexSymbol()

data class LatexArgumentSymbol(override val command: String,
                               override val component: String?,
                               val argument: String,
                               val index: Int,
                               override val imageId: Int) : LatexSymbol()
