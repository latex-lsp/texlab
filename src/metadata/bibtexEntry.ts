import Cite = require('citation-js');
import * as TurndownService from 'turndown';
import { MarkupContent, MarkupKind } from 'vscode-languageserver';
import { BibtexSyntaxTree } from '../syntax/bibtex/analysis';

const turndownService = new TurndownService();

export function generateBibliography(code: string): MarkupContent | undefined {
  if (!validate(code)) {
    return undefined;
  }

  try {
    const citation = new Cite(code);
    const html = citation.format('bibliography', {
      format: 'html',
      template: 'apa',
      lang: 'en-US',
    });
    const markdown = turndownService.turndown(html);
    return {
      kind: MarkupKind.Markdown,
      value: markdown,
    };
  } catch (err) {
    return undefined;
  }
}

function validate(code: string): boolean {
  const tree = new BibtexSyntaxTree(code);
  const entry = tree.entries[0];
  if (entry.right === undefined) {
    return false;
  }

  if (entry.fields.length === 0) {
    return false;
  }

  return true;
}
