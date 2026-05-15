import { mathjax } from "mathjax-full/js/mathjax.js";
import { TeX } from "mathjax-full/js/input/tex.js";
import { SVG } from "mathjax-full/js/output/svg.js";
import { liteAdaptor } from "mathjax-full/js/adaptors/liteAdaptor.js";
import { RegisterHTMLHandler } from "mathjax-full/js/handlers/html.js";
import { AllPackages } from "mathjax-full/js/input/tex/AllPackages.js";
import { OptionList } from "mathjax-full/js/util/Options";

const adaptor = liteAdaptor();
RegisterHTMLHandler(adaptor);

export default function(
	latex: string,
	options: OptionList | undefined,
): string {
	try {
		const tex = new TeX({ packages: AllPackages });
		const svg = new SVG();
		const doc = mathjax.document("", { InputJax: tex, OutputJax: svg });
		const node = doc.convert(latex, options);

		const svgString = adaptor.outerHTML(node);
		const svgTag = svgString.match(/<svg[^>]*>[\s\S]*<\/svg>/g)![0];

		// エラーをチェックする
		if (svgTag.includes("data-mjx-error")) {
			const errorTitle = svgTag.match(/title="([^"]+)"/)![1];
			throw new Error(errorTitle);
		}
		return svgTag;
	} catch (error: any) {
		throw new Error(`${error.message}`);
	}
}
