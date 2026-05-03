use std::borrow::Cow;
use std::sync::OnceLock;

static V8_INIT: OnceLock<Result<(), String>> = OnceLock::new();

pub(crate) struct DiagramRuntimeScript<'a> {
    pub(crate) name: &'static str,
    pub(crate) code: Cow<'a, str>,
}

impl<'a> DiagramRuntimeScript<'a> {
    pub(crate) fn borrowed(name: &'static str, code: &'a str) -> Self {
        Self {
            name,
            code: Cow::Borrowed(code),
        }
    }

    pub(crate) fn owned(name: &'static str, code: String) -> Self {
        Self {
            name,
            code: Cow::Owned(code),
        }
    }
}

pub(crate) struct DiagramV8Runtime;

impl DiagramV8Runtime {
    pub(crate) fn render(scripts: &[DiagramRuntimeScript<'_>]) -> Result<String, String> {
        Self::ensure_initialized()?;

        let mut isolate = v8::Isolate::new(Default::default());
        let handle_scope = &mut v8::HandleScope::new(&mut isolate);
        let context = v8::Context::new(handle_scope, Default::default());
        let scope = &mut v8::ContextScope::new(handle_scope, context);
        let scope = &mut v8::TryCatch::new(scope);

        let mut rendered = String::new();
        for script in scripts {
            rendered = evaluate(scope, script)?;
        }
        Ok(rendered)
    }

    fn ensure_initialized() -> Result<(), String> {
        V8_INIT
            .get_or_init(|| {
                /* WHY: mathjax_svg and rusty_v8 share the same global V8 state.
                Keep a single initialization owner to avoid panic during later math rendering. */
                mathjax_svg::convert_to_svg_inline("x")
                    .map(|_| ())
                    .map_err(|err| format!("Failed to initialize shared V8 runtime: {err}"))
            })
            .clone()
    }
}

fn evaluate(
    scope: &mut v8::TryCatch<v8::HandleScope>,
    script: &DiagramRuntimeScript<'_>,
) -> Result<String, String> {
    let source = v8::String::new(scope, script.code.as_ref())
        .ok_or_else(|| "source allocation failed".to_string())?;
    let origin_name = v8::String::new(scope, script.name)
        .ok_or_else(|| "filename allocation failed".to_string())?;
    let origin = script_origin(scope, origin_name);
    let script = v8::Script::compile(scope, source, Some(&origin))
        .ok_or_else(|| exception_message(scope))?;
    let value = script.run(scope).ok_or_else(|| exception_message(scope))?;
    resolve_value(scope, value)
}

fn resolve_value(
    scope: &mut v8::TryCatch<v8::HandleScope>,
    value: v8::Local<v8::Value>,
) -> Result<String, String> {
    let Ok(promise) = v8::Local::<v8::Promise>::try_from(value) else {
        return Ok(value.to_rust_string_lossy(scope));
    };
    scope.perform_microtask_checkpoint();
    match promise.state() {
        v8::PromiseState::Fulfilled => Ok(promise.result(scope).to_rust_string_lossy(scope)),
        v8::PromiseState::Rejected => Err(promise.result(scope).to_rust_string_lossy(scope)),
        v8::PromiseState::Pending => Err("Diagram render Promise did not settle".to_string()),
    }
}

fn script_origin<'a>(
    scope: &mut v8::TryCatch<v8::HandleScope<'a>>,
    origin_name: v8::Local<'a, v8::String>,
) -> v8::ScriptOrigin<'a> {
    v8::ScriptOrigin::new(
        scope,
        origin_name.into(),
        0,
        0,
        false,
        0,
        Some(origin_name.into()),
        false,
        false,
        false,
        None,
    )
}

fn exception_message(scope: &mut v8::TryCatch<v8::HandleScope>) -> String {
    let Some(exception) = scope.exception() else {
        return "unknown V8 exception".to_string();
    };
    let message = exception.to_rust_string_lossy(scope);
    let Some(object) = exception.to_object(scope) else {
        return message;
    };
    if let Some(stack_key) = v8::String::new(scope, "stack")
        && let Some(stack) = object.get(scope, stack_key.into())
    {
        let stack = stack.to_rust_string_lossy(scope);
        if !stack.is_empty() && stack != message {
            return format!("{message}\n{stack}");
        }
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bootstrap_v8() {
        let _ = DiagramV8Runtime::render(&[DiagramRuntimeScript::borrowed(
            "diagram_js_runtime_test_bootstrap.js",
            "'bootstrap'",
        )])
        .expect("bootstrap V8 runtime");
    }

    fn with_try_catch<T>(f: impl FnOnce(&mut v8::TryCatch<v8::HandleScope>) -> T) -> T {
        bootstrap_v8();
        let mut isolate = v8::Isolate::new(Default::default());
        let handle_scope = &mut v8::HandleScope::new(&mut isolate);
        let context = v8::Context::new(handle_scope, Default::default());
        let scope = &mut v8::ContextScope::new(handle_scope, context);
        let scope = &mut v8::TryCatch::new(scope);
        f(scope)
    }

    fn evaluate_script(source: &str) -> Result<String, String> {
        DiagramV8Runtime::render(&[DiagramRuntimeScript::borrowed(
            "diagram_js_runtime_test.js",
            source,
        )])
    }

    #[test]
    fn render_plain_value_from_script() {
        let result = evaluate_script("'ok'");
        assert_eq!(result.expect("expected rendered string"), "ok");
    }

    #[test]
    fn render_rejected_promise_as_error() {
        let result = evaluate_script("Promise.reject(new Error('rejected'))");
        let message = result.expect_err("expected promise rejection");
        assert!(message.contains("rejected"));
    }

    #[test]
    fn exception_is_unknown_when_scope_has_no_exception() {
        let message = with_try_catch(exception_message);
        assert_eq!(message, "unknown V8 exception");
    }

    #[test]
    fn exception_without_object_returns_message() {
        let message = evaluate_script("throw null");
        assert_eq!(message.expect_err("expected throw-null error"), "null");
    }

    #[test]
    fn exception_without_stack_property_returns_message() {
        let message = evaluate_script("(() => { throw { message: 'no-stack' }; })()");
        assert!(!message.expect_err("expected no-stack error").is_empty());
    }

    #[test]
    fn exception_with_equal_stack_returns_message() {
        let message = evaluate_script(
            "(() => { const error = new Error('same-stack'); error.stack = 'Error: same-stack'; throw error; })()",
        );
        assert_eq!(
            message.expect_err("expected same-stack error"),
            "Error: same-stack"
        );
    }

    #[test]
    fn exception_with_empty_stack_returns_base_message() {
        let message = evaluate_script(
            "(() => { const error = new Error('empty-stack'); error.stack = ''; throw error; })()",
        );
        assert_eq!(
            message.expect_err("expected empty-stack error"),
            "Error: empty-stack"
        );
    }

    #[test]
    fn exception_with_stack_message_separate_from_message_returns_message_with_stack() {
        let message = evaluate_script("throw new Error('stacked')");
        let message = message.expect_err("expected stacked error");
        assert!(message.contains("stacked"));
        assert!(message.contains("at "));
    }

    #[test]
    fn exception_with_failing_stack_access_uses_base_message() {
        let message = evaluate_script(
            r#"
            (() => {
                const error = new Error('base-message');
                Object.defineProperty(error, 'stack', {
                    get() {
                        throw new Error('stack-access');
                    },
                });
                throw error;
            })()
            "#,
        );
        assert_eq!(
            message.expect_err("expected access-failure error"),
            "Error: base-message"
        );
    }
}
