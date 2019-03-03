package texlab.provider

interface FeatureProvider<T, R> {
    suspend fun get(request: FeatureRequest<T>): List<R>

    fun <C> map(transform: (List<R>) -> List<C>): FeatureProvider<T, C> {
        return object : FeatureProvider<T, C> {
            override suspend fun get(request: FeatureRequest<T>): List<C> {
                val result = this@FeatureProvider.get(request)
                return transform(result)
            }
        }
    }
}
