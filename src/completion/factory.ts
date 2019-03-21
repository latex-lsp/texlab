import {
  CompletionItem,
  CompletionItemKind as LspCompletionItemKind,
  InsertTextFormat,
  MarkupContent,
  MarkupKind,
} from 'vscode-languageserver';
import { BibtexFormatter } from '../formatting/bibtex';
import {
  BibtexField,
  getFieldDocumentation,
  getFieldName,
} from '../metadata/bibtexField';
import { getTypeDocumentation } from '../metadata/bibtexType';
import { BibtexEntrySyntax } from '../syntax/bibtex/ast';

const KERNEL_DETAIL: string = 'built-in';

export const USER_COMPONENT: string = 'unknown';

function getDetail(component: string | undefined) {
  return component === undefined ? KERNEL_DETAIL : component;
}

export enum CompletionItemKind {
  Snippet,
  Command,
  Environment,
  Label,
  Folder,
  File,
  PgfLibrary,
  TikzLibrary,
  Color,
  ColorModel,
  Package,
  Class,
  EntryType,
  FieldName,
  Citation,
  CommandSymbol,
  ArgumentSymbol,
}

// export type CompletionData =
//   | { kind: CompletionItemKind.Snippet }
//   | { kind: CompletionItemKind.Command }
//   | { kind: CompletionItemKind.Environment }
//   | { kind: CompletionItemKind.Label }
//   | { kind: CompletionItemKind.Folder }
//   | { kind: CompletionItemKind.File }
//   | { kind: CompletionItemKind.PgfLibrary }
//   | { kind: CompletionItemKind.TikzLibrary }
//   | { kind: CompletionItemKind.Color }
//   | { kind: CompletionItemKind.ColorModel }
//   | { kind: CompletionItemKind.Package }
//   | { kind: CompletionItemKind.Class }
//   | { kind: CompletionItemKind.EntryType }
//   | { kind: CompletionItemKind.FieldName }
//   | { kind: CompletionItemKind.Citation; entry: string }
//   | { kind: CompletionItemKind.CommandSymbol }
//   | { kind: CompletionItemKind.ArgumentSymbol };

export function createSnippet(
  name: string,
  component: string | undefined,
  template: string,
): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Snippet,
    data: CompletionItemKind.Snippet,
    detail: getDetail(component),
    insertText: template,
    insertTextFormat: InsertTextFormat.Snippet,
  };
}

export function createCommand(
  name: string,
  component: string | undefined,
): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Function,
    data: { kind: CompletionItemKind.Command },
    detail: getDetail(component),
  };
}

export function createEnvironment(
  name: string,
  component: string | undefined,
): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.EnumMember,
    data: { kind: CompletionItemKind.Environment },
    detail: getDetail(component),
  };
}

export function createLabel(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Field,
    data: { kind: CompletionItemKind.Label },
  };
}

export function createFolder(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Folder,
    data: { kind: CompletionItemKind.Folder },
    commitCharacters: ['/'],
  };
}

export function createFile(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.File,
    data: { kind: CompletionItemKind.File },
  };
}

export function createPgfLibrary(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Module,
    data: { kind: CompletionItemKind.PgfLibrary },
    commitCharacters: [' '],
  };
}

export function createTikzLibrary(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Module,
    data: { kind: CompletionItemKind.TikzLibrary },
    commitCharacters: [' '],
  };
}

export function createColor(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Color,
    data: { kind: CompletionItemKind.Color },
  };
}

export function createColorModel(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Color,
    data: { kind: CompletionItemKind.ColorModel },
  };
}

export function createPackage(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Class,
    data: { kind: CompletionItemKind.Package },
  };
}

export function createClass(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Class,
    data: { kind: CompletionItemKind.Class },
  };
}

export function createEntryType(name: string): CompletionItem {
  const markdown = getTypeDocumentation(name);
  const documentation =
    markdown === undefined
      ? undefined
      : { kind: MarkupKind.Markdown, value: markdown };

  return {
    label: name,
    kind: LspCompletionItemKind.Interface,
    data: { kind: CompletionItemKind.EntryType },
    documentation,
  };
}

export function createCitation(entry: BibtexEntrySyntax): CompletionItem {
  const formatter = new BibtexFormatter(true, 4, 0);
  return {
    label: entry.name!.text,
    kind: LspCompletionItemKind.Constant,
    data: {
      kind: CompletionItemKind.Citation,
      entry: formatter.format(entry),
    },
  };
}

export function createFieldName(field: BibtexField): CompletionItem {
  return {
    label: getFieldName(field),
    kind: LspCompletionItemKind.Field,
    data: { kind: CompletionItemKind.FieldName },
    documentation: {
      kind: MarkupKind.Markdown,
      value: getFieldDocumentation(field),
    },
  };
}

export function createCommandSymbol(
  name: string,
  component: string | undefined,
  image: string,
): CompletionItem {
  const detail = getDetail(component);
  return {
    label: name,
    detail,
    kind: LspCompletionItemKind.Function,
    data: { kind: CompletionItemKind.CommandSymbol },
    documentation: createImage(name, image),
  };
}

export function createArgumentSymbol(
  name: string,
  image: string,
): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Field,
    data: { kind: CompletionItemKind.ArgumentSymbol },
    documentation: createImage(name, image),
  };
}

function createImage(name: string, image: string): MarkupContent {
  return {
    kind: MarkupKind.Markdown,
    value: `![${name}](data:image/png;base64,${image}|width=48,height=48)`,
  };
}
