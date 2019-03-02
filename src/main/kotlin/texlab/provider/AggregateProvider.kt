package texlab.provider

class AggregateProvider<T, R>(private vararg val providers: FeatureProvider<T, R>)
    : FeatureProvider<T, R> {
    override suspend fun get(request: FeatureRequest<T>): List<R> {
        return providers.flatMap { it.get(request) }
    }
}
