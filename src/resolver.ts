import fs from 'fs';
import path from 'path';
import { ProcessBuilder } from './process';

const TEXLIVE_DATABASE_PATH = 'ls-R';
const MIKTEX_DATABASE_PATH = 'miktex/data/le';
const FNDB_SIGNATURE = 0x42444e46;
const FNDB_WORD_SIZE = 4;
const FNDB_TABLE_POINTER_OFFSET = 4 * FNDB_WORD_SIZE;
const FNDB_TABLE_SIZE_OFFSET = 6 * FNDB_WORD_SIZE;
const FNDB_ENTRY_SIZE = 4 * FNDB_WORD_SIZE;

enum TexDistributionKind {
  Texlive,
  Miktex,
  Unknown,
}

export enum TexDistributionErrorKind {
  KpsewhichNotFound,
  UnknownDistribution,
  InvalidDistribution,
}

export class TexDistributionError extends Error {
  constructor(public kind: TexDistributionErrorKind) {
    super();
  }
}

export interface TexResolver {
  filesByName: Map<string, string>;
}

export async function createResolver(): Promise<TexResolver> {
  const rootDirectories = await findRootDirectories();
  const kind = detectDistribution(rootDirectories);
  if (kind === TexDistributionKind.Unknown) {
    throw new TexDistributionError(
      TexDistributionErrorKind.UnknownDistribution,
    );
  }

  try {
    return { filesByName: await readDatabases(rootDirectories, kind) };
  } catch {
    throw new TexDistributionError(
      TexDistributionErrorKind.InvalidDistribution,
    );
  }
}

async function findRootDirectories(): Promise<Set<string>> {
  try {
    const texmf = await runKpsewhich('-var-value', 'TEXMF');
    const expanded = await runKpsewhich(`--expand-braces=${texmf}`);
    return new Set(
      expanded
        .split(path.delimiter)
        .map(x => x.replace(/!/g, ''))
        .filter(fs.existsSync),
    );
  } catch {
    throw new TexDistributionError(TexDistributionErrorKind.KpsewhichNotFound);
  }
}

async function runKpsewhich(...args: string[]): Promise<string> {
  let line = '';
  await new ProcessBuilder('kpsewhich')
    .args(...args)
    .output(x => (line = x.toString().trim()))
    .start();
  return line;
}

function detectDistribution(
  directories: Iterable<string>,
): TexDistributionKind {
  for (const directory of directories) {
    if (fs.existsSync(path.resolve(directory, TEXLIVE_DATABASE_PATH))) {
      return TexDistributionKind.Texlive;
    }

    if (fs.existsSync(path.resolve(directory, MIKTEX_DATABASE_PATH))) {
      return TexDistributionKind.Miktex;
    }
  }

  return TexDistributionKind.Unknown;
}

async function readDatabases(
  rootDirectories: Iterable<string>,
  kind: TexDistributionKind,
): Promise<Map<string, string>> {
  const filesByName = new Map<string, string>();

  for (const directory of rootDirectories) {
    let database: string[] = [];

    switch (kind) {
      case TexDistributionKind.Texlive: {
        const file = path.resolve(directory, TEXLIVE_DATABASE_PATH);
        if (fs.existsSync(file)) {
          const buffer = await fs.promises.readFile(file);
          const lines = buffer.toString().split(/\r?\n/);
          database = parseTexliveDatabase(directory, lines);
        }

        break;
      }
      case TexDistributionKind.Miktex: {
        const databasePath = path.resolve(directory, MIKTEX_DATABASE_PATH);
        const files = await Promise.all(
          (await fs.promises.readdir(databasePath))
            .filter(x => x.match(/\w+\.fndb-\d+$/))
            .map(x => path.resolve(databasePath, x))
            .map(async x =>
              parseMiktexDatabase(directory, await fs.promises.readFile(x)),
            ),
        );

        database = new Array<string>().concat(...files);
        break;
      }
    }

    database.forEach(x => filesByName.set(path.basename(x), x));
  }

  return filesByName;
}

function parseTexliveDatabase(
  rootDirectory: string,
  lines: string[],
): string[] {
  const results: string[] = [];
  let currentDirectory = '';

  for (const line of lines.filter(x => x.trim() && !x.startsWith('%'))) {
    if (line.endsWith(':')) {
      const relativePath = line.substring(0, line.length - 1);
      currentDirectory = path.resolve(rootDirectory, relativePath);
    } else {
      const file = path.resolve(currentDirectory, line);
      if (path.extname(file)) {
        results.push(file);
      }
    }
  }

  return results;
}

function parseMiktexDatabase(rootDirectory: string, buffer: Buffer): string[] {
  if (buffer.readInt32LE(0) !== FNDB_SIGNATURE) {
    throw new TexDistributionError(
      TexDistributionErrorKind.InvalidDistribution,
    );
  }

  const results: string[] = [];
  const tableAddress = buffer.readInt32LE(FNDB_TABLE_POINTER_OFFSET);
  const tableSize = buffer.readInt32LE(FNDB_TABLE_SIZE_OFFSET);

  for (let i = 0; i < tableSize; i++) {
    const offset = tableAddress + i * FNDB_ENTRY_SIZE;
    const fileName = readString(buffer, buffer.readInt32LE(offset));
    const directory = readString(
      buffer,
      buffer.readInt32LE(offset + FNDB_WORD_SIZE),
    );

    const file = path.resolve(rootDirectory, directory, fileName);
    results.push(file);
  }

  return results;
}

function readString(buffer: Buffer, index: number): string {
  let byte = buffer[index];
  let length = 0;
  while (byte !== 0) {
    length++;
    byte = buffer[index + length];
  }

  return buffer.toString('utf-8', index, index + length);
}
