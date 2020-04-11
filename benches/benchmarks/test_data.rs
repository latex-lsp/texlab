use indoc::indoc;
use once_cell::sync::Lazy;

pub static TEST_LATEX: Lazy<String> = Lazy::new(|| {
    indoc!(
        r#"
            \documentclass{article}
            \usepackage{amsmath}
            \usepackage{lipsum}
            \usepackage{geometry}
            \usepackage[utf8]{inputenc}
            \newcommand{\foo}{foo}
            \DeclareMathOperator{\bar}{bar}
            \include{child1}
            \input{child2.tex}
            \begin{document}
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
            \begin{equation*}\label{eq:foo}
                e^{i \pi} + 1 = 0
            \end{equation*}
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
            \ref{eq:foo}
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
            \section{Foo}\label{sec:foo}
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
            \subsection{Bar}\label{sec:bar}
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
            \include{foo}
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
            \input{bar.tex}
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
            \cite{foo, bar, baz}
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
            \nocite{*}
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum lectus placerat, suscipit ligula quis.
            \end{document}
        "#
    )
    .repeat(15)
});

pub static TEST_BIBTEX: Lazy<String> = Lazy::new(|| {
    r#"
            @string{anch-ie = {Angew.~Chem. Int.~Ed.}}
            @string{cup     = {Cambridge University Press}}
            @string{dtv     = {Deutscher Taschenbuch-Verlag}}
            @string{hup     = {Harvard University Press}}
            @string{jams    = {J.~Amer. Math. Soc.}}
            @string{jchph   = {J.~Chem. Phys.}}
            @string{jomch   = {J.~Organomet. Chem.}}
            @string{pup     = {Princeton University Press}}
            
            @incollection{westfahl:space,
            author       = {Westfahl, Gary},
            title        = {The True Frontier},
            subtitle     = {Confronting and Avoiding the Realities of Space in {American}
                            Science Fiction Films},
            pages        = {55-65},
            crossref     = {westfahl:frontier},
            langid       = {english},
            langidopts   = {variant=american},
            indextitle   = {True Frontier, The},
            annotation   = {A cross-referenced article from a \texttt{collection}. This is
                            an \texttt{incollection} entry with a \texttt{crossref}
                            field. Note the \texttt{subtitle} and \texttt{indextitle}
                            fields},
            }
        "#
    .repeat(15)
});
