package texlab.completion.latex

class DefineColorSetModelProvider : LatexColorModelProvider() {

    override val commandNames: List<String> = listOf("\\definecolorset")

    override val argumentIndex: Int = 0
}
