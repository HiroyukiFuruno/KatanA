"""Keep published read-only HTML QA distinct from editable-source fixtures."""

from document_host_io import require


def required_html_operations(publication_version: str, phase: str) -> frozenset[str]:
    require(isinstance(publication_version, str) and isinstance(phase, str),
            "HTML QA version and phase must be strings")
    require((publication_version, phase) in {
        ("v0.22.42", "prepublish"), ("v0.22.42", "postpublication"),
        ("v0.22.47", "postpublication"),
    }, "unsupported HTML QA version/phase")
    if publication_version == "v0.22.47":
        return frozenset({"scroll", "reload"})
    return frozenset({"scroll", "reload", "dirty_source"})


def require_readonly_source(operation: dict) -> None:
    require(operation.get("name") in {"scroll", "reload"},
            "published read-only HTML cannot claim an editing operation")
    for boundary in ("before", "after"):
        observation = operation.get(boundary)
        require(isinstance(observation, dict), "read-only HTML operation boundary missing")
        source = observation.get("loaded_source")
        require(isinstance(source, dict) and source.get("dirty") is False,
                "published read-only HTML source must stay clean")
        require(isinstance(source.get("preview_input_fnv1a64"), str)
                and source["preview_input_fnv1a64"] == source.get("last_imported_disk_fnv1a64"),
                "published read-only HTML source differs from disk input")


def validate_html_operations(operations: list[dict], *, publication_version: str, phase: str) -> None:
    required = required_html_operations(publication_version, phase)
    require(isinstance(operations, list) and all(isinstance(item, dict) for item in operations),
            "HTML QA operation list required")
    html = [item for item in operations if item.get("kind") == "html"]
    require(all(isinstance(item.get("name"), str) for item in html), "HTML operation name required")
    require(required.issubset({item.get("name") for item in html}),
            "required HTML operations missing: " + ", ".join(sorted(required)))
    if publication_version == "v0.22.47":
        for operation in html:
            require_readonly_source(operation)
