import Cite = require('citation-js');
import { MarkupContent, MarkupKind } from 'vscode-languageserver';
import { BibtexSyntaxTree } from '../syntax/bibtex/analysis';
import { toMarkdown } from './markdown';

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
    return {
      kind: MarkupKind.Markdown,
      value: toMarkdown(html),
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
