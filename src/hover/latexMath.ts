import { MarkupKind, Range } from 'vscode-languageserver';
import { Language } from '../language';
import { contains } from '../range';
import { CharStream } from '../syntax/charStream';
import { LatexEnvironment, LatexInline } from '../syntax/latex/analysis';
import { renderMath } from './latexMathRenderer';
import { HoverProvider } from './provider';

export const MATH_ENVIRONMENTS = [
  'align',
  'align',
  'alignat',
  'aligned',
  'alignedat',
  'array',
  'Bmatrix',
  'bmatrix',
  'cases',
  'CD',
  'eqnarray',
  'equation',
  'equation',
  'gather',
  'gathered',
  'matrix',
  'multline',
  'pmatrix',
  'smallmatrix',
  'split',
  'subarray',
  'Vmatrix',
  'vmatrix',
];

export const LatexMathHoverProvider: HoverProvider = {
  execute: async context => {
    const { document, params } = context;
    const { tree } = document;
    if (tree.language !== Language.Latex) {
      return undefined;
    }

    const elements = [
      ...tree.environments.filter(x =>
        MATH_ENVIRONMENTS.includes(x.beginName.replace('*', '')),
      ),
      ...tree.equations,
      ...tree.inlines,
    ];

    const element = elements.find(x => contains(x.range, params.position));
    if (element === undefined) {
      return undefined;
    }

    const range =
      element instanceof LatexEnvironment
        ? element.range
        : Range.create(element.begin.end, element.end.start);

    const code = extractText(tree.text, range);
    try {
      const svg = renderMath(code, element instanceof LatexInline);
      const encodedSvg = Buffer.from(svg).toString('base64');
      return {
        contents: {
          kind: MarkupKind.Markdown,
          value: `![math](data:image/svg+xml;base64,${encodedSvg})`,
        },
      };
    } catch {
      return undefined;
    }
  },
};

function extractText(text: string, range: Range): string {
  const stream = new CharStream(text);
  stream.seek(range.start);
  const startIndex = stream.index;

  stream.seek(range.end);
  const endIndex = stream.index;

  return text.substring(startIndex, endIndex);
}
