package texlab.build

data class BuildConfig(var executable: String = "latexmk",
                       var args: List<String> = listOf("-pdf", "--interaction=nonstopmode"))
