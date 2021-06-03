# Configuration

This document describes the configuration settings
that the server will query from the LSP client / extension.

---

## texlab.rootDirectory

Defines the directory from which the source files get compiled.
You may need to set this property for multi-folder projects
where TexLab fails to detect the root document.

**Type:** `string | null`

**Default value**: `null`

---

## texlab.build.executable

Defines the executable of the LaTeX build tool.

**Type:** `string`

**Default value:** `latexmk`

---

## texlab.build.args

Defines additional arguments that are passed to the configured LaTeX build tool.
Note that flags and their arguments need to be separate
elements in this array.
To pass the arguments `-foo bar` to a build tool,
`latex.build.args` needs to be `["-foo", "bar"]`.
The placeholder `%f` will be replaced by the server.

**Placeholders:**

- `%f`: The path of the TeX file to compile.

**Type:** `string[]`

**Default value:** `["-pdf", "-interaction=nonstopmode", "-synctex=1", "%f"]`

---

## texlab.build.isContinuous

Set this property to true if the build arguments imply a continous build (like `latexmk -pvc`).

**Type:** `boolean`

**Default value:** `false`

---

## texlab.auxDirectory

Defines the directory containing the build artifacts.
Note that you need to set the output directory in `latex.build.args` too,
if you want to change the build directory.
In this case, use the `-outdir` flag for `latexmk`.

**Type:** `string`

**Default value:** `.` (the same directory as the TeX file)

## texlab.forwardSearch.executable

Defines the executable of the PDF previewer.
The previewer needs to support [SyncTeX](http://www.tug.org/TUGboat/tb29-3/tb93laurens.pdf).

**Type:** `string | null`

**Default value:** `null`

---

## texlab.forwardSearch.args

Defines additional arguments that are passed to the configured previewer to perform the forward search.
The placeholders `%f, %p, %l` will be replaced by the server.

**Placeholders:**

- `%f`: The path of the current TeX file.
- `%p`: The path of the current PDF file.
- `%l`: The current line number.

**Type:** `string[] | null`

**Default value:** `null`

---

## texlab.chktex.onOpenAndSave

Lint using [chktex](https://www.nongnu.org/chktex/) after opening and saving a file.

**Type:** `boolean`

**Default value:** `false`

---

## texlab.chktex.onEdit

Lint using [chktex](https://www.nongnu.org/chktex/) after editing a file.

**Type:** `boolean`

**Default value:** `false`

---

## texlab.diagnosticsDelay

Delay in milliseconds before reporting diagnostics.

**Type:** `integer`

**Default value:** `300`

---

## texlab.formatterLineLength

Defines the maximum amount of characters per line (0 = disable) when formatting BibTeX files.

**Type:** `integer`

**Default value:** `80`

---

## texlab.bibtexFormatter

Defines the formatter to use for BibTeX formatting.
Possible values are either `texlab` or `latexindent`.

**Type:** `string`

**Default value:** `texlab`

---

## texlab.latexFormatter

Defines the formatter to use for LaTeX formatting.
Possible values are either `texlab` or `latexindent`.
Note that `texlab` is not implemented yet.

**Type:** `string`

**Default value:** `latexindent`

---

## texlab.latexindent.local

Defines the path of a file containing the `latexindent` configuration.
This corresponds to the `--local=file.yaml` flag of `latexindent`.
By default the configuration inside the project root directory is used.

**Type:** `string`

**Default value:** `null`

---

## texlab.latexindent.modifyLineBreaks

Modifies linebreaks before, during, and at the end of code blocks
when formatting with `latexindent`.
This corresponds to the `--modifylinebreaks` flag of `latexindent`.

**Type:** `boolean`

**Default value:** `false`
