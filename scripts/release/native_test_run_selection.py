"""Validate raw listings and the supported observed whole-run selection subset."""

import hashlib

from native_test_inventory import parse_inventory, selected_cases, validate_ignored_subset


def verified_listing(value: dict):
    raw = value.get("raw")
    if not isinstance(raw, str) or len(raw.encode()) > 16 * 1024 * 1024:
        raise ValueError("raw native listing is missing or oversized")
    if hashlib.sha256(raw.encode()).hexdigest() != value.get("raw_sha256"):
        raise ValueError("raw native listing digest does not match")
    parsed = parse_inventory(raw)
    cases = [{"name": case.name, "kind": case.kind} for case in parsed.cases]
    footer = value.get("footer")
    if (value.get("cases") != cases or not isinstance(footer, list)
            or any(type(count) is not int for count in footer)
            or footer != list(parsed.footer_counts)):
        raise ValueError("serialized native cases/footer disagree with raw listing")
    return parsed


def _selection_options(arguments: list[str]) -> tuple[str, tuple[str, ...]]:
    mode, skips, threads = "default", [], False
    iterator = iter(arguments)
    for argument in iterator:
        if argument == "--skip":
            value = next(iterator, None)
            if value is None or value.startswith("-"):
                raise ValueError("native skip value is missing or unsupported")
            skips.append(value)
        elif argument in {"--ignored", "--include-ignored"}:
            if mode != "default":
                raise ValueError("native ignored selection is duplicated or conflicting")
            mode = argument
        elif argument.startswith("--test-threads="):
            value = argument.partition("=")[2]
            if threads or not value.isascii() or not value.isdigit() or int(value) <= 0:
                raise ValueError("native test-thread selection is invalid")
            threads = True
        elif argument not in {"--nocapture", "--show-output", "-q", "--quiet"}:
            raise ValueError("unsupported native selection")
    return mode, tuple(skips)


def selected_listing(record: dict, executed_argv: list[str]) -> set[tuple[str, str]]:
    all_cases = verified_listing(record["all"])
    ignored = verified_listing(record["ignored"])
    validate_ignored_subset(all_cases, ignored)
    boundary = executed_argv.index("--") if "--" in executed_argv else len(executed_argv)
    mode, skips = _selection_options(executed_argv[boundary + 1:])
    if any(case.kind != "test" for case in all_cases.cases):
        raise ValueError("benchmark execution selection is not established")
    ignored_set = set(ignored.cases)
    candidates = selected_cases(all_cases, skips)
    if mode == "--ignored":
        candidates = tuple(case for case in candidates if case in ignored_set)
    elif mode == "default":
        candidates = tuple(case for case in candidates if case not in ignored_set)
    return {(case.name, case.kind) for case in candidates}
