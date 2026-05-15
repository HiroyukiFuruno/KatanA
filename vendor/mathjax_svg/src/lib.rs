use std::cell::RefCell;

use anyhow::{Context as _, anyhow};
use rquickjs::{Context, Function, Object, Runtime};
use thiserror::Error;

/// Exceptions related to this crate
#[derive(Error, Debug)]
pub enum Error {
    /// Error with exception thrown from JavaScript
    #[error("{0}")]
    JavaScriptException(String),
    /// Unknown error
    #[error("unexpected JavaScript return value")]
    UnexpectedReturnValue,
    /// Other error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// local shortcode of Result
type Result<T> = std::result::Result<T, Error>;

const EXPORT_SUFFIX: &str = "export{Nj as default};";
const FUNC_ID: &str = "__katana_mathjax_render";

thread_local! {
    static MATHJAX_CONTEXT: RefCell<Option<MathJaxContext>> = const { RefCell::new(None) };
}

struct MathJaxContext {
    _runtime: Runtime,
    context: Context,
}

/// Convert a math string to Svg
pub fn convert_to_svg(latex: impl AsRef<str>) -> Result<String> {
    convert_to_svg_inner(latex, true)
}

/// Convert a math string to Svg in inline mode
pub fn convert_to_svg_inline(latex: impl AsRef<str>) -> Result<String> {
    convert_to_svg_inner(latex, false)
}

fn convert_to_svg_inner(latex: impl AsRef<str>, display: bool) -> Result<String> {
    MATHJAX_CONTEXT.with(|context_cell| {
        let mut context_slot = context_cell.borrow_mut();
        if context_slot.is_none() {
            *context_slot = Some(initialize()?);
        }
        let context = context_slot
            .as_ref()
            .context("MathJax JavaScript context was not initialized")?;
        context.context.with(|ctx| {
            let config =
                Object::new(ctx.clone()).map_err(|error| Error::JavaScriptException(error.to_string()))?;
            config
                .set("display", display)
                .map_err(|error| Error::JavaScriptException(error.to_string()))?;
            let render: Function = ctx
                .globals()
                .get(FUNC_ID)
                .map_err(|error| Error::JavaScriptException(error.to_string()))?;
            render
                .call((latex.as_ref(), config))
                .map_err(|error| Error::JavaScriptException(error.to_string()))
        })
    })
}

fn initialize() -> Result<MathJaxContext> {
    let runtime = Runtime::new().context("failed to create QuickJS runtime")?;
    let context = Context::full(&runtime).context("failed to create QuickJS context")?;
    context.with(|ctx| {
        ctx.eval::<(), _>(patched_bundle()?.as_str())
            .map_err(|error| Error::JavaScriptException(error.to_string()))
    })?;
    Ok(MathJaxContext {
        _runtime: runtime,
        context,
    })
}

fn patched_bundle() -> Result<String> {
    let source = include_str!("../js/out/index.mjs");
    let export_start = source
        .rfind(EXPORT_SUFFIX)
        .context("MathJax bundle export marker was not found")?;
    let export_end = export_start + EXPORT_SUFFIX.len();
    if !source[export_end..].trim().is_empty() {
        return Err(anyhow!("MathJax bundle has unexpected content after export marker").into());
    }
    let script = &source[..export_start];
    Ok(format!("{script}globalThis.{FUNC_ID}=Nj;"))
}
