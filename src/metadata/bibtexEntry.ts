import Cite = require('citation-js');
import * as TurndownService from 'turndown';
import { MarkupContent, MarkupKind } from 'vscode-languageserver';

const turndownService = new TurndownService();

export function generateBibliography(entry: string): MarkupContent | undefined {
  try {
    const citation = new Cite(entry);
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
