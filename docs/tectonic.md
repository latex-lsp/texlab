# Usage with `tectonic`

[`tectonic`](https://tectonic-typesetting.github.io/) is a modernized, alternative TeX engine.
Most features of `texlab` work out of the box when using `tectonic`.
To compile documents through `texlab`, you need to change the configuration.
See `tectonic --help` for more information about the flags.

---

**Hint:**

Please make sure to set `texlab.auxDirectory` if you change the build directory with the `--outdir` argument.

Also, `--keep-intermediates` is recommended because they allow `texlab`
to find out the section numbers and show them in the completion.
Without the `--keep-logs` flag, `texlab` won't be able to report compilation warnings.

---

## V2 CLI

```json
{
  "texlab.build.executable": "tectonic",
  "texlab.build.args": [
    "-X",
    "compile",
    "%f",
    "--synctex",
    "--keep-logs",
    "--keep-intermediates"
  ]
}
```

## V1 CLI

```json
{
  "texlab.build.executable": "tectonic",
  "texlab.build.args": [
    "%f",
    "--synctex",
    "--keep-logs",
    "--keep-intermediates"
  ]
}
```
