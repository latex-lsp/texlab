package texlab.metadata

interface MetadataProvider {
    suspend fun getMetadata(name: String): Metadata?
}
