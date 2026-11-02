"""Extract operation observations from actual completed-frame receipts."""

from document_host_io import require


def operation_observation(item: dict, kind: str, name: str, saved_sha: str) -> dict:
    if kind == "office":
        reason = "single_item" if name == "single_item" else None
        return {key: item[key] for key in ("source_revision", "idle", "frame")} | {"reason": reason}
    require(kind == "html", "unknown operation kind")
    result = {"loaded_source": item["loaded_source"], "saved_input_sha256": saved_sha}
    if name in {"scroll", "reload"}:
        result.update(session=item["katana_renderer_session_epoch"], frame_generation=item["frame_generation"])
    if name == "scroll":
        result["scroll_y"] = item["scroll_y"]
    if name == "reload":
        result["frame_session"] = item["katana_frame_renderer_session_epoch"]
    return result


def matching_observation(receipt: dict, kind: str, uri_sha: str, generation: int, adapter: int | None) -> dict:
    from document_host_io import digest
    if kind == "office":
        values = receipt.get("document_observations", [])
        matches = [item for item in values if digest(item["uri"].encode()) == uri_sha
                   and item["generation"] == generation]
    else:
        values = receipt.get("html_observations", [])
        matches = [item for item in values if digest(item["source_uri"].encode()) == uri_sha
                   and item["katana_adapter_instance_id"] == adapter]
    require(len(matches) == 1, "operation receipt lacks matching actual surface")
    return matches[0]


def verify_semantics(name: str, before: dict, after: dict) -> None:
    if name in {"page", "sheet", "slide"}:
        first, second = before["frame"], after["frame"]
        require(before["idle"] is True and after["idle"] is True
                and first["item_count"] > 1 and first["item_count"] == second["item_count"]
                and first["active_index"] != second["active_index"], "Office navigation not observed")
    elif name == "single_item":
        require(before == after and before["idle"] is True and before["frame"]["item_count"] == 1
                and before["frame"]["active_index"] == 0, "single item not observed")
    elif name == "scroll":
        require(after["scroll_y"] > before["scroll_y"] and after["frame_generation"] > before["frame_generation"]
                and after["session"] == before["session"] and after["loaded_source"] == before["loaded_source"],
                "HTML scroll not observed")
    elif name == "reload":
        require(after["session"] > before["session"] and after["frame_session"] == after["session"]
                and after["loaded_source"] == before["loaded_source"], "HTML reload not observed")
    elif name == "dirty_source":
        require(before == after and before["loaded_source"].get("dirty") is True, "dirty source not retained")
    else:
        require(False, "unknown operation")
