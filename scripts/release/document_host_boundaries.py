"""Verify real frame receipts at manual open and close boundaries."""

from document_host_io import digest, fields, require
from document_host_observation import receipt as parse_receipt


def require_closed(receipt: dict) -> None:
    require(receipt.get("document_surfaces") == [] and receipt.get("html_observations") == [],
            "document must be closed before open timing starts")


def surface(receipt: dict, kind: str) -> tuple[str, int, int | None]:
    key = "html_observations" if kind == "html" else "document_surfaces"
    other = "document_surfaces" if kind == "html" else "html_observations"
    values = receipt.get(key)
    require(isinstance(values, list) and len(values) == 1 and receipt.get(other) == [],
            "one actual open surface required")
    value = values[0]
    if kind == "html":
        return digest(value["source_uri"].encode()), value["katana_renderer_session_epoch"], value["katana_adapter_instance_id"]
    return digest(value["uri"].encode()), value["generation"], None


def require_same_surface(opened: dict, closing: dict, kind: str) -> tuple[int, int]:
    first, last = surface(opened, kind), surface(closing, kind)
    require(first[0] == last[0] and first[2] == last[2], "close-start surface identity differs")
    require(kind == "html" or first[1] == last[1], "Office generation changed before close")
    return first[1], last[1]


def validate_case(run, label: str, case: dict, transcript: dict) -> None:
    start = fields(run.json(label + ".start.json"), {"pid", "start_identity", "input", "input_sha256", "kind",
                                                    "start_seq", "receipt_sha256", "observedAt"}, "case start")
    close = fields(run.json(label + ".close-start.json"), {"seq", "receipt_sha256", "observedAt"}, "case close start")
    require(start["pid"] == transcript["pid"] and start["start_identity"] == transcript["start_identity"]
            and start["input_sha256"] == case["input_sha256"] and start["kind"] == case["kind"], "case start identity differs")
    start_raw, close_raw = (run.raw(label + suffix) for suffix in ("-start.receipt.json", "-close-start.receipt.json"))
    require(digest(start_raw) == start["receipt_sha256"] and digest(close_raw) == close["receipt_sha256"], "boundary receipt changed")
    before = parse_receipt(start_raw, transcript["run_id"], transcript["pid"])
    closing = parse_receipt(close_raw, transcript["run_id"], transcript["pid"])
    opened = parse_receipt(run.raw(label + "-open.receipt.json"), transcript["run_id"], transcript["pid"])
    require_closed(before)
    require(before["seq"] == start["start_seq"] < case["open_seq"] <= close["seq"] == closing["seq"] < case["close_seq"],
            "boundary receipt sequence differs")
    require(start["observedAt"] == case["open_started_at"] and close["observedAt"] == case["close_started_at"],
            "boundary observation time differs")
    initial, final = require_same_surface(opened, closing, case["kind"])
    uri, generation, adapter = surface(opened, case["kind"])
    require((uri, generation, adapter) == (case["uri_sha256"], case["generation"], case["adapter_instance_id"]),
            "boundary open identity differs")
    if initial != final:
        require(any(op["name"] == "reload" and op["kind"] == "html" and op["uri_sha256"] == uri
                    and op["generation"] == generation and op["adapter_instance_id"] == adapter
                    and op["before"]["session"] == initial and op["after"]["session"] == final
                    and op["seq_after"] <= close["seq"] for op in transcript["operations"]),
                "HTML generation changed without an observed reload")
