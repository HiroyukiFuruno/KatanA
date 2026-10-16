#[cfg(test)]
mod tests {
    use crate::preview_pane::MathLogicOps;

    fn render_on_small_stack(formula: &'static str, inline: bool) -> String {
        std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(move || MathLogicOps::render_math_svg(formula, inline))
            .expect("small-stack caller should spawn")
            .join()
            .expect("small-stack caller should finish")
            .expect("MathJax SVG should render")
    }

    fn assert_package_renders(formula: &'static str) {
        let svg = render_on_small_stack(formula, false);
        assert!(svg.contains("<svg"));
        assert!(!svg.contains("data-mjx-error"));
    }

    #[test]
    fn mathjax_backend_renders_svg_for_inline_and_block_math() {
        let inline_svg = render_on_small_stack("x^2", true);
        let block_svg = render_on_small_stack("\\frac{1}{2}", false);

        assert!(inline_svg.contains("<svg"));
        assert!(block_svg.contains("<svg"));
    }

    #[test]
    fn mathjax_backend_preserves_ams_on_small_stack() {
        assert_package_renders(r"\begin{aligned}a&=b+c\\d&=e\end{aligned}");
    }

    #[test]
    fn mathjax_backend_preserves_cancel_on_small_stack() {
        assert_package_renders(r"\cancel{x+y}");
    }

    #[test]
    fn mathjax_backend_preserves_mathtools_on_small_stack() {
        assert_package_renders(r"A \coloneqq B");
    }

    #[test]
    fn mathjax_backend_preserves_mhchem_on_small_stack() {
        assert_package_renders(r"\ce{H2SO4}");
    }

    #[test]
    fn mathjax_backend_preserves_textmacros_on_small_stack() {
        assert_package_renders(r"\text{KatanA}");
    }

    #[test]
    fn mathjax_backend_preserves_physics_on_small_stack() {
        for (formula, block_width, inline_width) in [
            (r"\qty{1}{m}", "5.757ex", "5.757ex"),
            (r"\dv{f}{x}", "3.548ex", "2.8ex"),
        ] {
            for inline in [false, true] {
                let svg = render_on_small_stack(formula, inline);
                let width = if inline { inline_width } else { block_width };
                assert!(svg.contains(&format!("data-latex=\"{formula}\"")));
                assert!(svg.contains(&format!("width=\"{width}\"")));
                assert!(!svg.contains("data-mjx-error"));
                assert!(!svg.contains("merror"));
            }
        }
    }

    #[test]
    fn mathjax_backend_matches_concurrent_requests_to_their_replies() {
        let formulas = [
            ("x", false),
            ("y", true),
            (r"\frac{1}{2}", true),
            (r"\ce{H2SO4}", false),
        ];
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(formulas.len()));
        let callers: Vec<_> = formulas
            .into_iter()
            .map(|(formula, inline)| {
                let expected = render_on_small_stack(formula, inline);
                let barrier = barrier.clone();
                std::thread::Builder::new()
                    .stack_size(2 * 1024 * 1024)
                    .spawn(move || {
                        barrier.wait();
                        let actual = MathLogicOps::render_math_svg(formula, inline)
                            .expect("concurrent MathJax SVG should render");
                        assert_eq!(
                            actual, expected,
                            "response must match {formula}, inline={inline}"
                        );
                    })
                    .expect("concurrent caller should spawn")
            })
            .collect();
        for caller in callers {
            caller.join().expect("concurrent caller should finish");
        }
    }

    #[test]
    fn mathjax_backend_does_not_leak_macros_between_requests() {
        let formula = r"\katanaworkertestmacro";
        let undefined = render_on_small_stack(formula, false);
        let defined = render_on_small_stack(
            r"\newcommand{\katanaworkertestmacro}{x}\katanaworkertestmacro",
            false,
        );
        assert_ne!(defined, undefined);
        assert_eq!(render_on_small_stack(formula, false), undefined);
    }

    #[test]
    fn mathjax_backend_returns_js_errors_and_recovers_for_the_next_request() {
        let error = mathjax_svg::convert_to_svg(r"\begin{aligned}x")
            .expect_err("unclosed environment must fail");
        assert!(matches!(error, mathjax_svg::Error::JavaScriptException(_)));
        assert_package_renders("x^2");
    }
}
