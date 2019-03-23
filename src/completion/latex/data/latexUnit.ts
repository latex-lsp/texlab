import fs from 'fs';
import os from 'os';
import path from 'path';
import { TexResolver } from '../../../resolver';
import { compile, TexFormat } from './compile';

enum LatexUnitKind {
  Style,
  Class,
}

export interface LatexPrimitives {
  commands: string[];
  environments: string[];
}

export class LatexUnit {
  public static async load(
    file: string,
    resolver: TexResolver,
  ): Promise<LatexUnit> {
    const extension = path.extname(file);
    const name = path.basename(file, extension);

    let kind = LatexUnitKind.Style;
    if (extension === '.cls') {
      kind = LatexUnitKind.Class;
    }

    let format = TexFormat.Latex;
    if (file.includes('lua')) {
      format = TexFormat.Lualatex;
    } else if (file.includes('xe')) {
      format = TexFormat.Xelatex;
    }

    const code = buildCode(
      buildCodeHeader(name, kind),
      '\\listfiles',
      '\\begin{document}',
      '\\end{document}',
    );

    const log = (await compile(code, format)) || '';
    const includes = extractIncludes(log, kind, resolver);
    const references = includes.filter(
      x => path.extname(x) === '.sty' && x !== file,
    );

    const likelyPrimitives = await getLikelyPrimitives(includes);
    return new LatexUnit(file, references, likelyPrimitives, kind, format);
  }

  public get name(): string {
    const extension = path.extname(this.file);
    return path.basename(this.file, extension);
  }

  constructor(
    public file: string,
    public references: string[],
    public likelyPrimitives: Iterable<string>,
    private kind: LatexUnitKind,
    private format: TexFormat,
  ) {}

  public async checkPrimitives(candidates: string[]): Promise<LatexPrimitives> {
    if (candidates.length === 0) {
      return { commands: [], environments: [] };
    }

    const code = buildCode(
      buildCodeHeader(this.name, this.kind),
      '\\usepackage{etoolbox}',
      '\\begin{document}',
      ...candidates.map(
        candidate => `
          \\ifcsundef{${candidate}}{} { \
          \\ifcsundef{end${candidate}} \
          { \\wlog{cmd:${candidate}} } \
          { \\wlog{env:${candidate}} } }
        `,
      ),
      '\\end{document}',
    );

    const log = (await compile(code, this.format)) || '';
    const commands: string[] = [];
    const environments: string[] = [];

    for (const line of log.split(/\r?\n/)) {
      if (line.startsWith('cmd:')) {
        commands.push(line.split(':')[1]);
      } else if (line.startsWith('env:')) {
        environments.push(line.split(':')[1]);
      }
    }

    return { commands, environments };
  }
}

const FILE_REGEX = /[a-zA-Z0-9_\-\.]+\.(sty|tex|def|cls)/g;
const PRIMITIVE_REGEX = /[a-zA-Z]+/g;

function buildCodeHeader(name: string, kind: LatexUnitKind): string {
  return kind === LatexUnitKind.Style
    ? `\\documentclass{article} \\usepackage{${name}}`
    : `\\documentclass{${name}}`;
}

function buildCode(...code: string[]): string {
  return code.join(os.EOL);
}

function extractIncludes(
  log: string,
  kind: LatexUnitKind,
  resolver: TexResolver,
): string[] {
  const startIndex = log.indexOf('*File List*');
  if (startIndex < 0) {
    return [];
  }

  return (log.match(FILE_REGEX) || [])
    .filter(x => x !== 'article.cls' || kind === LatexUnitKind.Class)
    .map(x => resolver.filesByName.get(x))
    .filter((x): x is string => x !== undefined);
}

async function getLikelyPrimitives(
  includes: string[],
): Promise<Iterable<string>> {
  const primitives = new Set<string>();

  const buffers = await Promise.all(includes.map(x => fs.promises.readFile(x)));
  const matches = buffers
    .map(x => x.toString())
    .join()
    .match(PRIMITIVE_REGEX);

  if (matches) {
    matches.forEach(x => primitives.add(x));
  }

  return primitives;
}
