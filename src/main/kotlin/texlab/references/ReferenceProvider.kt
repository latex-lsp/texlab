package texlab.references

import org.eclipse.lsp4j.Location
import org.eclipse.lsp4j.ReferenceParams
import texlab.provider.FeatureProvider

object ReferenceProvider : FeatureProvider<ReferenceParams, List<Location>> by
FeatureProvider.concat(
        LatexLabelReferenceProvider,
        BibtexEntryReferenceProvider)
