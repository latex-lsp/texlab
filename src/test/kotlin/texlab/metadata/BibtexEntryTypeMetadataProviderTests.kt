package texlab.metadata

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test

class BibtexEntryTypeMetadataProviderTests {
    @Test
    fun `it should return metadata when using a valid entry type`() {
        val metadata = BibtexEntryTypeMetadataProvider.getMetadata("article")!!
        assertEquals("article", metadata.name)
    }

    @Test
    fun `it should return null when using an invalid entry type`() {
        val metadata = BibtexEntryTypeMetadataProvider.getMetadata("foo")
        assertNull(metadata)
    }
}
