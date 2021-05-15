The following table describes the mapping of LaTeX and BibTeX structures
to their `CompletionItemKind` and `SymbolKind`.

| LaTeX / BibTeX structure  | CompletionItemKind   | SymbolKind           |
| ------------------------- | -------------------- | -------------------- |
| Command                   | `Function` (3)       | `Function` (12)      |
| Command Argument          | `Value` (12)         | `Number` (16)        |
| Snippet                   | `Snippet` (15)       |                      |
| Environment               | `Enum` (13)          | `Enum` (10)          |
| Section                   | `Module` (9)         | `Module` (2)         |
| Float                     | `Method` (2)         | `Method` (6)         |
| Theorem                   | `Variable` (6)       | `Variable` (13)      |
| Equation                  | `Constant` (21)      | `Constant` (14)      |
| Enumeration Item          | `EnumMember` (20)    | `EnumMember` (22)    |
| Label                     | `Constructor` (4)    | `Constructor` (9)    |
| Folder                    | `Folder` (19)        | `Namespace` (3)      |
| File                      | `File` (17)          | `File` (1)           |
| PGF Library               | `Property` (10)      | `Property` (7)       |
| TikZ Library              | `Property` (10)      | `Property` (7)       |
| Color                     | `Color` (16)         |                      |
| Color Model               | `Color` (16)         |                      |
| Package                   | `Class` (7)          | `Class` (5)          |
| Class                     | `Class` (7)          | `Class` (5)          |
| BibTeX Entry (Misc)       | `Interface` (8)      | `Interface` (11)     |
| BibTeX Entry (Article)    | `Event` (23)         | `Event` (24)         |
| BibTeX Entry (Book)       | `Struct` (22)        | `Struct` (23)        |
| BibTeX Entry (Collection) | `TypeParameter` (25) | `TypeParameter` (26) |
| BibTeX Entry (Part)       | `Operator` (24)      | `Operator` (25)      |
| BibTeX Entry (Thesis)     | `Unit` (11)          | `Object` (19)        |
| BibTeX String             | `Text` (1)           | `String` (15)        |
| BibTeX Field              | `Field` (5)          | `Field` (8)          |
