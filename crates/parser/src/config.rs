use rustc_hash::FxHashSet;

#[derive(Debug)]
pub struct SyntaxConfig {
    pub math_environments: FxHashSet<String>,
    pub enum_environments: FxHashSet<String>,
    pub verbatim_environments: FxHashSet<String>,
    pub citation_commands: FxHashSet<String>,
}

impl Default for SyntaxConfig {
    fn default() -> Self {
        let math_environments = DEFAULT_MATH_ENVIRONMENTS
            .iter()
            .map(ToString::to_string)
            .collect();

        let enum_environments = DEFAULT_ENUM_ENVIRONMENTS
            .iter()
            .map(ToString::to_string)
            .collect();

        let verbatim_environments = DEFAULT_VERBATIM_ENVIRONMENTS
            .iter()
            .map(ToString::to_string)
            .collect();

        let citation_commands = DEFAULT_CITATION_COMMANDS
            .iter()
            .map(ToString::to_string)
            .collect();

        Self {
            math_environments,
            enum_environments,
            verbatim_environments,
            citation_commands,
        }
    }
}

static DEFAULT_MATH_ENVIRONMENTS: &[&str] = &[
    "align",
    "align*",
    "alignat",
    "alignat*",
    "aligned",
    "aligned*",
    "alignedat",
    "alignedat*",
    "array",
    "array*",
    "Bmatrix",
    "Bmatrix*",
    "bmatrix",
    "bmatrix*",
    "cases",
    "cases*",
    "CD",
    "CD*",
    "eqnarray",
    "eqnarray*",
    "equation",
    "equation*",
    "IEEEeqnarray",
    "IEEEeqnarray*",
    "subequations",
    "subequations*",
    "gather",
    "gather*",
    "gathered",
    "gathered*",
    "matrix",
    "matrix*",
    "multline",
    "multline*",
    "pmatrix",
    "pmatrix*",
    "smallmatrix",
    "smallmatrix*",
    "split",
    "split*",
    "subarray",
    "subarray*",
    "Vmatrix",
    "Vmatrix*",
    "vmatrix",
    "vmatrix*",
];

static DEFAULT_ENUM_ENVIRONMENTS: &[&str] = &["enumerate", "itemize", "description"];

static DEFAULT_VERBATIM_ENVIRONMENTS: &[&str] =
    &["pycode", "minted", "asy", "lstlisting", "verbatim"];

static DEFAULT_CITATION_COMMANDS: &[&str] = &[
    "cite",
    "cite*",
    "Cite",
    "nocite",
    "citet",
    "citet*",
    "citep",
    "citep*",
    "citeauthor",
    "citeauthor*",
    "Citeauthor",
    "Citeauthor*",
    "citetitle",
    "citetitle*",
    "citeyear",
    "citeyear*",
    "citedate",
    "citedate*",
    "citeurl",
    "fullcite",
    "citeyearpar",
    "citealt",
    "citealp",
    "citetext",
    "parencite",
    "parencite*",
    "Parencite",
    "footcite",
    "footfullcite",
    "footcitetext",
    "textcite",
    "Textcite",
    "smartcite",
    "supercite",
    "autocite",
    "autocite*",
    "Autocite",
    "Autocite*",
    "volcite",
    "Volcite",
    "pvolcite",
    "Pvolcite",
    "fvolcite",
    "ftvolcite",
    "svolcite",
    "Svolcite",
    "tvolcite",
    "Tvolcite",
    "avolcite",
    "Avolcite",
    "notecite",
    "pnotecite",
    "Pnotecite",
    "fnotecite",
    "citeA",
    "citeA*",
];
