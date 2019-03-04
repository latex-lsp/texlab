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

    companion object {
        fun <T, R> concat(vararg providers: FeatureProvider<T, R>): FeatureProvider<T, R> {
            return object : FeatureProvider<T, R> {
                override suspend fun get(request: FeatureRequest<T>): List<R> {
                    return providers.flatMap { it.get(request) }
                }
            }
        }
    }
}
