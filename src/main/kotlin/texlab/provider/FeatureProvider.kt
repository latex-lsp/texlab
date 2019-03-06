package texlab.provider

interface FeatureProvider<T, R> {
    suspend fun get(request: FeatureRequest<T>): R

    fun <C> map(transform: (R) -> C): FeatureProvider<T, C> {
        return create { request ->
            transform(get(request))
        }
    }

    companion object {
        fun <T, R> create(get: suspend (FeatureRequest<T>) -> R): FeatureProvider<T, R> {
            return object : FeatureProvider<T, R> {
                override suspend fun get(request: FeatureRequest<T>): R = get(request)
            }
        }

        fun <T, R> concat(vararg providers: FeatureProvider<T, List<R>>): FeatureProvider<T, List<R>> {
            return create { request ->
                providers.flatMap { it.get(request) }
            }
        }

        fun <T, R> choice(vararg providers: FeatureProvider<T, R?>): FeatureProvider<T, R?> {
            return create { request ->
                for (provider in providers) {
                    val result = provider.get(request)
                    if (result != null) {
                        return@create result
                    }
                }

                null
            }
        }
    }
}
