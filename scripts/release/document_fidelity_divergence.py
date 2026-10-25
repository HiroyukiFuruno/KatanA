"""上流の未解決差分を表示一致の成功へすり替えないため、実測と固定Issueを照合する。"""

import math


HTML_ISSUE = "https://github.com/HiroyukiFuruno/katana-render-runtime/issues/106"
HTML_FIELDS = {
    "geometry.sticky_toc", "geometry.main", "geometry.visible_section",
    "active_toc", "visible_section_state",
}
PPTX_FIRST_FRAME_APPROVAL = "2026-10-24-pptx-krr-advance-publication"
PPTX_FIRST_FRAME_DISPOSITION_FIELDS = {
    "status", "owner", "approval", "release_target", "scope", "limit_ms",
    "measured_first_frame_ms", "unmet",
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


def verify_pptx_first_frame_disposition(receipt: object, first_frame: float) -> None:
    if not isinstance(receipt, dict) or set(receipt) != PPTX_FIRST_FRAME_DISPOSITION_FIELDS:
        raise DivergenceError("PPTX first-frame disposition fields are incomplete or unknown")
    if (receipt["status"] != "known_krr_pptx_first_frame_miss"
            or receipt["owner"] != "KRR"
            or receipt["approval"] != PPTX_FIRST_FRAME_APPROVAL
            or receipt["release_target"] != "v0.22.42"
            or receipt["scope"] != "katana-host"
            or receipt["limit_ms"] != 15000
            or receipt["unmet"] is not True):
        raise DivergenceError("only the approved KRR PPTX first-frame miss is deferred")
    measured = receipt["measured_first_frame_ms"]
    if (isinstance(measured, bool) or not isinstance(measured, (int, float))
            or not math.isfinite(measured) or measured <= 15000 or measured != first_frame):
        raise DivergenceError("PPTX first-frame disposition must retain the raw unmet measurement")
