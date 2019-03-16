import { WorkspaceBuilder } from './workspaceBuilder';

describe('Workspace', () => {
  it('should append extensions when analyzing includes', () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.tex', '\\include{bar/baz}');
    builder.document('bar/baz.tex', '');
    const workspace = builder.workspace;
    expect(workspace.relatedDocuments(uri)).toHaveLength(2);
  });

  it('should ignore invalid includes', () => {
    const builder = new WorkspaceBuilder();
    const text = '\\include{<foo>?|bar|:}\n\\include{}';
    const uri = builder.document('foo.tex', text);
    const workspace = builder.workspace;
    expect(workspace.relatedDocuments(uri)).toHaveLength(1);
  });

  it('should find related bibliographies', () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.tex', '\\addbibresource{bar.bib}');
    builder.document('bar.bib', '');
    const workspace = builder.workspace;
    expect(workspace.relatedDocuments(uri)).toHaveLength(2);
  });

  it('should ignore includes that cannot be resolved', () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.tex', '\\include{bar}');
    const workspace = builder.workspace;
    expect(workspace.relatedDocuments(uri)).toHaveLength(1);
  });

  it('should handle include cycles', () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.tex', '\\input{bar.tex}');
    builder.document('bar.tex', '\\input{foo.tex}');
    const workspace = builder.workspace;
    expect(workspace.relatedDocuments(uri)).toHaveLength(2);
  });

  it('should return a standalone document when finding the parent', () => {
    const builder = new WorkspaceBuilder();
    const uri1 = builder.document('foo.tex', '');
    const uri2 = builder.document(
      'bar.tex',
      '\\begin{document}\\include{foo}\\end{document}',
    );
    const workspace = builder.workspace;
    expect(workspace.findParent(uri1)!.uri).toEqual(uri2);
  });

  it('should return the document when no parent can be found', () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.tex', '');
    const workspace = builder.workspace;
    expect(workspace.findParent(uri)!.uri).toEqual(uri);
  });

  it('should ignore unrelated documents', () => {
    const builder = new WorkspaceBuilder();
    const uri = builder.document('foo.tex', '\\include{bar}');
    builder.document('bar.tex', '');
    builder.document('baz.tex', '');
    const workspace = builder.workspace;
    expect(workspace.relatedDocuments(uri)).toHaveLength(2);
  });
});
