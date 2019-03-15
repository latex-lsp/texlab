import { concat } from '../../provider';
import * as factory from '../factory';
import { CompletionProvider } from '../provider';
import { LatexArgumentCompletionProvider } from './argument';

export const COLOR_MODELS = ['gray', 'rgb', 'RGB', 'HTML', 'cmyk'];

const ITEMS = COLOR_MODELS.map(factory.createColorModel);

export const LatexColorModelCompletionProvider: CompletionProvider = concat(
  LatexArgumentCompletionProvider({
    commandNames: ['\\definecolor'],
    argumentIndex: 1,
    execute: async () => ITEMS,
  }),
  LatexArgumentCompletionProvider({
    commandNames: ['\\definecolorset'],
    argumentIndex: 0,
    execute: async () => ITEMS,
  }),
);
