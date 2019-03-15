import {
  CompletionItem,
  CompletionItemKind,
  InsertTextFormat,
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

export function createSnippet(
  name: string,
  component: string | undefined,
  template: string,
): CompletionItem {
  return {
    label: name,
    kind: CompletionItemKind.Snippet,
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
    kind: CompletionItemKind.Function,
    detail: getDetail(component),
  };
}

export function createEnvironment(
  name: string,
  component: string | undefined,
): CompletionItem {
  return {
    label: name,
    kind: CompletionItemKind.EnumMember,
    detail: getDetail(component),
  };
}

export function createLabel(name: string): CompletionItem {
  return {
    label: name,
    kind: CompletionItemKind.Field,
  };
}

export function createFolder(name: string): CompletionItem {
  return {
    label: name,
    kind: CompletionItemKind.Folder,
    commitCharacters: ['/'],
  };
}

export function createFile(name: string): CompletionItem {
  return {
    label: name,
    kind: CompletionItemKind.File,
  };
}

export function createPgfLibrary(name: string): CompletionItem {
  return {
    label: name,
    kind: CompletionItemKind.Module,
    commitCharacters: [' '],
  };
}

export function createTikzLibrary(name: string): CompletionItem {
  return {
    label: name,
    kind: CompletionItemKind.Module,
    commitCharacters: [' '],
  };
}

export function createColor(name: string): CompletionItem {
  return {
    label: name,
    kind: CompletionItemKind.Color,
  };
}

export function createColorModel(name: string): CompletionItem {
  return {
    label: name,
    kind: CompletionItemKind.Color,
  };
}

export function createPackage(name: string): CompletionItem {
  return {
    label: name,
    kind: CompletionItemKind.Class,
  };
}

export function createClass(name: string): CompletionItem {
  return {
    label: name,
    kind: CompletionItemKind.Class,
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
    kind: CompletionItemKind.Interface,
    documentation,
  };
}

export function createFieldName(field: BibtexField): CompletionItem {
  return {
    label: getFieldName(field),
    kind: CompletionItemKind.Field,
    documentation: {
      kind: MarkupKind.Markdown,
      value: getFieldDocumentation(field),
    },
  };
}
