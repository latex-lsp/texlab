import { Position, Range } from 'vscode-languageserver';
import { Language } from '../../language';
import * as range from '../../range';
import {
  descendants,
  LatexCommandSyntax,
  LatexDocumentSyntax,
  LatexSyntaxNode,
  LatexTextSyntax,
} from './ast';
import { LatexToken, LatexTokenKind } from './lexer';
import { parse } from './parser';

export const INCLUDE_COMMANDS = [
  '\\include',
  '\\input',
  '\\bibliography',
  '\\addbibresource',
  '\\usepackage',
  '\\documentclass',
];

export const LABEL_DEFINITION_COMMANDS = ['\\label'];

export const LABEL_REFERENCE_COMMANDS = ['\\ref', '\\autoref', '\\eqref'];

export const CITATION_COMMANDS = [
  '\\cite',
  '\\cite*',
  '\\Cite',
  '\\nocite',
  '\\citet',
  '\\citep',
  '\\citet*',
  '\\citep*',
  '\\citeauthor',
  '\\citeauthor*',
  '\\Citeauthor',
  '\\Citeauthor*',
  '\\citetitle',
  '\\citetitle*',
  '\\citeyear',
  '\\citeyear*',
  '\\citedate',
  '\\citedate*',
  '\\citeurl',
  '\\fullcite',
  '\\citeyearpar',
  '\\citealt',
  '\\citealp',
  '\\citetext',
  '\\parencite',
  '\\parencite*',
  '\\Parencite',
  '\\footcite',
  '\\footfullcite',
  '\\footcitetext',
  '\\textcite',
  '\\Textcite',
  '\\smartcite',
  '\\Smartcite',
  '\\supercite',
  '\\autocite',
  '\\Autocite',
  '\\autocite*',
  '\\Autocite*',
  '\\volcite',
  '\\Volcite',
  '\\pvolcite',
  '\\Pvolcite',
  '\\fvolcite',
  '\\ftvolcite',
  '\\svolcite',
  '\\Svolcite',
  '\\tvolcite',
  '\\Tvolcite',
  '\\avolcite',
  '\\Avolcite',
  '\\notecite',
  '\\notecite',
  '\\pnotecite',
  '\\Pnotecite',
  '\\fnotecite',
];

export const SECTION_COMMANDS = [
  '\\chapter',
  '\\chapter*',
  '\\section',
  '\\section*',
  '\\subsection',
  '\\subsection*',
  '\\subsubsection',
  '\\subsubsection*',
  '\\paragraph',
  '\\paragraph*',
  '\\subparagraph',
  '\\subparagraph*',
];

export const ENVIRONMENT_COMMANDS = ['\\begin', '\\end'];

export const EQUATION_COMMANDS = ['\\[', '\\]'];

export interface LatexInclude {
  command: LatexCommandSyntax;
  path: string;
  isUnitImport: boolean;
}

export interface LatexLabel {
  command: LatexCommandSyntax;
  name: LatexToken;
}

export interface LatexCitation {
  command: LatexCommandSyntax;
  name: LatexToken;
}

export interface LatexSection {
  command: LatexCommandSyntax;
  text: string;
  level: number;
}

export class LatexEnvironment {
  public readonly beginName: string;
  public readonly endName: string;
  public readonly beginNameRange: Range;
  public readonly endNameRange: Range;
  public readonly range: Range;

  constructor(
    public readonly begin: LatexCommandSyntax,
    public readonly end: LatexCommandSyntax,
  ) {
    this.beginName = this.getName(begin);
    this.endName = this.getName(end);
    this.beginNameRange = this.getNameRange(begin);
    this.endNameRange = this.getNameRange(end);
    this.range = { start: begin.start, end: end.end };
  }

  private getName(delimiter: LatexCommandSyntax): string {
    const name = delimiter.extractWord(0);
    return name !== undefined ? name.text : '';
  }

  private getNameRange(delimiter: LatexCommandSyntax): Range {
    const group = delimiter.args[0];
    return group.children.length > 0
      ? group.children[0].range
      : { start: group.left.end, end: group.left.end };
  }
}

enum LatexEnvironmentDelimiterKind {
  Begin,
  End,
}

interface LatexEnvironmentDelimiter {
  command: LatexCommandSyntax;
  kind: LatexEnvironmentDelimiterKind;
}

export class LatexEquation {
  public readonly range: Range;

  constructor(
    public readonly begin: LatexCommandSyntax,
    public readonly end: LatexCommandSyntax,
  ) {
    this.range = { start: begin.start, end: end.end };
  }
}

export class LatexInline {
  public readonly range: Range;

  constructor(
    public readonly begin: LatexToken,
    public readonly end: LatexToken,
  ) {
    this.range = { start: begin.start, end: end.end };
  }
}

