import { concat, deferred } from '../provider';
import { TexResolver } from '../resolver';
import { BibtexEntryTypeCompletionProvider } from './bibtex/entryType';
import { BibtexFieldNameCompletionProvider } from './bibtex/fieldName';
import { BibtexKernelCommandCompletionProvider } from './bibtex/kernelCommand';
import { DistinctCompletionProvider } from './distinct';
import { LatexBeginCommandCompletionProvider } from './latex/beginCommand';
import { LatexClassImportCompletionProvider } from './latex/classImport';
import { LatexColorCompletionProvider } from './latex/color';
import { LatexColorModelCompletionProvider } from './latex/colorModel';
import { LatexComponentCommandCompletionProvider } from './latex/componentCommand';
import { LatexComponentEnvironmentCompletionProvider } from './latex/componentEnvironment';
import { LatexComponentSource } from './latex/data/component';
import { LatexIncludeCompletionProvider } from './latex/include';
import { LatexKernelCommandProvider } from './latex/kernelCommand';
import { LatexKernelEnvironmentCompletionProvider } from './latex/kernelEnvironment';
import { LatexLabelCompletionProvider } from './latex/label';
import { LatexPgfLibraryCompletionProvider } from './latex/pgfLibrary';
import { LatexTikzLibraryCompletionProvider } from './latex/tikzLibrary';
import { LatexUserCommandCompletionProvider } from './latex/userCommand';
import { LatexUserEnvironmentCompletionProvider } from './latex/userEnvironment';
import { LimitedCompletionProvider } from './limited';
import { OrderByQualityCompletionProvider } from './orderByQuality';
import { CompletionProvider as Provider } from './provider';
import { LatexPackageImportCompletionProvider } from './latex/packageImport';

type Factory = (
  resolver: Promise<TexResolver>,
  database: Promise<LatexComponentSource>,
) => Provider;

export const CompletionProvider: Factory = (resolver, database) =>
  concat(
    LatexIncludeCompletionProvider,
    LimitedCompletionProvider(
      OrderByQualityCompletionProvider(
        DistinctCompletionProvider(
          concat(
            BibtexFieldNameCompletionProvider,
            BibtexEntryTypeCompletionProvider,
            BibtexKernelCommandCompletionProvider,
            LatexPgfLibraryCompletionProvider,
            LatexTikzLibraryCompletionProvider,
            LatexLabelCompletionProvider,
            LatexColorCompletionProvider,
            LatexColorModelCompletionProvider,
            deferred(LatexComponentEnvironmentCompletionProvider, database, []),
            LatexKernelEnvironmentCompletionProvider,
            LatexUserEnvironmentCompletionProvider,
            LatexBeginCommandCompletionProvider,
            deferred(LatexClassImportCompletionProvider, resolver, []),
            deferred(LatexPackageImportCompletionProvider, resolver, []),
            deferred(LatexComponentCommandCompletionProvider, database, []),
            LatexKernelCommandProvider,
            LatexUserCommandCompletionProvider,
          ),
        ),
      ),
    ),
  );
