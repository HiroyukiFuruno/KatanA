#[cfg(test)]
mod tests {
    use crate::preview_pane::MathLogicOps;

    #[test]
    fn mathjax_backend_renders_svg_for_inline_and_block_math() {
        let inline_svg =
            MathLogicOps::render_math_svg("x^2", true).expect("inline MathJax SVG should render");
        let block_svg = MathLogicOps::render_math_svg("\\frac{1}{2}", false)
            .expect("block MathJax SVG should render");

        assert!(inline_svg.contains("<svg"));
        assert!(block_svg.contains("<svg"));
    }

    #[test]
    fn mathjax_backend_preserves_supported_tex_packages_after_bundle_upgrade() {
        let formulas = [
            ("ams", r"\begin{aligned}a&=b+c\\d&=e\end{aligned}"),
            ("cancel", r"\cancel{x+y}"),
            ("mathtools", r"A \coloneqq B"),
            ("mhchem", r"\ce{H2SO4}"),
            ("textmacros", r"\text{KatanA}"),
        ];

        for (package, formula) in formulas {
            let svg = MathLogicOps::render_math_svg(formula, false)
                .unwrap_or_else(|error| panic!("{package} formula must render: {error}"));
            assert!(svg.contains("<svg"), "{package} must produce SVG output");
            assert!(
                !svg.contains("data-mjx-error"),
                "{package} must not emit a MathJax error node"
            );
        }
    }
}
