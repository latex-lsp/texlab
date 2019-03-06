package texlab.provider

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import texlab.WorkspaceBuilder
import texlab.provider.FeatureProviderTests.NumberProvider

class DeferredProviderTests {
    @Test
    fun `it should eventually provide the source to the given provider`() = runBlocking {
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .request("foo.tex") {}

        val deferred = CompletableDeferred<Int>()
        val deferredProvider = DeferredProvider(::NumberProvider, deferred, null)
        deferred.complete(42)
        val result = deferredProvider.get(request)
        assertNotNull(result)
    }

    @Test
    fun `it should return nothing as long as the source is not ready`() = runBlocking {
        val request = WorkspaceBuilder()
                .document("foo.tex", "")
                .request("foo.tex") {}

        val deferred = CompletableDeferred<Int>()
        val deferredProvider = DeferredProvider(::NumberProvider, deferred, null)
        val result = deferredProvider.get(request)
        assertNull(result)
    }
}
