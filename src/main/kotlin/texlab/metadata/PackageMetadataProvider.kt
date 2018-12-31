package texlab.metadata

interface PackageMetadataProvider {
    fun getMetadata(name: String): PackageMetadata?
}
