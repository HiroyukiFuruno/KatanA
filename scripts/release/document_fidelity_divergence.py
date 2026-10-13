"""上流の未解決差分を表示一致の成功へすり替えないため、実測と固定Issueを照合する。"""

HTML_ISSUE = "https://github.com/HiroyukiFuruno/katana-render-runtime/issues/106"
HTML_FIELDS = {
    "geometry.sticky_toc", "geometry.main", "geometry.visible_section",
    "active_toc", "visible_section_state",
}


class DivergenceError(ValueError):
    """測定されていない差分やhost要件を延期扱いにさせないための拒否。"""


def verify_disposition(receipt: object, measured_fields: set[str]) -> None:
    if not isinstance(receipt, dict):
        raise DivergenceError("HTML fidelity disposition is required")
    if not measured_fields:
        if receipt != {"status": "matched"}:
            raise DivergenceError("a matching measurement must not claim deferred divergence")
        return
    expected = {"status", "owner", "issue", "divergent_fields"}
    if set(receipt) != expected:
        raise DivergenceError("upstream disposition fields are incomplete or unknown")
    if (receipt["status"] != "known_upstream_divergence"
            or receipt["owner"] != "KRR" or receipt["issue"] != HTML_ISSUE):
        raise DivergenceError("only the unresolved KRR #106 HTML differential is deferred")
    fields = receipt["divergent_fields"]
    if (not isinstance(fields, list) or any(not isinstance(field, str) for field in fields)
            or len(set(fields)) != len(fields) or set(fields) != measured_fields
            or not measured_fields.issubset(HTML_FIELDS)):
        raise DivergenceError("deferred fields must exactly match measured HTML differences")
