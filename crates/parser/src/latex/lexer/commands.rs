use crate::SyntaxConfig;

use super::types::{CommandName, ParagraphLevel, SectionLevel};

pub fn classify(name: &str, config: &SyntaxConfig) -> CommandName {
    match name {
        "begin" => CommandName::BeginEnvironment,
        "end" => CommandName::EndEnvironment,
        "[" => CommandName::BeginEquation,
        "]" => CommandName::EndEquation,
        "part" | "part*" => CommandName::Section(SectionLevel::Part),
        "chapter" | "chapter*" => CommandName::Section(SectionLevel::Chapter),
        "section" | "section*" => CommandName::Section(SectionLevel::Section),
        "subsection" | "subsection*" => CommandName::Section(SectionLevel::Subsection),
        "subsubsection" | "subsubsection*" => CommandName::Section(SectionLevel::Subsubsection),
        "paragraph" | "paragraph*" => CommandName::Paragraph(ParagraphLevel::Paragraph),
        "subparagraph" | "subparagraph*" => CommandName::Paragraph(ParagraphLevel::Subparagraph),
        "item" => CommandName::EnumItem,
        "caption" => CommandName::Caption,
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
        "newlabel" => CommandName::LabelNumber,
        "def" | "gdef" | "edef" | "xdef" | "let" | "glet" => CommandName::OldCommandDefinition,
        "newcommand"
        | "newcommand*"
        | "renewcommand"
        | "renewcommand*"
        | "DeclareRobustCommand"
        | "DeclareRobustCommand*"
        | "NewDocumentCommand"
        | "RenewDocumentCommand"
        | "ProvideDocumentCommand"
        | "DeclareDocumentCommand"
        | "NewExpandableDocumentCommand"
        | "RenewExpandableDocumentCommand"
        | "ProvideExpandableDocumentCommand"
        | "DeclareExpandableDocumentCommand"
        | "NewCommandCopy"
        | "RenewCommandCopy"
        | "DeclareCommandCopy" => CommandName::NewCommandDefinition,
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
        "newtheorem" | "newtheorem*" => CommandName::TheoremDefinitionAmsThm,
        "declaretheorem" | "declaretheorem*" => CommandName::TheoremDefinitionThmTools,
        "color" | "colorbox" | "textcolor" | "pagecolor" => CommandName::ColorReference,
        "definecolor" => CommandName::ColorDefinition,
        "definecolorset" => CommandName::ColorSetDefinition,
        "usepgflibrary" | "usetikzlibrary" => CommandName::TikzLibraryImport,
        "newenvironment"
        | "newenvironment*"
        | "renewenvironment"
        | "renewenvironment*"
        | "NewDocumentEnvironment"
        | "RenewDocumentEnvironment"
        | "ProvideDocumentEnvironment"
        | "DeclareDocumentEnvironment"
        | "NewEnvironmentCopy"
        | "RenewEnvironmentCopy"
        | "DeclareEnvironmentCopy" => CommandName::EnvironmentDefinition,
        "graphicspath" => CommandName::GraphicsPath,
        "iffalse" => CommandName::BeginBlockComment,
        "fi" => CommandName::EndBlockComment,
        "verb" => CommandName::VerbatimBlock,
        "bibitem" => CommandName::BibItem,
        "contentsline" => CommandName::TocContentsLine,
        "numberline" => CommandName::TocNumberLine,

        _ if config.citation_commands.contains(name) => CommandName::Citation,
        _ if config.label_definition_commands.contains(name) => CommandName::LabelDefinition,
        _ if config.label_reference_commands.contains(name) => CommandName::LabelReference,
        _ if config.label_reference_range_commands.contains(name) => {
            CommandName::LabelReferenceRange
        }
        _ => CommandName::Generic,
    }
}
