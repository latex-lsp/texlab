use super::{
    compile::{Artifacts, CompileError, CompileParams, Compiler},
    kpsewhich::{KpsewhichError, Resolver},
    Distribution, DistributionKind,
};
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Tectonic;

#[async_trait]
impl Distribution for Tectonic {
    fn kind(&self) -> DistributionKind {
        DistributionKind::Tectonic
    }

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

    async fn load(&self) -> Result<(), KpsewhichError> {
        Ok(())
    }

    async fn resolver(&self) -> Arc<Resolver> {
        Arc::new(Resolver::default())
    }
}
