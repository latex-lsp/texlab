use crate::data::completion::dependency::LatexDependency;
use crate::data::kernel_primitives::{KERNEL_COMMANDS, KERNEL_ENVIRONMENTS};
use crate::tex;
use futures::compat::*;
use futures::prelude::*;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexComponent {
    pub file_names: Vec<String>,
    pub references: Vec<String>,
    pub commands: Vec<String>,
    pub environments: Vec<String>,
}

static PRIMITIVE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"[a-zA-Z]+"#).unwrap());

impl LatexComponent {
    pub async fn load(
        component: Vec<Arc<LatexDependency>>,
        loaded_refs: Vec<Arc<Self>>,
    ) -> Option<Self> {
        let dependency = &component[0];
        let candidates = Self::find_likely_primitives(&dependency, loaded_refs).await;

        let mut code = tex::build_test_code_header(&dependency.file)?;
        code += "\\makeatletter\n";
        code += "\\begin{document}\n";
        for candidate in &candidates {
            code += &format!("\\@ifundefined{{{}}}{{}} {{\n", candidate);
            code += &format!("\\@ifundefined{{end{}}}\n", candidate);
            code += &format!("{{ \\wlog{{cmd:{}}} }}\n", candidate);
            code += &format!("{{ \\wlog{{env:{}}} }} }}\n", candidate);
        }

        let format = dependency.file.as_path().into();
        let result = tex::compile("check.tex", &code, format).await.ok()?;
        let mut commands = Vec::new();
        let mut environments = Vec::new();
        for line in result.log.lines() {
            let primitive = line.split(":").nth(1).map(|x| x.to_owned());
            if line.starts_with("cmd:") {
                commands.push(primitive.unwrap())
            } else if line.starts_with("env:") {
                environments.push(primitive.unwrap())
            }
        }

        let file_names = component
            .iter()
            .map(|dep| dep.file.file_name().unwrap().to_str().unwrap().to_owned())
            .collect();

        let references = dependency
            .references()
            .map(|ref_| ref_.file_name().unwrap().to_str().unwrap().to_owned())
            .collect();

        Some(Self {
            file_names,
            references,
            commands,
            environments,
        })
    }

    async fn find_likely_primitives(
        dependency: &LatexDependency,
        loaded_refs: Vec<Arc<Self>>,
    ) -> HashSet<String> {
        let mut bytes = Vec::new();
        for include in dependency.includes.clone() {
            bytes.extend_from_slice(
                &tokio::fs::read(include)
                    .compat()
                    .unwrap_or_else(|_| panic!("Could not read include"))
                    .await,
            );
        }

        let mut likely_primitives: HashSet<_> = PRIMITIVE_REGEX
            .find_iter(&String::from_utf8_lossy(&bytes))
            .map(|x| x.as_str().to_owned())
            .collect();

        for primitive in loaded_refs
            .iter()
            .flat_map(|ref_| ref_.commands.iter().chain(ref_.environments.iter()))
            .map(String::as_str)
            .chain(
                KERNEL_COMMANDS
                    .iter()
                    .chain(KERNEL_ENVIRONMENTS.iter())
                    .map(Deref::deref),
            )
        {
            likely_primitives.remove(primitive);
        }

        likely_primitives
    }
}
