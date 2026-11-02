#!/usr/bin/env python3
"""Record the default no-cleanup outcome before resolving a branch target."""

import argparse
from pathlib import Path

from cleanup_receipt import CleanupReceipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--controller", type=Path, required=True)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--published", required=True)
    args = parser.parse_args()
    request = {"repository": args.repository, "version": args.version,
               "published": args.published, "include_remote": False}
    with CleanupReceipt(args.controller, request) as receipt:
        receipt.record("remote", "not-requested", "explicit opt-in not supplied")
        receipt.record("local", "not-requested", "target resolution skipped")


if __name__ == "__main__":
    main()
