import fs from 'fs';
import path from 'path';
import {
  CompletionItem,
  TextDocumentPositionParams,
} from 'vscode-languageserver';
import { FeatureContext } from '../../provider';
import { LatexCommandSyntax } from '../../syntax/latex/ast';
import * as factory from '../factory';
import { CompletionProvider } from '../provider';
import { LatexArgumentCompletionProvider } from './argument';

const NO_EXTENSION_COMMANDS = ['\\include', '\\includesvg'];

export const LatexIncludeCompletionProvider: CompletionProvider = LatexArgumentCompletionProvider(
  {
    commandNames: [
      '\\include',
      '\\input',
      '\\bibliography',
      '\\addbibresource',
      '\\includegraphics',
      '\\includesvg',
    ],
    argumentIndex: 0,
    execute: async (context, command) => {
      try {
        const items: CompletionItem[] = [];
        const directory = getCurrentDirectory(context, command);
        for (let entry of await fs.promises.readdir(directory)) {
          try {
            const stats = await fs.promises.lstat(path.join(directory, entry));
            if (stats.isFile() && isIncluded(command, entry)) {
              if (NO_EXTENSION_COMMANDS.includes(command.name.text)) {
                entry = removeExtension(entry);
              }
              items.push(factory.createFile(entry));
            } else if (stats.isDirectory()) {
              items.push(factory.createFolder(entry));
            }
          } catch {}
        }
        return items;
      } catch {
        return [];
      }
    },
  },
);

function getCurrentDirectory(
  context: FeatureContext<TextDocumentPositionParams>,
  command: LatexCommandSyntax,
): string {
  const basePath = path.dirname(context.uri.fsPath);
  const include = command.extractText(0);
  if (include === undefined) {
    return basePath;
  }

  const relativePath = include.words.map(x => x.text).join(' ');
  const fullPath = path.normalize(path.resolve(basePath, relativePath));
  return relativePath.endsWith('/') ? fullPath : path.dirname(fullPath);
}

function isIncluded(command: LatexCommandSyntax, file: string): boolean {
  const extension = path.extname(file).toLowerCase();
  return getAllowedExtensions(command).includes(extension);
}

function getAllowedExtensions(command: LatexCommandSyntax): string[] {
  switch (command.name.text) {
    case '\\include':
    case '\\input':
      return ['.tex'];
    case '\\bibliography':
    case '\\addbibresource':
      return ['.bib'];
    case '\\includegraphics':
      return ['.pdf', '.png', '.jpg', '.jpeg', '.bmp'];
    case '\\includesvg':
      return ['.svg'];
    default:
      return [];
  }
}

function removeExtension(name: string) {
  const index = name.lastIndexOf('.');
  return index === -1 ? name : name.substring(0, index);
}
