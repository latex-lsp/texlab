import {
  LatexCitation,
  LatexEnvironment,
  LatexEquation,
  LatexInclude,
  LatexInline,
  LatexLabel,
  LatexSection,
  LatexSyntaxTree,
} from './analysis';
import { LatexCommandSyntax } from './ast';
import { LatexToken, LatexTokenKind } from './lexer';

describe('LaTeX Analysis', () => {
  it('should parse includes with spaces', () => {
    const text = '\\include{a b c}';
    const tree = new LatexSyntaxTree(text);
    const include: LatexInclude = {
      command: tree.root.children[0] as LatexCommandSyntax,
      path: 'a b c',
      isUnitImport: false,
    };
    expect(tree.includes).toEqual([include]);
  });

  it('should ignore invalid includes', () => {
    const text = '\\input';
    const tree = new LatexSyntaxTree(text);
    expect(tree.includes).toEqual([]);
  });

  it('should parse component imports', () => {
    const text = '\\documentclass{article}\n\\usepackage{amsmath}';
    const tree = new LatexSyntaxTree(text);
    const includes: LatexInclude[] = [
      {
        command: tree.root.children[0] as LatexCommandSyntax,
        path: 'article',
        isUnitImport: true,
      },
      {
        command: tree.root.children[1] as LatexCommandSyntax,
        path: 'amsmath',
        isUnitImport: true,
      },
    ];
    expect(tree.includes).toEqual(includes);
    expect(tree.components).toEqual(['article.cls', 'amsmath.sty']);
  });

  it('should parse label definitions', () => {
    const text = '\\label{foo}';
    const tree = new LatexSyntaxTree(text);
    const label: LatexLabel = {
      command: tree.root.children[0] as LatexCommandSyntax,
      name: new LatexToken(
        { line: 0, character: 7 },
        'foo',
        LatexTokenKind.Word,
      ),
    };
    expect(tree.labelDefinitions).toEqual([label]);
  });

  it('should ignore invalid label definitions', () => {
    const text = '\\label\\label{}\\label{\\foo}';
    const tree = new LatexSyntaxTree(text);
    expect(tree.labelDefinitions).toEqual([]);
  });

  it('should parse label references', () => {
    const text = '\\ref[bar]{foo}';
    const tree = new LatexSyntaxTree(text);
    const label: LatexLabel = {
      command: tree.root.children[0] as LatexCommandSyntax,
      name: new LatexToken(
        { line: 0, character: 10 },
        'foo',
        LatexTokenKind.Word,
      ),
    };
    expect(tree.labelReferences).toEqual([label]);
  });

  it('should ignore invalid label references', () => {
    const text = '\\ref\\autoref{}';
    const tree = new LatexSyntaxTree(text);
    expect(tree.labelReferences).toEqual([]);
  });

  it('should parse citations', () => {
    const text = '\\cite{foo}';
    const tree = new LatexSyntaxTree(text);
    const citation: LatexCitation = {
      command: tree.root.children[0] as LatexCommandSyntax,
      name: new LatexToken(
        { line: 0, character: 6 },
        'foo',
        LatexTokenKind.Word,
      ),
    };
    expect(tree.citations).toEqual([citation]);
  });

  it('should ignore invalid citations', () => {
    const text = '\\cite\\nocite{}';
    const tree = new LatexSyntaxTree(text);
    expect(tree.citations).toEqual([]);
  });

  it('should parse sections', () => {
    const text = '\\section{foo}}\n\\subsection{bar}';
    const tree = new LatexSyntaxTree(text);
    const sections: LatexSection[] = [
      {
        command: tree.root.children[0] as LatexCommandSyntax,
        text: 'foo',
        level: 1,
      },
      {
        command: tree.root.children[1] as LatexCommandSyntax,
        text: 'bar',
        level: 2,
      },
    ];
    expect(tree.sections).toEqual(sections);
  });

  it('should ignore invalid sections', () => {
    const text = '\\section\\subsection{}';
    const tree = new LatexSyntaxTree(text);
    expect(tree.sections).toEqual([]);
  });

  it('should parse nested environments', () => {
    const text = '\\begin{a}\\begin{b}\\end{c}\\end{d}';
    const tree = new LatexSyntaxTree(text);
    const environment1 = new LatexEnvironment(
      tree.root.children[1] as LatexCommandSyntax,
      tree.root.children[2] as LatexCommandSyntax,
    );
    const environment2 = new LatexEnvironment(
      tree.root.children[0] as LatexCommandSyntax,
      tree.root.children[3] as LatexCommandSyntax,
    );
    expect(tree.environments).toEqual([environment1, environment2]);
  });

  it('should parse environments with empty names', () => {
    const text = '\\begin{}\\end{}';
    const tree = new LatexSyntaxTree(text);
    const environment = new LatexEnvironment(
      tree.root.children[0] as LatexCommandSyntax,
      tree.root.children[1] as LatexCommandSyntax,
    );
    expect(tree.environments).toEqual([environment]);
  });

  it('should ignore unmatched environment delimiters', () => {
    const text = '\\end{a}\\begin{b}';
    const tree = new LatexSyntaxTree(text);
    expect(tree.environments).toEqual([]);
  });

  it('should ignore invalid environment delimiters', () => {
    const text = '\\begin\\end';
    const tree = new LatexSyntaxTree(text);
    expect(tree.environments).toEqual([]);
  });

  it('should parse equations', () => {
    const text = '\\[ x \\]';
    const tree = new LatexSyntaxTree(text);
    const equation = new LatexEquation(
      tree.root.children[0] as LatexCommandSyntax,
      tree.root.children[2] as LatexCommandSyntax,
    );
    expect(tree.equations).toEqual([equation]);
  });

  it('should ignore invalid equations', () => {
    const text = '\\] \\[';
    const tree = new LatexSyntaxTree(text);
    expect(tree.equations).toEqual([]);
  });

  it('should parse inline math', () => {
    const text = '$ x $ $ y $';
    const tree = new LatexSyntaxTree(text);
    const inlines = [
      new LatexInline(
        new LatexToken({ line: 0, character: 0 }, '$', LatexTokenKind.Math),
        new LatexToken({ line: 0, character: 4 }, '$', LatexTokenKind.Math),
      ),
      new LatexInline(
        new LatexToken({ line: 0, character: 6 }, '$', LatexTokenKind.Math),
        new LatexToken({ line: 0, character: 10 }, '$', LatexTokenKind.Math),
      ),
    ];
    expect(tree.inlines).toEqual(inlines);
  });

  it('should ignore invalid inline math', () => {
    const text = '$';
    const tree = new LatexSyntaxTree(text);
    expect(tree.inlines).toEqual([]);
  });

  it('should ignore unknown commands', () => {
    const text = '\\foo';
    const tree = new LatexSyntaxTree(text);
    expect(tree.includes).toEqual([]);
    expect(tree.labelDefinitions).toEqual([]);
    expect(tree.labelReferences).toEqual([]);
    expect(tree.citations).toEqual([]);
    expect(tree.sections).toEqual([]);
    expect(tree.components).toEqual([]);
    expect(tree.environments).toEqual([]);
    expect(tree.equations).toEqual([]);
    expect(tree.inlines).toEqual([]);
    expect(tree.isStandalone).toBeFalsy();
  });

  it('should detect if the document is standalone', () => {
    const text = '\\begin{document}\\end{document}';
    const tree = new LatexSyntaxTree(text);
    expect(tree.isStandalone).toBeTruthy();
  });
});
