package texlab.provider

class LimitedProvider<T, R>(private val provider: FeatureProvider<T, R>,
                            private val limit: Int = 50) : FeatureProvider<T, R> {
    override suspend fun get(request: FeatureRequest<T>): List<R> {
        return provider.get(request).take(limit)
    }
}
