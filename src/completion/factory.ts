import {
  CompletionItem,
  CompletionItemKind as LspCompletionItemKind,
  InsertTextFormat,
  MarkupContent,
  MarkupKind,
} from 'vscode-languageserver';
import {
  BibtexField,
  getFieldDocumentation,
  getFieldName,
} from '../metadata/bibtexField';
import { getTypeDocumentation } from '../metadata/bibtexType';

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
  CommandSymbol,
  ArgumentSymbol,
  Image,
}

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
    data: CompletionItemKind.Command,
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
    data: CompletionItemKind.Environment,
    detail: getDetail(component),
  };
}

export function createLabel(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Field,
    data: CompletionItemKind.Label,
  };
}

export function createFolder(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Folder,
    data: CompletionItemKind.Folder,
    commitCharacters: ['/'],
  };
}

export function createFile(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.File,
    data: CompletionItemKind.File,
  };
}

export function createPgfLibrary(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Module,
    data: CompletionItemKind.PgfLibrary,
    commitCharacters: [' '],
  };
}

export function createTikzLibrary(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Module,
    data: CompletionItemKind.TikzLibrary,
    commitCharacters: [' '],
  };
}

export function createColor(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Color,
    data: CompletionItemKind.Color,
  };
}

export function createColorModel(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Color,
    data: CompletionItemKind.ColorModel,
  };
}

export function createPackage(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Class,
    data: CompletionItemKind.Package,
  };
}

export function createClass(name: string): CompletionItem {
  return {
    label: name,
    kind: LspCompletionItemKind.Class,
    data: CompletionItemKind.Class,
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
    data: CompletionItemKind.EntryType,
    documentation,
  };
}

export function createFieldName(field: BibtexField): CompletionItem {
  return {
    label: getFieldName(field),
    kind: LspCompletionItemKind.Field,
    data: CompletionItemKind.FieldName,
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
    data: CompletionItemKind.CommandSymbol,
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
    data: CompletionItemKind.ArgumentSymbol,
    documentation: createImage(name, image),
  };
}

function createImage(name: string, image: string): MarkupContent {
  return {
    kind: MarkupKind.Markdown,
    value: `![${name}](data:image/png;base64,${image}|width=48,height=48)`,
  };
}
