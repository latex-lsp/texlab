import * as factory from '../factory';
import { KERNEL_ENVIRONMENTS } from '../kernel';
import { CompletionProvider } from '../provider';
import { LatexEnvironmentCompletionProvider } from './environment';

const ITEMS = KERNEL_ENVIRONMENTS.map(x =>
  factory.createEnvironment(x, undefined),
);

export const LatexKernelEnvironmentCompletionProvider: CompletionProvider = LatexEnvironmentCompletionProvider(
  {
    execute: async () => ITEMS,
  },
);
