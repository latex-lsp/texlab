package texlab.completion.latex.data

data class LatexComponent(val fileNames: List<String>,
                          val references: List<String>,
                          val commands: List<String>,
                          val environments: List<String>)
