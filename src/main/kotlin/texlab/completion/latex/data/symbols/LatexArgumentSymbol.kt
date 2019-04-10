package texlab.completion.latex.data.symbols

data class LatexArgumentSymbol(val command: String,
                               val component: String?,
                               val index: Int,
                               val arguments: List<LatexArgumentSymbolItem>)