package texlab.provider

class DistinctProvider<T, K, R>(private val provider: FeatureProvider<T, R>,
                                private val selector: (R) -> K) : FeatureProvider<T, R> {
    override suspend fun get(request: FeatureRequest<T>): List<R> {
        return provider.get(request).distinctBy(selector)
    }
}
