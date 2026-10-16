"""Validate cargo-llvm-cov show-env output without executing shell code."""

import shlex


MAX_OUTPUT_BYTES = 64 * 1024
KNOWN_KEYS = frozenset({
    "LLVM_PROFILE_FILE", "RUSTFLAGS", "RUSTDOCFLAGS", "RUSTC_WRAPPER",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER", "__CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER_COVERAGE_TARGET",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER_HOST", "__CARGO_LLVM_COV_RUSTC_WRAPPER_PRE_EXISTING",
    "CARGO_LLVM_COV", "CARGO_LLVM_COV_SHOW_ENV", "CARGO_LLVM_COV_TARGET_DIR",
    "CARGO_LLVM_COV_BUILD_DIR",
})
REQUIRED_KEYS = frozenset({
    "CARGO_LLVM_COV", "CARGO_LLVM_COV_SHOW_ENV", "LLVM_PROFILE_FILE",
    "CARGO_LLVM_COV_TARGET_DIR", "CARGO_LLVM_COV_BUILD_DIR",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER", "__CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS",
    "__CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES", "RUSTC_WRAPPER",
})


def _assignment(parts: list[str]) -> tuple[str, str]:
    if len(parts) != 2 or parts[0] != "export" or "=" not in parts[1]:
        raise ValueError("show-env contains a malformed export")
    key, value = parts[1].split("=", 1)
    if key not in KNOWN_KEYS or not key.isidentifier():
        raise ValueError("show-env contains an unknown key")
    return key, value


def _require_contract(environment: dict[str, str]) -> None:
    required = ("CARGO_LLVM_COV", "CARGO_LLVM_COV_SHOW_ENV", "LLVM_PROFILE_FILE",
                "CARGO_LLVM_COV_TARGET_DIR", "CARGO_LLVM_COV_BUILD_DIR")
    if any(not environment.get(key) for key in required) or any(
            environment.get(key) != "1" for key in required[:2]):
        raise ValueError("show-env lacks the coverage instrumentation contract")
    marker = "__CARGO_LLVM_COV_RUSTC_WRAPPER"
    flags = environment.get(f"{marker}_RUSTFLAGS", "").split("\x1f")
    instrumentation = [flag for index, flag in enumerate(flags)
                       if index > 0 and flags[index - 1] == "-C"
                       and flag.startswith("instrument-coverage")]
    if (environment.get(marker) != "1" or not environment.get("RUSTC_WRAPPER")
            or not environment.get(f"{marker}_CRATE_NAMES")
            or instrumentation != ["instrument-coverage"]
            or any(flag.startswith("-Cinstrument-coverage") for flag in flags)):
        raise ValueError("show-env lacks active wrapper metadata")


def _tokens(raw: bytes) -> list[str]:
    if not isinstance(raw, bytes) or len(raw) > MAX_OUTPUT_BYTES:
        raise ValueError("show-env output exceeds the limit")
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ValueError("show-env output is not UTF-8") from error
    if "\x00" in text:
        raise ValueError("show-env output contains NUL")
    try:
        lexer = shlex.shlex(text, posix=True, punctuation_chars=";&|()<>`")
        lexer.whitespace_split = True
        lexer.commenters = ""
        return list(lexer)
    except ValueError as error:
        raise ValueError("show-env contains invalid shell quoting") from error


def _apply_tokens(tokens: list[str], parent: dict[str, str]) -> tuple[dict[str, str], set[str]]:
    environment, seen = dict(parent), set()
    index = 0
    while index < len(tokens):
        if tokens[index] == "unset" and index + 1 < len(tokens):
            operation, key, value = "unset", tokens[index + 1], None
            index += 2
        else:
            operation, key, value = "set", "", None
            if index + 1 >= len(tokens):
                raise ValueError("show-env contains a truncated operation")
            key, value = _assignment(tokens[index:index + 2])
            index += 2
        if key not in KNOWN_KEYS:
            raise ValueError("show-env contains an unknown key")
        if key in seen:
            raise ValueError("show-env assigns a key more than once")
        seen.add(key)
        if operation == "unset":
            environment.pop(key, None)
        else:
            environment[key] = value or ""
    return environment, seen


def parse_show_environment(raw: bytes, parent: dict[str, str]) -> dict[str, str]:
    environment, seen = _apply_tokens(_tokens(raw), parent)
    if not REQUIRED_KEYS <= seen:
        raise ValueError("show-env omits required instrumentation keys")
    _require_contract(environment)
    return environment
