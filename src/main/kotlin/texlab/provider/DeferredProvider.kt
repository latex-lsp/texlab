package texlab.provider

import kotlinx.coroutines.Deferred

class DeferredProvider<S, T, R>(private val providerFactory: (source: S) -> FeatureProvider<T, R>,
                                private val source: Deferred<S>,
                                private val defaultValue: R) : FeatureProvider<T, R> {
    private var provider: FeatureProvider<T, R>? = null

    override suspend fun get(request: FeatureRequest<T>): R {
        if (provider == null && source.isCompleted) {
            provider = providerFactory(source.await())
        }

        return provider?.get(request) ?: defaultValue
    }
}
