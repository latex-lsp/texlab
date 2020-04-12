use super::compile::{Artifacts, CompileError, CompileParams, Compiler};
use super::kpsewhich::{KpsewhichError, Resolver};
use super::{Distribution, DistributionKind};
use futures_boxed::boxed;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Tectonic;

impl Tectonic {
    pub fn new() -> Self {
        Self
    }
}

impl Distribution for Tectonic {
    fn kind(&self) -> DistributionKind {
        DistributionKind::Tectonic
    }

    #[boxed]
    async fn compile<'a>(&'a self, params: CompileParams<'a>) -> Result<Artifacts, CompileError> {
        let args = [params.file_name];
        let compiler = Compiler {
            executable: "tectonic",
            args: &args,
            file_name: params.file_name,
            timeout: params.timeout,
        };
        compiler.compile(params.code).await
    }

    #[boxed]
    async fn load(&self) -> Result<(), KpsewhichError> {
        Ok(())
    }

    #[boxed]
    async fn resolver(&self) -> Arc<Resolver> {
        Arc::new(Resolver::default())
    }
}
