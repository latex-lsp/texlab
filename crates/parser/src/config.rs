use rustc_hash::FxHashSet;

#[derive(Debug)]
pub struct SyntaxConfig {
    pub follow_package_links: bool,
    pub math_environments: FxHashSet<String>,
    pub enum_environments: FxHashSet<String>,
    pub verbatim_environments: FxHashSet<String>,
    pub citation_commands: FxHashSet<String>,
    pub label_definition_commands: FxHashSet<String>,
    pub label_reference_commands: FxHashSet<String>,
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

        let label_definition_commands = DEFAULT_LABEL_DEFINITION_COMMANDS
            .iter()
            .map(ToString::to_string)
            .collect();

        let label_reference_commands = DEFAULT_LABEL_REFERENCE_COMMANDS
            .iter()
            .map(ToString::to_string)
            .collect();

        Self {
            follow_package_links: false,
            math_environments,
            enum_environments,
            verbatim_environments,
            citation_commands,
            label_definition_commands,
            label_reference_commands,
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
    "bmatrix",
    "Bmatrix",
    "bmatrix*",
    "Bmatrix*",
    "cases",
    "cases*",
    "CD",
    "CD*",
    "eqnarray",
    "eqnarray*",
    "equation",
    "equation*",
    "flalign",
    "flalign*",
    "gather",
    "gather*",
    "gathered",
    "gathered*",
    "IEEEeqnarray",
    "IEEEeqnarray*",
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
    "subequations",
    "subequations*",
    "vmatrix",
    "Vmatrix",
    "vmatrix*",
    "Vmatrix*",
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

static DEFAULT_LABEL_DEFINITION_COMMANDS: &[&str] = &["label"];

static DEFAULT_LABEL_REFERENCE_COMMANDS: &[&str] = &[
    "ref",
    "vref",
    "Vref",
    "autoref",
    "pageref",
    "cref",
    "cref*",
    "Cref",
    "Cref*",
    "namecref",
    "nameCref",
    "lcnamecref",
    "namecrefs",
    "nameCrefs",
    "lcnamecrefs",
    "labelcref",
    "labelcpageref",
    "eqref",
];
