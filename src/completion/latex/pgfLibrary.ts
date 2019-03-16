import * as factory from '../factory';
import { CompletionProvider } from '../provider';
import { LatexArgumentCompletionProvider } from './argument';

export const LIBRARIES = [
  'arrows',
  'arrows.meta',
  'arrows.spaced',
  'curvilinear',
  'datavisualization.barcharts',
  'datavisualization.formats.functions',
  'datavisualization.polar',
  'decorations.footprints',
  'decorations.fractals',
  'decorations.markings',
  'decorations.pathmorphing',
  'decorations.pathreplacing',
  'decorations.shapes',
  'decorations.text',
  'fadings',
  'fixedpointarithmetic',
  'fpu',
  'intersections',
  'lindenmayersystems',
  'luamath',
  'patterns',
  'patterns.meta',
  'plothandlers',
  'plotmarks',
  'profiler',
  'shadings',
  'shapes.arrows',
  'shapes.callouts',
  'shapes',
  'shapes.gates.ee',
  'shapes.gates.ee.IEC',
  'shapes.gates.logic',
  'shapes.gates.logic.IEC',
  'shapes.gates.logic.US',
  'shapes.geometric',
  'shapes.misc',
  'shapes.multipart',
  'shapes.symbols',
  'snakes',
  'svg.path',
];

const ITEMS = LIBRARIES.map(factory.createPgfLibrary);

export const LatexPgfLibraryCompletionProvider: CompletionProvider = LatexArgumentCompletionProvider(
  {
    commandNames: ['\\usepgflibrary'],
    argumentIndex: 0,
    execute: async () => ITEMS,
  },
);
