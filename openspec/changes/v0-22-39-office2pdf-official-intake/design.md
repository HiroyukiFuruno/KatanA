## Context

KDV v0.5.3 は公式 `office2pdf = "=0.6.7"` を registry から取得する。KatanA の
v0.22.39 はこの公開済み KDV を利用する consumer patch release であり、表示 engine
や format semantics を所有しない。

## Decisions

### D1. Dependency direction remains unchanged

Dependency direction is `KatanA -> KDV -> KUC/KRR`. KatanA は KDV の neutral document
session / frame / command を既存 egui backend に投影するだけとし、KUC を直接依存に
追加しない。KDV の worker、font_paths、Office conversion、page/grid/slide layout、
hit-test、selection は KDV 所有のままとする。

### D2. Official registry chain is release-critical

KatanA は `katana-document-viewer = "=0.5.3"` を使用する。Cargo.lock は registry source
の `katana-document-viewer 0.5.3` と transitive `office2pdf 0.6.7` を正確に解決し、
`office2pdf-katana`、path、git source を含んではならない。KatanA は `office2pdf` を
direct dependency にしない。

### D3. Existing functionality is regression-tested, not reimplemented

PDF / DOCX / XLSX / PPTX の source intake、KDV command、diagnostics、headless corpus、
sidecar packaging をそのまま再利用する。PPTX は #745 の paragraph line advance と
font fallback を含む KDV v0.5.3 を、macOS / Linux / Windows で検証する。変更対象は
dependency provenance と release evidence だけである。

### D4. Release target and evidence

GitHub の最新公開版 v0.22.38 から許可される次版は v0.22.39 のみとする。v0.22.40、
minor/major jump、withdrawn v0.29.0 は SemVer guard で拒否する。公開前に strict
coverage 100% / uncovered 0、OpenSpec strict validation、3 OS headless acceptance、
package asset contract を通し、公開後に GitHub Release と asset を再取得して確認する。
