package texlab.completion.latex

object DefineColorModelProvider : LatexColorModelProvider() {

    override val commandNames: List<String> = listOf("\\definecolor")

    override val argumentIndex: Int = 1
}
