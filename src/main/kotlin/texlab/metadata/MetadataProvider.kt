package texlab.metadata

interface MetadataProvider {
    fun getMetadata(name: String): Metadata?
}
