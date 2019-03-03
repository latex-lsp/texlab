package texlab.metadata

import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test

class BibtexEntryTypeMetadataProviderTests {
    @Test
    fun `it should return metadata when using a valid entry type`() = runBlocking {
        val metadata = BibtexEntryTypeMetadataProvider.getMetadata("article")!!
        assertEquals("article", metadata.name)
    }

    @Test
    fun `it should return nothing when using an invalid entry type`() = runBlocking {
        val metadata = BibtexEntryTypeMetadataProvider.getMetadata("foo")
        assertNull(metadata)
    }
}
