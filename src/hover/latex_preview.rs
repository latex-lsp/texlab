use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::latex::*;
use crate::syntax::text::{CharStream, SyntaxNode};
use crate::syntax::SyntaxTree;
use futures::compat::*;
use futures_boxed::boxed;
use image::png::PNGEncoder;
use image::DynamicImage;
use image::GenericImageView;
use lsp_types::*;
use std::borrow::Cow;
use std::io::Cursor;
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::{tempdir, TempDir};
use tokio_process::CommandExt;
use wait_timeout::ChildExt;

const PREVIEW_ENVIRONMENTS: &[&str] = &[
    "align",
    "align",
    "alignat",
    "aligned",
    "alignedat",
    "array",
    "Bmatrix",
    "bmatrix",
    "cases",
    "CD",
    "eqnarray",
    "equation",
    "equation",
    "gather",
    "gathered",
    "matrix",
    "multline",
    "pmatrix",
    "smallmatrix",
    "split",
    "subarray",
    "Vmatrix",
    "vmatrix",
];

const IGNORED_PACKAGES: &[&str] = &["biblatex", "pgf", "tikz"];

#[derive(Debug, PartialEq, Eq, Clone)]
enum MathElement<'a> {
    Environment(&'a LatexEnvironment),
    Equation(&'a LatexEquation),
    Inline(&'a LatexGroup),
}

impl<'a> SyntaxNode for MathElement<'a> {
    fn range(&self) -> Range {
        match self {
            MathElement::Environment(environment) => environment.range(),
            MathElement::Equation(equation) => equation.range(),
            MathElement::Inline(inline) => inline.range(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum RenderError {
    IO,
    LatexNotInstalled,
    LatexFaulty,
    Timeout,
    DviPngNotInstalled,
    DviPngFaulty,
    CannotDecodeImage,
}

pub struct LatexPreviewHoverProvider;

impl LatexPreviewHoverProvider {
    fn is_math_environment(environment: &LatexEnvironment) -> bool {
        let canonical_name = environment
            .left
            .name()
            .map(LatexToken::text)
            .unwrap_or_default()
            .replace('*', "");
        PREVIEW_ENVIRONMENTS.contains(&canonical_name.as_ref())
    }

    async fn render(
        request: &FeatureRequest<TextDocumentPositionParams>,
        range: Range,
    ) -> Result<Hover, RenderError> {
        let code = Self::generate_code(request, range);
        let directory = Self::compile(&code).await?;
        let image = Self::dvipng(&directory).await?;
        let base64 = Self::encode_image(image);
        let markdown = format!("![preview](data:image/png;base64,{})", base64);
        Ok(Hover {
            range: Some(range),
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: Cow::from(markdown),
            }),
        })
    }

    fn generate_code(request: &FeatureRequest<TextDocumentPositionParams>, range: Range) -> String {
        let mut code = String::new();
        code.push_str("\\documentclass{article}\n");
        code.push_str("\\thispagestyle{empty}\n");
        Self::generate_includes(request, &mut code);
        // TODO: Include command definitions
        Self::generate_math_operators(request, &mut code);
        code.push_str("\\begin{document}\n");
        code.push_str(&Self::extract_text(&request.document.text, range));
        code.push('\n');
        code.push_str("\\end{document}\n");
        code
    }

    fn generate_includes(request: &FeatureRequest<TextDocumentPositionParams>, code: &mut String) {
        for document in &request.related_documents {
            if let SyntaxTree::Latex(tree) = &document.tree {
                let text = &request.document.text;
                tree.includes
                    .iter()
                    .filter(|include| include.kind() == LatexIncludeKind::Package)
                    .filter(|include| !IGNORED_PACKAGES.contains(&include.path().text()))
                    .for_each(|include| {
                        code.push_str(&Self::extract_text(&text, include.command.range));
                        code.push('\n');
                    });
            }
        }
    }

    fn generate_math_operators(
        request: &FeatureRequest<TextDocumentPositionParams>,
        code: &mut String,
    ) {
        for document in &request.related_documents {
            if let SyntaxTree::Latex(tree) = &document.tree {
                tree.math_operators
                    .iter()
                    .map(|op| Self::extract_text(&document.text, op.range()))
                    .for_each(|op| {
                        code.push_str(&op);
                        code.push('\n');
                    })
            }
        }
    }

    async fn compile(code: &str) -> Result<TempDir, RenderError> {
        let directory = tempdir().map_err(|_| RenderError::IO)?;

        let tex_file = directory.path().join("preview.tex");
        tokio::fs::write(tex_file, code)
            .compat()
            .await
            .map_err(|_| RenderError::IO)?;

        let mut process = Command::new("latex")
            .args(&["--interaction=nonstopmode", "preview.tex"])
            .current_dir(&directory)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| RenderError::LatexNotInstalled)?;

        match process
            .wait_timeout(Duration::from_secs(10))
            .map_err(|_| RenderError::LatexFaulty)?
        {
            Some(_) => Ok(directory),
            None => {
                process.kill().map_err(|_| RenderError::Timeout)?;
                Err(RenderError::Timeout)
            }
        }
    }

    async fn dvipng(directory: &TempDir) -> Result<DynamicImage, RenderError> {
        let process = Command::new("dvipng")
            .args(&["-D", "200", "-T", "tight", "preview.dvi"])
            .current_dir(directory.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn_async()
            .map_err(|_| RenderError::DviPngNotInstalled)?;

        process
            .compat()
            .await
            .map_err(|_| RenderError::DviPngFaulty)?;

        let png_file = directory.path().join("preview1.png");
        let png = image::open(png_file).map_err(|_| RenderError::CannotDecodeImage)?;
        Ok(png)
    }

    fn encode_image(image: DynamicImage) -> String {
        let mut image_buf = Cursor::new(Vec::new());
        let png_encoder = PNGEncoder::new(&mut image_buf);
        png_encoder
            .encode(
                &image.raw_pixels(),
                image.width(),
                image.height(),
                image.color(),
            )
            .unwrap();
        base64::encode(&image_buf.into_inner())
    }

    fn extract_text(text: &str, range: Range) -> String {
        let mut stream = CharStream::new(text);
        stream.seek(range.start);
        stream.start_span();
        stream.seek(range.end);
        stream.end_span().text
    }
}

impl FeatureProvider for LatexPreviewHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let mut elements = Vec::new();
            tree.environments
                .iter()
                .filter(|env| Self::is_math_environment(env))
                .map(MathElement::Environment)
                .for_each(|env| elements.push(env));

            tree.equations
                .iter()
                .map(MathElement::Equation)
                .for_each(|eq| elements.push(eq));

            tree.inlines
                .iter()
                .map(|inline| MathElement::Inline(&inline))
                .for_each(|inline| elements.push(inline));

            let range = elements
                .iter()
                .find(|elem| elem.range().contains(request.params.position))
                .map(MathElement::range)?;

            return Some(Self::render(request, range).await.ok()?);
        }
        None
    }
}
