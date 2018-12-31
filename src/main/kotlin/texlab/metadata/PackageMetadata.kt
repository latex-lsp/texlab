package texlab.metadata

data class PackageMetadata(val name: String,
                           val caption: String,
                           val descriptions: List<PackageDescription>)
