### Example: Using `tectonic`

[`tectonic`](https://tectonic-typesetting.github.io/) is an alternative to `latexmk`, built in Rust.
You can quickly get started by changing `.vscode/settings.json` in your workspace to include the following:

```jsonc
{
  // See `tectonic --help` for the format
  "texlab.build.executable": "tectonic",
  "texlab.build.args": [
    // Input
    "%f",
    // Flags
    "--synctex",
    "--keep-logs",
    "--keep-intermediates"
    // Options
    // OPTIONAL: If you want a custom out directory,
    // uncomment the following line.
    //"--outdir out",
  ]
  // OPTIONAL: The server needs to be configured
  // to read the logs from the out directory as well.
  // "texlab.auxDirectory": "out",
}
```
