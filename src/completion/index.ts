import { concat } from '../provider';
import { BibtexEntryTypeCompletionProvider } from './bibtex/entryType';
import { BibtexFieldNameCompletionProvider } from './bibtex/fieldName';
import { BibtexKernelCommandCompletionProvider } from './bibtex/kernelCommand';
import { DistinctCompletionProvider } from './distinct';
import { LatexKernelCommandProvider } from './latex/kernelCommand';
import { LimitedCompletionProvider } from './limited';
import { OrderByQualityCompletionProvider } from './orderByQuality';

export const completionProvider = LimitedCompletionProvider(
  OrderByQualityCompletionProvider(
    DistinctCompletionProvider(
      concat(
        BibtexFieldNameCompletionProvider,
        BibtexEntryTypeCompletionProvider,
        BibtexKernelCommandCompletionProvider,
        LatexKernelCommandProvider,
      ),
    ),
  ),
);
