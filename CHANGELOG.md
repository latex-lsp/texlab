# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Use bibliographies found in `BIBINPUTS` environment variable ([#493](https://github.com/latex-lsp/texlab/issues/493))
- Add `texlab.build.pdfDirectory` setting ([#911](https://github.com/latex-lsp/texlab/issues/911))

### Fixed

- Fix search path for aux files when using `\include` instead of `\input` ([#906](https://github.com/latex-lsp/texlab/issues/906))

## [5.8.0] - 2023-07-30

### Added

- Report diagnostics for unused and undefined labels
- Report diagnostics for unused BibTeX entries and undefined citations
- Report diagnostics for duplicate BibTeX entries
- Report diagnostics for duplicate labels
- Add `texlab.build.auxDirectory` and `texlab.build.logDirectory` settings ([#906](https://github.com/latex-lsp/texlab/issues/906))

### Deprecated

- Deprecate `texlab.auxDirectory` in favor of `texlab.build.auxDirectory`

### Fixed

- Fix parsing paths with `|` ([#568](https://github.com/latex-lsp/texlab/issues/568))
- Fix parsing LaTeX identifiers with `=` ([#568](https://github.com/latex-lsp/texlab/issues/568))

## [5.7.0] - 2023-06-07

### Added

- Add symbol support for `thmtools` package ([#894](https://github.com/latex-lsp/texlab/issues/894))
- Add `texlab.findEnvironments` command to return the list of environments containing a position ([#892](https://github.com/latex-lsp/texlab/issues/892))

### Changed

- Show inlay hints for labels after the command instead of inside the argument ([#890](https://github.com/latex-lsp/texlab/issues/890))

### Fixed

- Prevent adding trailing newline after formatting with `latexmk` ([#895](https://github.com/latex-lsp/texlab/issues/895))
- Improving `\paragraph` parsing

## [5.6.0] - 2023-05-20

### Added

- Add `texlab.cancelBuild` command to cancel the currently active build ([#887](https://github.com/latex-lsp/texlab/issues/887))

### Fixed

- Fix resolving include commands from the `import` package ([#885](https://github.com/latex-lsp/texlab/issues/885))
- Fix regression when tracking active cursor position ([#880](https://github.com/latex-lsp/texlab/issues/880))

## [5.5.1] - 2023-05-06

### Fixed

- Fix spurious completion results when completing environments ([#883](https://github.com/latex-lsp/texlab/issues/883))
- Fix regression when guessing cursor position after formatting ([#880](https://github.com/latex-lsp/texlab/issues/880))
- Fix parsing `\verb` command ([#828](https://github.com/latex-lsp/texlab/issues/828))
- Add `flalign` and `flalign*` to default list of math environments ([#884](https://github.com/latex-lsp/texlab/issues/884))

## [5.5.0] - 2023-04-16

### Added

- Allow optionally passing cursor position to `textDocument/build` request for use in forward search after building.
  Previously, the server had to guess the cursor position ([#475](https://github.com/latex-lsp/texlab/issues/475))
- Add experimental `texlab.experimental.citationCommands` setting to allow extending the list of citation commands
  ([#832](https://github.com/latex-lsp/texlab/issues/832))
- Add support for escaping placeholders in build arguments similar to forward search
- Allow configuring completion matching algorithm ([#872](https://github.com/latex-lsp/texlab/issues/872))

### Fixed

- Fix regression introduced in `v5.4.2` involving `texlab.cleanArtifacts` command.

## [5.4.2] - 2023-04-11

### Fixed

- Fix memory leak when editing documents over a long time ([#856](https://github.com/latex-lsp/texlab/issues/856))
- Fix parsing parentheses in file paths ([#874](https://github.com/latex-lsp/texlab/issues/874))

## [5.4.1] - 2023-03-26

### Fixed

- Do not return symbols with empty names (e. g. sections without name) ([#870](https://github.com/latex-lsp/texlab/issues/870))
- Repair `textDocument/formatting` request ([#871](https://github.com/latex-lsp/texlab/issues/871))

## [5.4.0] - 2023-03-12

### Added

- Add experimental settings to allow extending the list of special environments:
  - `texlab.experimental.mathEnvironments`
  - `texlab.experimental.enumEnvironments`
  - `texlab.experimental.verbatimEnvironments`
- Add `texlab.changeEnvironment` workspace command ([#849](https://github.com/latex-lsp/texlab/issues/849))
- Add `texlab.showDependencyGraph` workspace command

### Changed

- Do not show caption or section names in label inlay hints ([#858](https://github.com/latex-lsp/texlab/issues/858))
- Include more user-defined commands in command completion

### Fixed

- Parse nested `\iffalse` blocks correctly ([#853](https://github.com/latex-lsp/texlab/issues/853))
- Parse commands with multi-byte characters correctly ([#857](https://github.com/latex-lsp/texlab/issues/857))
- Fix checking whether a document can be a root file

## [5.3.0] - 2023-02-25

### Added

- Allow filtering `textDocument/documentSymbols` using regular expressions specified via
  `texlab.symbols.allowedPatterns` and `texlab.symbols.ignoredPatterns`
  ([#851](https://github.com/latex-lsp/texlab/issues/851))

### Fixed

- Do not use percent-encoded path when searching for PDF files during forward search
  ([#848](https://github.com/latex-lsp/texlab/issues/848))
- Always return an empty list of code actions instead of returning "method not found" ([#850](https://github.com/latex-lsp/texlab/issues/850))

## [5.2.0] - 2023-01-29

### Added

- Include line numbers in build warnings when available ([#840](https://github.com/latex-lsp/texlab/issues/840))
- Add `none` formatter to `texlab.latexFormatter` and `texlab.bibtexFormatter` options
  to allow disabling formatting ([#846](https://github.com/latex-lsp/texlab/issues/846))

### Fixed

- Concatenate more than two lines of maximum length in build diagnostics ([#842](https://github.com/latex-lsp/texlab/pull/842))
- Apply the correct range of references to labels when renaming ([#841](https://github.com/latex-lsp/texlab/issues/841))
- Use `document` environment to detect root file instead of `\documentclass` ([#845](https://github.com/latex-lsp/texlab/issues/845))

## [5.1.0] - 2023-01-21

### Added

- Allow manually overriding the root directory using a `texlabroot`/`.texlabroot` marker file.
  See the wiki for more information.
  ([#826](https://github.com/latex-lsp/texlab/issues/826), [#838](https://github.com/latex-lsp/texlab/pull/838))

### Deprecated

- Deprecate `texlab.rootDirectory` setting in favor of `.texlabroot` files

### Fixed

- Do not use `.git`, `.chktexrc`, `.latexmkrc` files/directories to determine the root directory
  ([#826](https://github.com/latex-lsp/texlab/issues/826))
- Fix building documents without an explicit root directory ([#837](https://github.com/latex-lsp/texlab/issues/837))

## [5.0.0] - 2022-12-29

### Changed

- _BREAKING_: `texlab.rootDirectory` is now used as the folder path from which the compiler is executed
  relative to the main document. By default it is equal to `"."`. For more information, please visit the wiki.
- Improve performance of completion by a huge margin due to a faster filtering method used internally
- Do not discover project files beyond the provided workspace folders
- Try to guess the root directory by checking for files such as `.latexmkrc` or `Tectonic.toml` if `texlab.rootDirectory` is not set

### Fixed

- Update positions of reported build diagnostics when editing the affected line
- Do not treat links to files as bidirectional by default. This prevents issues where `texlab` ends up compiling the wrong file
  in projects with shared files ([#806](https://github.com/latex-lsp/texlab/issues/806), [#757](https://github.com/latex-lsp/texlab/issues/757), [#679](https://github.com/latex-lsp/texlab/issues/679))
- Fix coverage of directories which need to be watched for changes ([#502](https://github.com/latex-lsp/texlab/issues/502), [#491](https://github.com/latex-lsp/texlab/issues/491))
- Resolve links of the `import` package correctly
- Use `filterText` of completion items when filtering internally ([#829](https://github.com/latex-lsp/texlab/issues/829))

## [4.3.2] - 2022-11-20

### Fixed

- Do not try to run the TeX engine on package files and fail the build instead ([#801](https://github.com/latex-lsp/texlab/issues/801))
- Handle URIs with URL-encoded drive letters on Windows ([#802](https://github.com/latex-lsp/texlab/issues/802))
- Parse BibTeX entries with unbalanced quotes correctly ([#809](https://github.com/latex-lsp/texlab/issues/809))
- Provide completion for more acronym commands ([#813](https://github.com/latex-lsp/texlab/issues/813))
- Fix parsing acronym definitions ([#813](https://github.com/latex-lsp/texlab/issues/813))

## [4.3.1] - 2022-10-22

### Fixed

- Do not crash with a stack overflow when trying to load packages with many internal dependencies ([#793](https://github.com/latex-lsp/texlab/issues/793))
- Normalize drive letters of all document URIs
- Fix parsing commands that take file paths as arguments ([#789](https://github.com/latex-lsp/texlab/issues/789))
- Use the correct working directory and command line arguments when calling `latexindent` ([#645](https://github.com/latex-lsp/texlab/issues/645))
- Fix publishing to CTAN

## [4.3.0] - 2022-09-25

### Added

- Add inlay hints for `\label{...}` ([#753](https://github.com/latex-lsp/texlab/issues/753))

### Fixed

- Improve accuracy of the error locations reported by the TeX engine ([#738](https://github.com/latex-lsp/texlab/issues/738))
- Reduce number of false positive errors reported by `texlab` ([#745](https://github.com/latex-lsp/texlab/issues/745))

## [4.2.2] - 2022-08-28

### Fixed

- Do not watch the same directory multiple times, which can result in a memory leak on Windows ([#737](https://github.com/latex-lsp/texlab/issues/679))
- Fix detection of root document when sharing files between projects ([#679](https://github.com/latex-lsp/texlab/issues/679))
- Fix text synchronization problem caused by file watcher ([#724](https://github.com/latex-lsp/texlab/issues/724))

## [4.2.1] - 2022-08-05

### Fixed

- Deserialize server options with missing keys (or not keys at all) correctly ([#707](https://github.com/latex-lsp/texlab/issues/707))
- Pass `chktexrc` files if they are not in the current directory ([#683](https://github.com/latex-lsp/texlab/issues/683))
- Revert back to server-side file watching due to lack of client support ([#679](https://github.com/latex-lsp/texlab/issues/679))

## [4.2.0] - 2022-07-03

### Added

- Add support for escaping placeholders in forward search ([#649](https://github.com/latex-lsp/texlab/issues/649))
- Add support for diagnostic filtering ([#323](https://github.com/latex-lsp/texlab/issues/323))
- Add pre-built binaries for the following targets:
  - `aarch64-unknown-linux-gnu`
  - `armv7-unknown-linux-gnueabihf`
  - `x86_64-unknown-linux-musl`
  - `aarch64-pc-windows-msvc`
  - `i686-pc-windows-msvc`

### Fixed

- Parse incomplete server options correctly ([#651](https://github.com/latex-lsp/texlab/issues/651))

## [4.1.0] - 2022-06-12

### Added

- Add server commands to clean build directory ([#607](https://github.com/latex-lsp/texlab/issues/607))

### Changed

- Improve output when hovering over BibTeX strings
- Improve the heuristic for finding build artifacts ([#635](https://github.com/latex-lsp/texlab/issues/635))

### Fixed

- Allow brackets in included file paths ([#639](https://github.com/latex-lsp/texlab/issues/639))
- Allow commands in included file paths ([#641](https://github.com/latex-lsp/texlab/issues/641))

## [4.0.0] - 2022-05-25

### Added

- Add `--version` command line flag
- Provide pre-built binaries for `aarch64-apple-darwin` architecture ([#591](https://github.com/latex-lsp/texlab/pull/591))
- Autocomplete files based on `\graphicspath` ([#590](https://github.com/latex-lsp/texlab/issues/590))
- Release `texlab` on `crates.io` ([#399](https://github.com/latex-lsp/texlab/issues/399))

### Changed

- _BREAKING_: Use client-side file watching instead of server-side notifications (`workspace/didChangeWatchedFiles`)
- _BREAKING_: Bump minimum supported Rust version to 1.58.1
- _BREAKING_: Do not use `citeproc-rs` to render citations. Instead, use a custom approach that tries to resemble the `BibLaTeX` output ([#629](https://github.com/latex-lsp/texlab/pull/629))

### Fixed

- Parse `\subinputfrom` command correctly ([#610](https://github.com/latex-lsp/texlab/pull/610))
- Parse verbatim environments correctly ([#490](https://github.com/latex-lsp/texlab/issues/490))
- Stop capturing stdout when build exits ([#588](https://github.com/latex-lsp/texlab/issues/588))
- Fix parsing of key-value pairs ([#573](https://github.com/latex-lsp/texlab/issues/573))
- Normalize `texlab.rootDirectory` when resolving includes ([#571](https://github.com/latex-lsp/texlab/issues/571))
- Allow optional arguments in environment definitions ([#570](https://github.com/latex-lsp/texlab/issues/570))
- Allow `=` in include paths ([#568](https://github.com/latex-lsp/texlab/issues/568))

## [3.3.2] - 2022-02-26

### Fixed

- Parse command definitions with optional arguments correctly
- Fix detection of command definitions in completion ([latex-lsp/texlab-vscode#618](https://github.com/latex-lsp/texlab-vscode/issues/618))
- Watch aux directory by default for changes ([#563](https://github.com/latex-lsp/texlab/issues/563))
- Do not allow multi-line keys in the grammar ([#559](https://github.com/latex-lsp/texlab/issues/559))
- Use `textEdit` property for snippets ([#558](https://github.com/latex-lsp/texlab/issues/558))
- Allow simple commands as text argument for most commands ([#557](https://github.com/latex-lsp/texlab/issues/557))
- Treat `\renewcommand` as an environment definition ([#556](https://github.com/latex-lsp/texlab/issues/556))
- Do not return `null` from forward search request
- Make directory path in `\import` optional ([#540](https://github.com/latex-lsp/texlab/issues/540))
- Do not spam workspace/configuration requests ([#533](https://github.com/latex-lsp/texlab/issues/533))

## [3.3.1] - 2021-11-10

### Fixed

- Fix completion for symbols in commands with incomplete braces ([#510](https://github.com/latex-lsp/texlab/issues/510))
- Do not produce syntax errors for macro parameters inside special command arguments ([#508](https://github.com/latex-lsp/texlab/issues/508))
- Fix a bug that sometimes causes the `aux` file to pick up the diagnostics of the `tex` file ([#502](https://github.com/latex-lsp/texlab/issues/502))
- Fix a bug that sometimes prevents `log` files from being reanalyzed ([#502](https://github.com/latex-lsp/texlab/issues/502))

## [3.3.0] - 2021-10-10

### Added

- Enable incremental text synchronization to reduce serialization overhead ([#460](https://github.com/latex-lsp/texlab/issues/460))

### Changed

- Reduce size of executable by compressing the completion database

### Fixed

- Fix completion of commands near delimiters ([#449](https://github.com/latex-lsp/texlab/issues/449))
- Prevent `texlab` from hanging because of unanswered configuration requests sent to Emacs ([#456](https://github.com/latex-lsp/texlab/issues/456))
- Re-analyze the workspace if the initial configuration has been received late

## [3.2.0] - 2021-06-12

### Added

- Re-introduce `texlab.build.forwardSearchAfter` with a more reliable way of detecting the current line number.
- Re-introduce `texlab.build.onSave` due to popular request ([#427](https://github.com/latex-lsp/texlab/issues/427)).
- Re-introduce work done progress notifications for building.

### Changed

- Recommend `texlab.build.onSave` instead of `-pvc` in documentation.
- Do not rely on `.latexmkrc` for previewing anymore,
  instead `texlab.build.forwardSearchAfter` can be used ([#440](https://github.com/latex-lsp/texlab/issues/440), [#436](https://github.com/latex-lsp/texlab/issues/436)).
- Deprecate `texlab.build.isContinuous` setting

### Fixed

- Fix conditional compilation of the `citation` feature.

## [3.1.0] - 2021-06-03

### Added

- Add `texlab.latexFormatter` setting to allow turning off `latexindent`.
  At the moment, `texlab.latexFormatter: texlab` is not implemented yet and does nothing.
- Expose the `--local` flag of `latexindent` via `texlab.latexindent.local` setting ([#365](https://github.com/latex-lsp/texlab/issues/365))
- Expose the `--modifylinebreaks` flag of `latexindent` via `texlab.latexindent.modifyLineBreaks` setting ([#365](https://github.com/latex-lsp/texlab/issues/365))
- Assign (unique) error codes to static analysis diagnostics.

### Fixed

- Avoid creating defunct `latexindent` processes which clear out the document ([#437](https://github.com/latex-lsp/texlab/issues/437))
- Allow whitespace in LaTeX identifiers like labels ([#433](https://github.com/latex-lsp/texlab/issues/433))
- Run CI on Ubuntu 18.04 to allow an older `glibc` version ([#439](https://github.com/latex-lsp/texlab/issues/439)).

### Fixed

## [3.0.1] - 2021-05-22

### Fixed

- Sometimes the log parser does not pick up errors from the log file ([#426](https://github.com/latex-lsp/texlab/issues/426))
- Fix a bug involving characters that are not part of the ASCII charset ([#428](https://github.com/latex-lsp/texlab/issues/428))

## [3.0.0] - 2021-05-16

### Added

- Basic error analysis for LaTeX files ([#323](https://github.com/latex-lsp/texlab/issues/323))
- Parse LaTeX3 commands correctly ([#410](https://github.com/latex-lsp/texlab/issues/410))
- Allow configuring ChkTeX using a `chktexrc` file ([#309](https://github.com/latex-lsp/texlab/issues/309))
- Implement goto definition for includes ([#386](https://github.com/latex-lsp/texlab/issues/386))
- Provide completion for `\citeA{...}` ([#409](https://github.com/latex-lsp/texlab/issues/409))
- Allow passing additional arguments to `latexindent` ([#365](https://github.com/latex-lsp/texlab/issues/365))
- Document symbols and label completion now correctly handle `subequations`.

### Changed

- _BREAKING_: The configuration format has changed.
  Every setting is now under the `texlab` scope instead of the two scopes `latex` and `bibtex`. For a list of possible options, please see [here](docs/options.md).
  The `latex.build.onSave` setting has been removed in favor of `-pvc` of `latexmk`. In the VSCode extension, the `latex.build.onSave` setting is still available along with the `latex.build.forwardSearchAfter` setting. The reasoning is that that `latex.build.forwardSearchAfter` cannot reliably implemented in the server because it requires the current cursor position, which the LSP spec does not offer. In previous versions, TexLab had to guess the cursor position. We encourage editor extensions, to still support these settings under the `texlab` scope.
- _BREAKING_: Previewing equations has been removed for now until
  a better solution is found. The existing approach is way too slow and does not work reliably.
- Distribution detection no longer produces an error message in the client.
  Instead, a log message is generated. A TeX distribution is only required to compile documents.
- Improve compile times a bit.

### Fixed

- Do not send snippets if the client does not support them ([#413](https://github.com/latex-lsp/texlab/issues/413))
- Fix protocol violation when exiting the server ([#310](https://github.com/latex-lsp/texlab/issues/310))
- Fix reporting compile-time diagnostics using file watching ([#339](https://github.com/latex-lsp/texlab/issues/339))
- Fix compilation warnings ([#359](https://github.com/latex-lsp/texlab/issues/359))
- Fix crash when exiting with NeoVim LSP client ([#405](https://github.com/latex-lsp/texlab/issues/405))
- Hopefully fixes the ChkTeX spamming issue ([#186](https://github.com/latex-lsp/texlab/issues/186))
- Reduce CPU-load when idle ([#400](https://github.com/latex-lsp/texlab/issues/400))

## [2.2.2] - 2021-01-10

### Fixed

- Fix compilation on `arm64-apple-darwin` (Apple Silicon) ([#343](https://github.com/latex-lsp/texlab/issues/343))

## [2.2.1] - 2021-01-06

### Added

- Add basic support for RNW files
- Add support for `varioref` package

### Changed

- Set `isIncomplete` to `false` for small completion lists

### Fixed

- Fix compilation on `aarch64` and `armv7l` ([#289](https://github.com/latex-lsp/texlab/issues/289))

## [2.2.0] - 2020-05-27

### Added

- Fuzzy matching now works with Visual Studio Code

### Changed

- Improve performance of completion

### Fixed

- Fix the ordering of completion items when using `lsp-mode` ([#227](https://github.com/latex-lsp/texlab/issues/227))
- Fix preview when using custom class files ([#228](https://github.com/latex-lsp/texlab/issues/228))

## [2.1.0] - 2020-05-10

### Added

- Add a new setting `latex.build.forwardSearchAfter` to trigger the forward search after building.
- Add option to write the log output to a log file

### Fixed

- Fix crash in symbols when encountering theorem descriptions ([#220](https://github.com/latex-lsp/texlab/issues/220))
- Fix a parsing error that caused `texlab` to take 100% CPU usage in some cases ([#212](https://github.com/latex-lsp/texlab/issues/212))
- Prevent building the same file multiple times at once

## [2.0.0] - 2020-04-20

### Added

- Add basic support for the `import` package
- Allow LaTeX and BibTeX formatting via `latexindent`.
  The built-in BibTeX formatter is still available via `"bibtex.formatting.formatter": "texlab"` ([#151](https://github.com/latex-lsp/texlab/issues/151))

### Fixed

- Handle `subfiles` package when executing forward search ([#208](https://github.com/latex-lsp/texlab/issues/208))
- Fix detection of terminated builds
- Ensure that there is at most one instance of ChkTeX running
- Fix deserialization of incoming JSON-RPC errors
- Fix preview when including packages in a child file

### Changed

- **Breaking change**: `latex.build.args` now uses placeholders like the forward search.
  The filename (`%f`) is no longer implicitly appended to the end of the argument list.
- **Breaking change**: Update the LSP types to accommodate newer LSP clients ([#200](https://github.com/latex-lsp/texlab/issues/200))
- Improve performance of completion (when completing LaTeX commands)
- Improve workspace detection algorithm

## [1.10.0] - 2020-02-11

### Added

- Add a new setting `latex.build.outputDirectory` to specify the directory containing the build artifacts.
  This setting can be used in combination with the `-outdir` flag of `latexmk`
  ([#147](https://github.com/latex-lsp/texlab/issues/147))
- Add basic support for push-based configuration via `workspace/didChangeConfiguration` ([#123](https://github.com/latex-lsp/texlab/issues/123))

### Fixed

- Show all digits of chktex warning number ([#160](https://github.com/latex-lsp/texlab/issues/160))

## [1.9.0] - 2019-12-30

### Added

- Provide completion for local packages if `kpsewhich` is installed
- Add `.def` and `.bibtex` to the list of supported extensions
- Add basic support for `tectonic`

### Fixed

- Fix rendering of citations with DOIs ([#117](https://github.com/latex-lsp/texlab/issues/117))
- Fix building of LaTeX files without `\begin{document}` ([#122](https://github.com/latex-lsp/texlab/issues/122))
- Do not crash when editing remote files
- Run LaTeX linter when opening a file if enabled
- Handle `\hyphen` when rendering citations

## [1.8.0] - 2019-12-01

### Added

- Add support for `crossref` when previewing citations ([#16](https://github.com/latex-lsp/texlab/issues/16))
- Warn if the user does not have a TeX distribution installed

### Changed

- Change license to GPLv3
- Do not require Node.js when building the server (#[87](https://github.com/latex-lsp/texlab/issues/87))

## [1.7.0] - 2019-11-20

### Added

- Add logging for JSON-RPC errors via `stderr` ([#111](https://github.com/latex-lsp/texlab/issues/111))
- Provide completion for `\subfile`
- Provide completion for glossary entries
- Show full path when hovering over includes
- Implement "Goto Definition" for BibTeX strings

### Changed

- Use Rust Stable (1.39+) instead of Rust Beta
- Sort symbols by project order ([#93](https://github.com/latex-lsp/texlab/issues/93))

### Fixed

- Improve detection of local packages inside the current workspace
  ([#110](https://github.com/latex-lsp/texlab/issues/110))
- Fix potential crash in "Goto Definition"

## [1.6.0] - 2019-09-29

### Added

- Include enumeration environments in symbols
- Implement `workspace/symbol` request
- Handle enumeration items when rendering labels
- Handle subtables in symbols and completion

### Changed

- Handle BibTeX strings when rendering citations
- Improve rendering of labels
- Do not require a label when generating symbols
- Improve detection of included files
- Reorganize completion and symbol kinds
- Do not rely on `workspace/didChangeWatchedFiles`
- Use Rust Beta instead of Rust Nightly
- Make rendering of section labels more consistent

### Fixed

- Fix theorem numbers in multi-file projects
- Fix filter text of citations with braces inside a field
- Handle invalid UTF-8 in log files

## [1.5.0] - 2019-08-27

### Added

- Add support for clients that do not support hierarchical symbols
- Add support for hovering over BibTeX strings

### Changed

- Use formatted references in symbol request

### Fixed

- Do not run ChkTeX on BibTeX files
- Fix build freezes on Windows ([#63](https://github.com/latex-lsp/texlab/issues/63))

## [1.4.1] - 2019-08-22

### Added

- Add support for \part

### Fixed

- Increase hover range when hovering over labels
- Fix rendering of theorem labels
- Handle Windows paths correctly

## [1.4.0] - 2019-08-20

### Added

- Provide symbols for BibTeX fields and BibTeX strings
- Provide symbols for LaTeX labels
- Show theorem name when hovering over theorem references
- Show Unicode glyps when completing symbols

### Changed

- Use LocationLink for "peek definition" when possible
- Node.js is no longer a dependency

## [1.3.0] - 2019-08-06

### Added

- Provide document symbols for BibTeX entries and LaTeX sections

### Changed

- Hovering over a package does not require an internet connection anymore
- The Linux server binaries do not depend on `libssl` anymore ([#55](https://github.com/latex-lsp/texlab/issues/55))

### Fixed

- Build cancellation has been reimplemented ([#47](https://github.com/latex-lsp/texlab/issues/47), [#63](https://github.com/latex-lsp/texlab/issues/63))

## [1.2.0] - 2019-07-23

### Added

- Add completion support for `\RequirePackage`
- Filter completion list based on the contents of the reference

### Changed

- The index mechanism has been removed. Packages are now indexed with a script beforehand.

## [1.1.0] - 2019-07-13

### Added

- Add section name and caption to label completion
- Show section name and caption when hovering over labels
- Add some missing kernel commands with stars
- Add support for comma-separated imports
- Add setting to lint after a change occurs

### Changed

- Improve completion at the end of the file

### Fixed

- Fix preselect for environments with missing braces

## [1.0.0] - 2019-07-04

### Added

- Add support citations with multiple keys ([#22](https://github.com/latex-lsp/texlab/issues/22))
- Add support for the cleveref package ([#21](https://github.com/latex-lsp/texlab/issues/21))
- Implement "Go to Definition" for commands ([#15](https://github.com/latex-lsp/texlab/issues/15))
- Provide preview for user-defined commands
- Provide completion and preview for theorem environments

### Changed

- Java is no longer a required dependency
- Node.js is now an optional dependency required to display citations
- Improve performance of completion
- Improve startup time
- The server no longer depends on a workspace folder
- "Find all References" works from a reference instead
  of just the definition ([#25](https://github.com/latex-lsp/texlab/issues/25))
- All configuration items are now optional
- Provide only math labels when completing \eqref
- Preselect the matching environment name ([#29](https://github.com/latex-lsp/texlab/issues/29))

### Fixed

- Let the client decide whether to include the declaration
  when finding all references ([#25](https://github.com/latex-lsp/texlab/issues/25))
- Renaming a label with colons now works as expected ([#30](https://github.com/latex-lsp/texlab/issues/30))
- Handle colons when completing labels and citations ([#30](https://github.com/latex-lsp/texlab/issues/30))
- Do not crash when encountering a BibTeX entry with
  a `crossref` field ([#16](https://github.com/latex-lsp/texlab/issues/16))
- Hovering over uppercase BibTeX fields now shows the documentation
  ([#17](https://github.com/latex-lsp/texlab/issues/17))
- Do not depend on extensions when resolving included files ([#22](https://github.com/latex-lsp/texlab/issues/22))
- Do not depend on the `workspace/configuration` request (#[22](https://github.com/latex-lsp/texlab/issues/22))
- Prevent completion from triggering too often

## [0.4.2] - 2019-04-10

### Fixed

- Fix completion inside `\( \)`. ([#14](https://github.com/latex-lsp/texlab/issues/14))
- Do not crash on invalid requests.

## [0.4.1] - 2019-03-30

### Changed

- Improve startup time

### Fixed

- Improve MiKTeX support ([#8](https://github.com/latex-lsp/texlab-vscode/issues/8))

## [0.4.0] - 2019-03-09

### Added

- Add linting support for LaTeX via [ChkTeX](https://www.nongnu.org/chktex/)

### Changed

- Analyze referenced files that are not part of the current workspace
- Improve completion for includes
- Improve performance of completion

## [0.3.0] - 2019-03-05

### Added

- Show preview when hovering over math expressions
- Show package name when hovering over a command

### Changed

- Store completion database in `~/.texlab` directory

### Fixed

- Fix crash when editing a BibTeX file
- Fix crash when hovering over invalid BibTeX entries
- Fix a bug where the completion does not get triggered correctly

## [0.2.0] - 2019-03-01

### Added

- Show bibliography when completing citations
- Show bibliography when hovering over citations
- Completion for equation references

### Fixed

- Fix completion of file includes
- Prevent server crash when opening a locked file

## [0.1.2] - 2019-02-16

### Fixed

- Do not display an error when PDF viewers return a non-zero
  exit code while performing forward search

## [0.1.1] - 2019-02-15

### Changed

- Reduce binary size

### Fixed

- Fix rendering of completion symbols

## [0.1.0] - 2019-02-15

- Initial release
