use anyhow::Result;
use texlab::db::*;

fn main() -> Result<()> {
    let mut db = RootDatabase::default();
    Ok(())
}

// use std::sync::Arc;

// use anyhow::Result;
// use salsa::{InternId, InternKey};
// use texlab::{db::*, protocol::Uri, DocumentLanguage};

// static CODE: &str = r#"
// \documentclass[12pt]{article}
// \usepackage{lingmacros}
// \usepackage{tree-dvips}
// \begin{document}

// \section*{Notes for My Paper}\label{sec:foo}

// Don't forget to include examples of topicalization.
// They look like this:

// {\small
// \enumsentence{Topicalization from sentential subject:\\
// \shortex{7}{a John$_i$ [a & kltukl & [el &
//   {\bf l-}oltoir & er & ngii$_i$ & a Mary]]}
// { & {\bf R-}clear & {\sc comp} &
//   {\bf IR}.{\sc 3s}-love   & P & him & }
// {John, (it's) clear that Mary loves (him).}}
// }

// \subsection*{How to handle topicalization}

// I'll just assume a tree structure like (\ex{1}).

// {\small
// \enumsentence{Structure of A$'$ Projections:\\ [2ex]
// \begin{tabular}[t]{cccc}
//     & \node{i}{CP}\\ [2ex]
//     \node{ii}{Spec} &   &\node{iii}{C$'$}\\ [2ex]
//         &\node{iv}{C} & & \node{v}{SAgrP}
// \end{tabular}
// \nodeconnect{i}{ii}
// \nodeconnect{i}{iii}
// \nodeconnect{iii}{iv}
// \nodeconnect{iii}{v}
// }
// }

// \subsection*{Mood}

// Mood changes when there is a topic, as well as when
// there is WH-movement.  \emph{Irrealis} is the mood when
// there is a non-subject topic or WH-phrase in Comp.
// \emph{Realis} is the mood when there is a subject topic
// or WH-phrase.

// \end{document}

// "#;

// fn main() -> Result<()> {
//     let mut db = RootDatabase::default();
//     let document = db.intern_document(DocumentData {
//         uri: Arc::new(Uri::parse("http://www.example.com/foo.tex")?),
//     });

//     db.set_source_code(document, Arc::new(CODE.to_string()));
//     db.set_source_language(document, DocumentLanguage::Latex);
//     db.set_documents(Arc::new(vec![document]));

//     // println!("{:#?}", db.symbol_tree(document));
//     // println!(
//     //     "{:#?}",
//     //     db.lookup_intern_latex_section(LatexSection::from_intern_id(InternId::from(2u32)))
//     // );
//     // println!(
//     //     "{:#?}",
//     //     db.lookup_intern_latex_label_definition(LatexLabelDefinition::from_intern_id(
//     //         InternId::from(0u32)
//     //     ))
//     // );

//     Ok(())
// }
