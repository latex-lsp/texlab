use anyhow::Result;
use insta::{assert_json_snapshot, internals::Redaction};
use lsp_types::{
    notification::{DidChangeConfiguration, Notification, PublishDiagnostics},
    ClientCapabilities, Diagnostic, DidChangeConfigurationParams, PublishDiagnosticsParams, Url,
};
use rustc_hash::FxHashMap;

use crate::lsp::{client::Client, fixture};

struct DiagnosticResult {
    all_diagnostics: FxHashMap<Url, Vec<Diagnostic>>,
    uri_redaction: Redaction,
}

fn find_diagnostics(fixture: &str, settings: serde_json::Value) -> Result<DiagnosticResult> {
    let mut client = Client::spawn()?;
    client.initialize(ClientCapabilities::default(), None)?;

    client.notify::<DidChangeConfiguration>(DidChangeConfigurationParams { settings })?;

    let fixture = fixture::parse(fixture);
    for file in fixture.files {
        client.store_on_disk(file.name, &file.text)?;
        if file.lang != "log" {
            client.open(file.name, file.lang, file.text)?;
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(1));

    let result = client.shutdown()?;

    let uri = Url::from_directory_path(result.directory.path()).unwrap();
    let uri_redaction = insta::dynamic_redaction(move |content, _path| {
        content.as_str().unwrap().replace(uri.as_str(), "[tmp]/")
    });

    let all_diagnostics = result
        .incoming
        .notifications
        .into_iter()
        .filter_map(|notification| {
            notification
                .extract::<PublishDiagnosticsParams>(PublishDiagnostics::METHOD)
                .ok()
        })
        .map(|params| (params.uri, params.diagnostics))
        .collect();

    Ok(DiagnosticResult {
        all_diagnostics,
        uri_redaction,
    })
}

macro_rules! assert_symbols {
    ($result:expr) => {
        let result = $result;
        assert_json_snapshot!(result.all_diagnostics, {
            ".$key" => result.uri_redaction
        });
    };
}

static BUILD_LOG_FIXTURE: &str = r#"
%TEX main.tex
%SRC \documentclass{article}
%SRC 
%SRC \usepackage{amsmath}
%SRC 
%SRC \begin{document}
%SRC 
%SRC \foo{}
%SRC aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
%SRC 
%SRC \end{document}

%LOG main.log
%SRC This is pdfTeX, Version 3.141592653-2.6-1.40.22 (TeX Live 2021/W32TeX) (preloaded format=pdflatex 2022.1.23)  16 JUN 2022 11:04
%SRC entering extended mode
%SRC  restricted \write18 enabled.
%SRC  %&-line parsing enabled.
%SRC **main.tex
%SRC (./main.tex
%SRC LaTeX2e <2020-10-01> patch level 4
%SRC L3 programming layer <2021-02-18>
%SRC (c:/texlive/2021/texmf-dist/tex/latex/base/article.cls
%SRC Document Class: article 2020/04/10 v1.4m Standard LaTeX document class
%SRC (c:/texlive/2021/texmf-dist/tex/latex/base/size10.clo
%SRC File: size10.clo 2020/04/10 v1.4m Standard LaTeX file (size option)
%SRC )
%SRC \c@part=\count179
%SRC \c@section=\count180
%SRC \c@subsection=\count181
%SRC \c@subsubsection=\count182
%SRC \c@paragraph=\count183
%SRC \c@subparagraph=\count184
%SRC \c@figure=\count185
%SRC \c@table=\count186
%SRC \abovecaptionskip=\skip47
%SRC \belowcaptionskip=\skip48
%SRC \bibindent=\dimen138
%SRC )
%SRC (c:/texlive/2021/texmf-dist/tex/latex/amsmath/amsmath.sty
%SRC Package: amsmath 2020/09/23 v2.17i AMS math features
%SRC \@mathmargin=\skip49
%SRC 
%SRC For additional information on amsmath, use the `?' option.
%SRC (c:/texlive/2021/texmf-dist/tex/latex/amsmath/amstext.sty
%SRC Package: amstext 2000/06/29 v2.01 AMS text
%SRC 
%SRC (c:/texlive/2021/texmf-dist/tex/latex/amsmath/amsgen.sty
%SRC File: amsgen.sty 1999/11/30 v2.0 generic functions
%SRC \@emptytoks=\toks15
%SRC \ex@=\dimen139
%SRC ))
%SRC (c:/texlive/2021/texmf-dist/tex/latex/amsmath/amsbsy.sty
%SRC Package: amsbsy 1999/11/29 v1.2d Bold Symbols
%SRC \pmbraise@=\dimen140
%SRC )
%SRC (c:/texlive/2021/texmf-dist/tex/latex/amsmath/amsopn.sty
%SRC Package: amsopn 2016/03/08 v2.02 operator names
%SRC )
%SRC \inf@bad=\count187
%SRC LaTeX Info: Redefining \frac on input line 234.
%SRC \uproot@=\count188
%SRC \leftroot@=\count189
%SRC LaTeX Info: Redefining \overline on input line 399.
%SRC \classnum@=\count190
%SRC \DOTSCASE@=\count191
%SRC LaTeX Info: Redefining \ldots on input line 496.
%SRC LaTeX Info: Redefining \dots on input line 499.
%SRC LaTeX Info: Redefining \cdots on input line 620.
%SRC \Mathstrutbox@=\box47
%SRC \strutbox@=\box48
%SRC \big@size=\dimen141
%SRC LaTeX Font Info:    Redeclaring font encoding OML on input line 743.
%SRC LaTeX Font Info:    Redeclaring font encoding OMS on input line 744.
%SRC \macc@depth=\count192
%SRC \c@MaxMatrixCols=\count193
%SRC \dotsspace@=\muskip16
%SRC \c@parentequation=\count194
%SRC \dspbrk@lvl=\count195
%SRC \tag@help=\toks16
%SRC \row@=\count196
%SRC \column@=\count197
%SRC \maxfields@=\count198
%SRC \andhelp@=\toks17
%SRC \eqnshift@=\dimen142
%SRC \alignsep@=\dimen143
%SRC \tagshift@=\dimen144
%SRC \tagwidth@=\dimen145
%SRC \totwidth@=\dimen146
%SRC \lineht@=\dimen147
%SRC \@envbody=\toks18
%SRC \multlinegap=\skip50
%SRC \multlinetaggap=\skip51
%SRC \mathdisplay@stack=\toks19
%SRC LaTeX Info: Redefining \[ on input line 2923.
%SRC LaTeX Info: Redefining \] on input line 2924.
%SRC )
%SRC (c:/texlive/2021/texmf-dist/tex/latex/l3backend/l3backend-pdftex.def
%SRC File: l3backend-pdftex.def 2021-03-18 L3 backend support: PDF output (pdfTeX)
%SRC \l__color_backend_stack_int=\count199
%SRC \l__pdf_internal_box=\box49
%SRC )
%SRC (./main.aux)
%SRC \openout1 = `main.aux'.
%SRC 
%SRC LaTeX Font Info:    Checking defaults for OML/cmm/m/it on input line 5.
%SRC LaTeX Font Info:    ... okay on input line 5.
%SRC LaTeX Font Info:    Checking defaults for OMS/cmsy/m/n on input line 5.
%SRC LaTeX Font Info:    ... okay on input line 5.
%SRC LaTeX Font Info:    Checking defaults for OT1/cmr/m/n on input line 5.
%SRC LaTeX Font Info:    ... okay on input line 5.
%SRC LaTeX Font Info:    Checking defaults for T1/cmr/m/n on input line 5.
%SRC LaTeX Font Info:    ... okay on input line 5.
%SRC LaTeX Font Info:    Checking defaults for TS1/cmr/m/n on input line 5.
%SRC LaTeX Font Info:    ... okay on input line 5.
%SRC LaTeX Font Info:    Checking defaults for OMX/cmex/m/n on input line 5.
%SRC LaTeX Font Info:    ... okay on input line 5.
%SRC LaTeX Font Info:    Checking defaults for U/cmr/m/n on input line 5.
%SRC LaTeX Font Info:    ... okay on input line 5.
%SRC 
%SRC ! Undefined control sequence.
%SRC l.7 \foo
%SRC         {}
%SRC The control sequence at the end of the top line
%SRC of your error message was never \def'ed. If you have
%SRC misspelled it (e.g., `\hobx'), type `I' and the correct
%SRC spelling (e.g., `I\hbox'). Otherwise just continue,
%SRC and I'll forget about whatever was undefined.
%SRC 
%SRC 
%SRC Overfull \hbox (80.00125pt too wide) in paragraph at lines 8--9
%SRC []\OT1/cmr/m/n/10 aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
%SRC aaaaaaaaaaaaaaaaaaaaa 
%SRC  []
%SRC 
%SRC [1
%SRC 
%SRC {c:/texlive/2021/texmf-var/fonts/map/pdftex/updmap/pdftex.map}] (./main.aux) ) 
%SRC Here is how much of TeX's memory you used:
%SRC  1014 strings out of 478994
%SRC  13996 string characters out of 5862207
%SRC  300924 words of memory out of 5000000
%SRC  18565 multiletter control sequences out of 15000+600000
%SRC  403430 words of font info for 27 fonts, out of 8000000 for 9000
%SRC  1141 hyphenation exceptions out of 8191
%SRC  67i,4n,74p,200b,104s stack positions out of 5000i,500n,10000p,200000b,80000s
%SRC <
%SRC c:/texlive/2021/texmf-dist/fonts/type1/public/amsfonts/cm/cmr10.pfb>
%SRC Output written on main.pdf (1 page, 9741 bytes).
%SRC PDF statistics:
%SRC  12 PDF objects out of 1000 (max. 8388607)
%SRC  7 compressed objects within 1 object stream
%SRC  0 named destinations out of 1000 (max. 500000)
%SRC  1 words of extra memory for PDF output out of 10000 (max. 10000000)"#;

#[test]
fn build_log_filter_none() -> Result<()> {
    assert_symbols!(find_diagnostics(
        BUILD_LOG_FIXTURE,
        serde_json::json!({
            "diagnosticsDelay": 0,
        })
    )?);
    Ok(())
}

#[test]
fn build_log_filter_allowed() -> Result<()> {
    assert_symbols!(find_diagnostics(
        BUILD_LOG_FIXTURE,
        serde_json::json!({
            "diagnosticsDelay": 0,
            "diagnostics": {
                "allowedPatterns": ["Overfull \\\\[hv]box"]
            }
        })
    )?);

    Ok(())
}

#[test]
fn build_log_filter_ignored() -> Result<()> {
    assert_symbols!(find_diagnostics(
        BUILD_LOG_FIXTURE,
        serde_json::json!({
            "diagnosticsDelay": 0,
            "diagnostics": {
                "ignoredPatterns": ["Overfull \\\\[hv]box"]
            }
        })
    )?);

    Ok(())
}

#[test]
fn build_log_spaces_in_path() -> Result<()> {
    assert_symbols!(find_diagnostics(
        r#"
%TEX foo bar/main.tex
%SRC \documentclass{article}
%SRC \usepackage{amsmath}
%SRC \begin{document}
%SRC \foo{}
%SRC \end{document}

%LOG foo bar/main.log
%SRC This is pdfTeX, Version 3.141592653-2.6-1.40.22 (TeX Live 2021/W32TeX) (preloaded format=pdflatex 2022.1.23)  16 JUN 2022 11:04
%SRC entering extended mode
%SRC  restricted \write18 enabled.
%SRC  %&-line parsing enabled.
%SRC **main.tex
%SRC (./main.tex
%SRC LaTeX2e <2020-10-01> patch level 4
%SRC L3 programming layer <2021-02-18>
%SRC (c:/texlive/2021/texmf-dist/tex/latex/base/article.cls
%SRC Document Class: article 2020/04/10 v1.4m Standard LaTeX document class
%SRC (c:/texlive/2021/texmf-dist/tex/latex/base/size10.clo
%SRC File: size10.clo 2020/04/10 v1.4m Standard LaTeX file (size option)
%SRC ))
%SRC (c:/texlive/2021/texmf-dist/tex/latex/amsmath/amsmath.sty
%SRC Package: amsmath 2020/09/23 v2.17i AMS math features
%SRC \@mathmargin=\skip49
%SRC 
%SRC For additional information on amsmath, use the `?' option.
%SRC (c:/texlive/2021/texmf-dist/tex/latex/amsmath/amstext.sty
%SRC Package: amstext 2000/06/29 v2.01 AMS text
%SRC 
%SRC (c:/texlive/2021/texmf-dist/tex/latex/amsmath/amsgen.sty
%SRC File: amsgen.sty 1999/11/30 v2.0 generic functions
%SRC \@emptytoks=\toks15
%SRC \ex@=\dimen139
%SRC ))
%SRC )
%SRC (./main.aux)
%SRC \openout1 = `main.aux'.
%SRC 
%SRC ! Undefined control sequence.
%SRC l.4 \foo
%SRC         {}
%SRC The control sequence at the end of the top line
%SRC of your error message was never \def'ed. If you have
%SRC misspelled it (e.g., `\hobx'), type `I' and the correct
%SRC spelling (e.g., `I\hbox'). Otherwise just continue,
%SRC and I'll forget about whatever was undefined.
%SRC {c:/texlive/2021/texmf-var/fonts/map/pdftex/updmap/pdftex.map}] (./main.aux) ) 
%SRC <
%SRC c:/texlive/2021/texmf-dist/fonts/type1/public/amsfonts/cm/cmr10.pfb>
%SRC Output written on main.pdf (1 page, 9741 bytes).
"#,
        serde_json::json!({
            "diagnosticsDelay": 0,
        })
    )?);
    Ok(())
}