export class LatexSyntaxTree {
  public readonly language: Language.Latex;
  public readonly root: LatexDocumentSyntax;
  public readonly descendants: LatexSyntaxNode[];
  public readonly includes: LatexInclude[];
  public readonly labelDefinitions: LatexLabel[];
  public readonly labelReferences: LatexLabel[];
  public readonly citations: LatexCitation[];
  public readonly sections: LatexSection[];
  public readonly components: string[];
  public readonly environments: LatexEnvironment[];
  public readonly equations: LatexEquation[];
  public readonly inlines: LatexInline[];
  public readonly isStandalone: boolean;

  constructor(public readonly text: string) {
    this.language = Language.Latex;
    this.root = parse(text);
    this.descendants = descendants(this.root);
    this.includes = [];
    this.labelDefinitions = [];
    this.labelReferences = [];
    this.citations = [];
    this.sections = [];
    this.components = [];
    this.environments = [];
    this.equations = [];
    this.inlines = [];
    this.analyze();
    this.isStandalone = this.environments.some(
      x => x.beginName === 'document' || x.endName === 'document',
    );
  }

  public find(position: Position): LatexSyntaxNode | undefined {
    for (let i = this.descendants.length - 1; i >= 0; i--) {
      const node = this.descendants[i];
      if (range.contains(node, position)) {
        return node;
      }
    }
    return undefined;
  }

  private analyze() {
    const environmentStack: LatexEnvironmentDelimiter[] = [];
    let beginEquation: LatexCommandSyntax | undefined;
    this.descendants.filter(LatexCommandSyntax.is).forEach(command => {
      if (INCLUDE_COMMANDS.includes(command.name.text)) {
        const text = command.extractText(0);
        if (text !== undefined) {
          const path = text.words.map(word => word.text).join(' ');
          this.includes.push({
            command,
            path,
            isUnitImport:
              command.name.text === '\\usepackage' ||
              command.name.text === '\\documentclass',
          });
        }
      } else if (LABEL_DEFINITION_COMMANDS.includes(command.name.text)) {
        const name = command.extractWord(0);
        if (name !== undefined) {
          this.labelDefinitions.push({ command, name });
        }
      } else if (LABEL_REFERENCE_COMMANDS.includes(command.name.text)) {
        const name = command.extractWord(0);
        if (name !== undefined) {
          this.labelReferences.push({ command, name });
        }
      } else if (CITATION_COMMANDS.includes(command.name.text)) {
        const name = command.extractWord(0);
        if (name !== undefined) {
          this.citations.push({ command, name });
        }
      } else if (SECTION_COMMANDS.includes(command.name.text)) {
        const text = command.extractText(0);
        if (text !== undefined) {
          const index = SECTION_COMMANDS.indexOf(command.name.text);
          const level = Math.floor(index / 2);
          this.sections.push({
            command,
            text: text.words.map(word => word.text).join(' '),
            level,
          });
        }
      } else if (ENVIRONMENT_COMMANDS.includes(command.name.text)) {
        function createDelimiter(): LatexEnvironmentDelimiter | undefined {
          const kind =
            command.name.text === ENVIRONMENT_COMMANDS[0]
              ? LatexEnvironmentDelimiterKind.Begin
              : LatexEnvironmentDelimiterKind.End;

          if (
            command.args.length > 0 &&
            command.args[0].children.length === 0
          ) {
            return { command, kind };
          }

          const name = command.extractWord(0);
          return name === undefined ? undefined : { command, kind };
        }

        const delimiter = createDelimiter();
        if (delimiter !== undefined) {
          if (delimiter.kind === LatexEnvironmentDelimiterKind.Begin) {
            environmentStack.push(delimiter);
          } else if (environmentStack.length > 0) {
            this.environments.push(
              new LatexEnvironment(
                environmentStack.pop()!.command,
                delimiter.command,
              ),
            );
          }
        }
      } else if (EQUATION_COMMANDS.includes(command.name.text)) {
        if (command.name.text === EQUATION_COMMANDS[0]) {
          beginEquation = command;
        } else if (beginEquation !== undefined) {
          this.equations.push(new LatexEquation(beginEquation, command));
        }
      }
    });

    this.includes.forEach(include => {
      switch (include.command.name.text) {
        case '\\usepackage':
          this.components.push(include.path + '.sty');
          break;
        case '\\documentclass':
          this.components.push(include.path + '.cls');
          break;
      }
    });

    let beginInline: LatexToken | undefined;
    this.descendants.filter(LatexTextSyntax.is).forEach(text => {
      text.words
        .filter(x => x.kind === LatexTokenKind.Math)
        .forEach(math => {
          if (beginInline === undefined) {
            beginInline = math;
          } else {
            this.inlines.push(new LatexInline(beginInline, math));
            beginInline = undefined;
          }
        });
    });
  }
}
