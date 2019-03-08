package texlab.rename

import org.eclipse.lsp4j.RenameParams
import org.eclipse.lsp4j.WorkspaceEdit
import texlab.provider.FeatureProvider

object RenameProvider : FeatureProvider<RenameParams, WorkspaceEdit?> by
FeatureProvider.choice(
        LatexCommandRenamer,
        LatexEnvironmentRenamer,
        LatexLabelRenamer,
        BibtexEntryRenamer)
