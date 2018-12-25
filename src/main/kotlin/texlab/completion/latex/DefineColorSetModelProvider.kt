package texlab.completion.latex

object DefineColorSetModelProvider : LatexColorModelProvider() {
    override val commandNames: List<String> = listOf("\\definecolorset")

    override val argumentIndex: Int = 0
}
