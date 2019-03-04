package texlab.provider

interface FeatureProvider<T, R> {
    suspend fun get(request: FeatureRequest<T>): List<R>

    fun <C> map(transform: (List<R>) -> List<C>): FeatureProvider<T, C> {
        return create { request ->
            transform(get(request))
        }
    }

    companion object {
        fun <T, R> create(get: suspend (FeatureRequest<T>) -> List<R>): FeatureProvider<T, R> {
            return object : FeatureProvider<T, R> {
                override suspend fun get(request: FeatureRequest<T>): List<R> = get(request)
            }
        }

        fun <T, R> concat(vararg providers: FeatureProvider<T, R>): FeatureProvider<T, R> {
            return create { request ->
                providers.flatMap { it.get(request) }
            }
        }
    }
}
