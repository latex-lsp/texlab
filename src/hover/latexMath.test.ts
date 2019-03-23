import { runSingleFile } from '../workspaceBuilder';
import { LatexMathHoverProvider } from './latexMath';

describe('LatexMathHoverProvider', () => {
  const provider = LatexMathHoverProvider;

  it('should render math when hovering over inline math', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '$x$',
      line: 0,
      character: 1,
    });

    expect(actual).toBeDefined();
  });

  it('should render math when hovering over an equation', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\[ x \\]',
      line: 0,
      character: 1,
    });

    expect(actual).toBeDefined();
  });

  it('should render math when hovering over a math environment', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: '\\begin{align*} x &= 0 \\ x^2 &= 0 \\end{align*}',
      line: 0,
      character: 1,
    });

    expect(actual).toBeDefined();
  });

  it('should not render math when hovering outside of a math expression', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.tex',
      text: 'foo $x$',
      line: 0,
      character: 1,
    });

    expect(actual).toBeUndefined();
  });

  it('should not render math when hovering inside of a BibTeX document', async () => {
    const actual = await runSingleFile({
      provider,
      file: 'foo.bib',
      text: '@article',
      line: 0,
      character: 1,
    });

    expect(actual).toBeUndefined();
  });
});
