import TurndownService from 'turndown';

const turndownService = new TurndownService();

export function toMarkdown(html: string): string {
  return turndownService.turndown(html);
}
