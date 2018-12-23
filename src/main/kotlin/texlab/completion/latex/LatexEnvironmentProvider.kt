package texlab.completion.latex

abstract class LatexEnvironmentProvider : LatexArgumentProvider() {

    override val commandNames: List<String> = listOf("\\begin", "\\end")

    override val argumentIndex: Int = 0
}
