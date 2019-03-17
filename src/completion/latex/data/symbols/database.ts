import * as fs from 'fs';
import * as path from 'path';
import * as data from './symbols.json';

export interface LatexCommandSymbol {
  command: string;
  component: string | null;
  image: string;
}

export interface LatexArgumentSymbolGroup {
  command: string;
  component: string | null;
  index: number;
  arguments: LatexArgumentSymbol[];
}

export interface LatexArgumentSymbol {
  argument: string;
  image: string;
}

export interface LatexSymbolDatabase {
  commands: LatexCommandSymbol[];
  arguments: LatexArgumentSymbolGroup[];
}

export const DATABASE_FILE = path.join(__dirname, 'symbols.json');

export const SYMBOLS: LatexSymbolDatabase = data
  ? (data as LatexSymbolDatabase)
  : JSON.parse(fs.readFileSync(DATABASE_FILE).toString());
