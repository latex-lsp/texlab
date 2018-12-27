package texlab.build

data class BuildResult(val status: BuildStatus, val errors: List<BuildError>)
