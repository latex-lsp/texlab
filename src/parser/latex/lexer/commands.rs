use super::types::CommandName;

pub fn classify(name: &str) -> CommandName {
    match name {
        "begin" => CommandName::BeginEnvironment,
        "end" => CommandName::EndEnvironment,
        "[" => CommandName::BeginEquation,
        "]" => CommandName::EndEquation,
        "part" | "part*" => CommandName::Part,
        "chapter" | "chapter*" => CommandName::Chapter,
        "section" | "section*" => CommandName::Section,
        "subsection" | "subsection*" => CommandName::Subsection,
        "subsubsection" | "subsubsection*" => CommandName::Subsubsection,
        "paragraph" | "paragraph*" => CommandName::Paragraph,
        "subparagraph" | "subparagraph*" => CommandName::Subparagraph,
        "item" => CommandName::EnumItem,
        "caption" => CommandName::Caption,
        "cite" | "cite*" | "Cite" | "nocite" | "citet" | "citet*" | "citep" | "citep*"
        | "citeauthor" | "citeauthor*" | "Citeauthor" | "Citeauthor*" | "citetitle"
        | "citetitle*" | "citeyear" | "citeyear*" | "citedate" | "citedate*" | "citeurl"
        | "fullcite" | "citeyearpar" | "citealt" | "citealp" | "citetext" | "parencite"
        | "parencite*" | "Parencite" | "footcite" | "footfullcite" | "footcitetext"
        | "textcite" | "Textcite" | "smartcite" | "supercite" | "autocite" | "autocite*"
        | "Autocite" | "Autocite*" | "volcite" | "Volcite" | "pvolcite" | "Pvolcite"
        | "fvolcite" | "ftvolcite" | "svolcite" | "Svolcite" | "tvolcite" | "Tvolcite"
        | "avolcite" | "Avolcite" | "notecite" | "pnotecite" | "Pnotecite" | "fnotecite"
        | "citeA" | "citeA*" => CommandName::Citation,
        "usepackage" | "RequirePackage" => CommandName::PackageInclude,
        "documentclass" => CommandName::ClassInclude,
        "include" | "subfileinclude" | "input" | "subfile" => CommandName::LatexInclude,
        "addbibresource" => CommandName::BiblatexInclude,
        "bibliography" => CommandName::BibtexInclude,
        "includegraphics" => CommandName::GraphicsInclude,
        "includesvg" => CommandName::SvgInclude,
        "includeinkscape" => CommandName::InkscapeInclude,
        "verbatiminput" | "VerbatimInput" => CommandName::VerbatimInclude,
        "import" | "subimport" | "inputfrom" | "subinputfrom" | "subincludefrom" => {
            CommandName::Import
        }
        "label" => CommandName::LabelDefinition,
        "ref" | "vref" | "Vref" | "autoref" | "pageref" | "cref" | "cref*" | "Cref" | "Cref*"
        | "namecref" | "nameCref" | "lcnamecref" | "namecrefs" | "nameCrefs" | "lcnamecrefs"
        | "labelcref" | "labelcpageref" | "eqref" => CommandName::LabelReference,
        "crefrange" | "crefrange*" | "Crefrange" | "Crefrange*" => CommandName::LabelReferenceRange,
        "newlabel" => CommandName::LabelNumber,
        "newcommand"
        | "newcommand*"
        | "renewcommand"
        | "renewcommand*"
        | "DeclareRobustCommand"
        | "DeclareRobustCommand*" => CommandName::CommandDefinition,
        "DeclareMathOperator" | "DeclareMathOperator*" => CommandName::MathOperator,
        "newglossaryentry" => CommandName::GlossaryEntryDefinition,
        "gls" | "Gls" | "GLS" | "glspl" | "Glspl" | "GLSpl" | "glsdisp" | "glslink" | "glstext"
        | "Glstext" | "GLStext" | "glsfirst" | "Glsfirst" | "GLSfirst" | "glsplural"
        | "Glsplural" | "GLSplural" | "glsfirstplural" | "Glsfirstplural" | "GLSfirstplural"
        | "glsname" | "Glsname" | "GLSname" | "glssymbol" | "Glssymbol" | "glsdesc" | "Glsdesc"
        | "GLSdesc" | "glsuseri" | "Glsuseri" | "GLSuseri" | "glsuserii" | "Glsuserii"
        | "glsuseriii" | "glsuseriv" | "Glsuseriv" | "GLSuseriv" | "glsuserv" | "Glsuserv"
        | "GLSuserv" | "glsuservi" | "Glsuservi" | "GLSuservi" => {
            CommandName::GlossaryEntryReference
        }
        "newacronym" | "newacro" | "acrodef" | "acro" | "newacroindefinite"
        | "acrodefindefinite" | "acroindefinite" | "acroplural" | "newacroplural"
        | "acrodefplural" => CommandName::AcronymDefinition,
        "DeclareAcronym" => CommandName::AcronymDeclaration,
        "acrshort" | "Acrshort" | "ACRshort" | "acrshortpl" | "Acrshortpl" | "ACRshortpl"
        | "acrlong" | "Acrlong" | "ACRlong" | "acrlongpl" | "Acrlongpl" | "ACRlongpl"
        | "acrfull" | "Acrfull" | "ACRfull" | "acrfullpl" | "Acrfullpl" | "ACRfullpl" | "acs"
        | "Acs" | "acsp" | "Acsp" | "acl" | "Acl" | "aclp" | "Aclp" | "acf" | "Acf" | "acfi"
        | "Acfi" | "acfp" | "Acfp" | "ac" | "Ac" | "acp" | "Acp" | "acused" | "acsu" | "Aclu"
        | "iac" | "Iac" | "acs*" | "Acs*" | "acsp*" | "Acsp*" | "acl*" | "Acl*" | "aclp*"
        | "Aclp*" | "acf*" | "Acf*" | "acfi*" | "Acfi*" | "acfp*" | "Acfp*" | "ac*" | "Ac*"
        | "acp*" | "Acp*" | "acused*" | "acsu*" | "Aclu*" | "iac*" | "Iac*" | "glsentrylong"
        | "Glsentrylong" | "glsentrylongpl" | "Glsentrylongpl" | "glsentryshort"
        | "Glsentryshort" | "glsentryshortpl" | "Glsentryshortpl" | "glsentryfullpl"
        | "Glsentryfullpl" => CommandName::AcronymReference,
        "newtheorem" | "newtheorem*" | "declaretheorem" | "declaretheorem*" => {
            CommandName::TheoremDefinition
        }
        "color" | "colorbox" | "textcolor" | "pagecolor" => CommandName::ColorReference,
        "definecolor" => CommandName::ColorDefinition,
        "definecolorset" => CommandName::ColorSetDefinition,
        "usepgflibrary" | "usetikzlibrary" => CommandName::TikzLibraryImport,
        "newenvironment" | "newenvironment*" | "renewenvironment" | "renewenvironment*" => {
            CommandName::EnvironmentDefinition
        }
        "graphicspath" => CommandName::GraphicsPath,
        "iffalse" => CommandName::BeginBlockComment,
        "fi" => CommandName::EndBlockComment,
        _ => CommandName::Generic,
    }
}
