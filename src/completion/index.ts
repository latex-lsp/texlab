import { concat, deferred } from '../provider';
import { TexResolver } from '../resolver';
import { BibtexEntryTypeCompletionProvider } from './bibtex/entryType';
import { BibtexFieldNameCompletionProvider } from './bibtex/fieldName';
import { BibtexKernelCommandCompletionProvider } from './bibtex/kernelCommand';
import { DistinctCompletionProvider } from './distinct';
import { LatexBeginCommandCompletionProvider } from './latex/beginCommand';
import { LatexClassImportCompletionProvider } from './latex/classImport';
import { LatexColorModelCompletionProvider } from './latex/colorModel';
import { LatexIncludeCompletionProvider } from './latex/include';
import { LatexKernelCommandProvider } from './latex/kernelCommand';
import { LatexUserCommandCompletionProvider } from './latex/userCommand';
import { LimitedCompletionProvider } from './limited';
import { OrderByQualityCompletionProvider } from './orderByQuality';
import { CompletionProvider as Provider } from './provider';

type CompletionProviderFactory = (resolver: Promise<TexResolver>) => Provider;

export const CompletionProvider: CompletionProviderFactory = resolver =>
  concat(
    LatexIncludeCompletionProvider,
    LimitedCompletionProvider(
      OrderByQualityCompletionProvider(
        DistinctCompletionProvider(
          concat(
            BibtexFieldNameCompletionProvider,
            BibtexEntryTypeCompletionProvider,
            BibtexKernelCommandCompletionProvider,
            LatexColorModelCompletionProvider,
            deferred(LatexClassImportCompletionProvider, resolver, []),
            LatexBeginCommandCompletionProvider,
            LatexKernelCommandProvider,
            LatexUserCommandCompletionProvider,
          ),
        ),
      ),
    ),
  );
