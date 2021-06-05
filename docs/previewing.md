# Previewing

`texlab` supports compiling LaTeX using a custom request (`textDocument/build`)
and by building a document after saving if configured to do so.
To enable building on save, simply set `texlab.build.onSave` to true.
Previewing can be configured in a variety of ways:

1. If you are using `latexmk`, you can create a `.latexmkrc` file and call your viewer accordingly.
   Afterwards, you can add the `-pv` flag to your `texlab.build.args`.

2. If you want the PDF viewer to stay synchronized with the cursor position in your editor,
   you can instruct `texlab` to execute a forward search after every build (`texlab.build.forwardSearchAfter`).
   To do so, you need to enable [SyncTeX](http://www.tug.org/TUGboat/tb29-3/tb93laurens.pdf)
   and update the `texlab.forwardSearch` configuration.
   If you want to use this feature, we do _not_ recommend the `-pvc` flag
   because `texlab` does not get notified by `latexmk` when a document gets built.
   Instead, you can use `texlab.build.onSave`.

In the following sections, we will give forward search configurations for several popular viewers
and Visual Studio Code.
However, these settings can easily be adapted to other editors.
If your viewer is not listed here, you can send us a pull request or create an issue.

---

## SumatraPDF

We highly recommend [SumatraPDF](https://www.sumatrapdfreader.org) on Windows
because Adobe Reader locks the opened PDF file and will therefore prevent further builds.

### Forward Search

Add the following lines to your editor config:

```json
{
  "texlab.forwardSearch.executable": "C:/Users/{User}/AppData/Local/SumatraPDF/SumatraPDF.exe",
  "texlab.forwardSearch.args": [
    "-reuse-instance",
    "%p",
    "-forward-search",
    "%f",
    "%l"
  ]
}
```

### Inverse Search

Add the following line to your SumatraPDF settings file (Menu -> Settings -> Advanced Options):

```ini
InverseSearchCmdLine = "C:\Users\{User}\AppData\Local\Programs\Microsoft VS Code\Code.exe" -g "%f":%l
```

> **Note**: Please make sure to replace `{User}` with your Windows username.

You can execute the search by pressing `Alt+DoubleClick` in the PDF document.

---

## Evince

The SyncTeX feature of [Evince](https://wiki.gnome.org/Apps/Evince) requires communication via D-Bus.
In order to use it from the command line, install the [evince-synctex](https://github.com/latex-lsp/evince-synctex) script.

### Forward Search

Add the following lines to your editor config:

```json
{
  "texlab.forwardSearch.executable": "evince-synctex",
  "texlab.forwardSearch.args": ["-f", "%l", "%p", "\"code -g %f:%l\""]
}
```

### Inverse Search

The inverse search feature is already configured if you use `evince-synctex`.
You can execute the search by pressing `Ctrl+Click` in the PDF document.

---

## Okular

### Forward Search

Add the following lines to your editor config:

```json
{
  "texlab.forwardSearch.executable": "okular",
  "texlab.forwardSearch.args": ["--unique", "file:%p#src:%l%f"]
}
```

### Inverse Search

Change the editor of Okular (Settings -> Configure Okular... -> Editor) to "Custom Text Editor" and set the following command:

```bash
code -g "%f":%l
```

You can execute the search by pressing `Shift+Click` in the PDF document.

---

## Zathura

### Forward Search

Add the following lines to your editor config:

```json
{
  "texlab.forwardSearch.executable": "zathura",
  "texlab.forwardSearch.args": ["--synctex-forward", "%l:1:%f", "%p"]
}
```

### Inverse Search

Add the following lines to your `~/.config/zathura/zathurarc` file:

```bash
set synctex true
set synctex-editor-command "code -g %{input}:%{line}"
```

You can execute the search by pressing `Alt+Click` in the PDF document.

---

## qpdfview

### Forward Search

Add the following lines to your editor config:

```json
{
  "texlab.forwardSearch.executable": "qpdfview",
  "texlab.forwardSearch.args": ["--unique", "%p#src:%f:%l:1"]
}
```

### Inverse Search

Change the source editor setting (Edit -> Settings... -> Behavior -> Source editor) to:

```bash
code -g "%1":%2
```

and select a mouse button modifier (Edit -> Settings... -> Behavior -> Modifiers -> Mouse button modifiers -> Open in Source Editor)
of choice.
You can execute the search by pressing `Modifier+Click` in the PDF document.

---

## Skim

We recommend [Skim](https://skim-app.sourceforge.io/) on macOS since it is the only native viewer that supports SyncTeX.

Additionally, enable the "Reload automatically" setting in the Skim preferences (Skim -> Preferences -> Sync -> Check for file changes).

### Forward Search

Add the following lines to your editor config:

```json
{
  "texlab.forwardSearch.executable": "/Applications/Skim.app/Contents/SharedSupport/displayline",
  "texlab.forwardSearch.args": ["%l", "%p", "%f"]
}
```

If you want Skim to stay in the background after
executing the forward search, you can add the `-g` option
to `texlab.forwardSearch.args`.

### Inverse Search

Select the Visual Studio Code preset in the Skim preferences (Skim -> Preferences -> Sync -> PDF-TeX Sync support).
You can execute the search by pressing `Shift+⌘+Click` in the PDF document.

---
