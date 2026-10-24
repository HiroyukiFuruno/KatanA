"""Command-line boundary for the packaged document observer."""

import argparse
from pathlib import Path


def options_from(arguments=None):
    parser = argparse.ArgumentParser()
    parser.add_argument("--evidence", required=True, type=Path)
    parser.add_argument("--expected-sha256", required=True)
    parser.add_argument("--document", required=True, type=Path)
    parser.add_argument("--format", choices=("docx", "xlsx", "pptx", "pdf"), required=True)
    parser.add_argument("--debug-log", required=True, type=Path)
    parser.add_argument("--receipt-dir", required=True, type=Path)
    parser.add_argument("--receipt-run-id", required=True)
    parser.add_argument("--prepared-settings", type=Path)
    parser.add_argument("--accept-terms", action="store_true")
    options = parser.parse_args(arguments)
    if options.accept_terms and options.prepared_settings is not None:
        parser.error("--accept-terms cannot be combined with --prepared-settings")
    return options


def main(driver_type, terms_blocked_type) -> int:
    options = options_from()
    acceptance = driver_type(
        options.evidence.resolve(), options.expected_sha256, options.document,
        options.format, options.debug_log, options.receipt_dir,
        options.receipt_run_id, options.prepared_settings, options.accept_terms,
    )
    try:
        acceptance.run()
    except terms_blocked_type:
        return 2
    except Exception as error:
        acceptance.record("failed", error=repr(error), stderr=getattr(error, "stderr", None))
        raise
    return 0
