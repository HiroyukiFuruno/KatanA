"""Create a single compare-and-swap version commit through GitHub's verified API."""
from __future__ import annotations

import base64
import json
from pathlib import Path
import subprocess
import sys

from manual_bump_transition import COMMIT, FILES, expected_files

QUERY = '''mutation($input: CreateCommitOnBranchInput!) {
  createCommitOnBranch(input: $input) { commit { oid } }
}'''


def build_request(root: Path, trigger: str, version: str) -> dict:
    if not COMMIT.fullmatch(trigger):
        raise ValueError("expected head must be a full commit")
    expected = expected_files(root, trigger, version)
    additions = []
    for path in FILES:
        file = root / path
        if file.is_symlink() or not file.is_file() or file.read_text() != expected[path]:
            raise ValueError("version file does not match the exact validated change")
        additions.append({"path": path, "contents": base64.b64encode(file.read_bytes()).decode("ascii")})
    return {"query": QUERY, "variables": {"input": {
        "branch": {"repositoryNameWithOwner": "HiroyukiFuruno/KatanA", "branchName": "master"},
        "expectedHeadOid": trigger, "message": {"headline": f"chore: {version}のリリース準備"},
        "fileChanges": {"additions": additions},
    }}}


def parse_response(response: object) -> str:
    if not isinstance(response, dict) or response.get("errors"):
        raise ValueError("GitHub rejected the atomic version commit")
    try:
        oid = response["data"]["createCommitOnBranch"]["commit"]["oid"]
    except (KeyError, TypeError) as error:
        raise ValueError("atomic version commit response is incomplete") from error
    if not isinstance(oid, str) or not COMMIT.fullmatch(oid):
        raise ValueError("atomic version commit ID is invalid")
    return oid


def main() -> None:
    trigger, version = sys.argv[1:]
    request = build_request(Path.cwd(), trigger, version)
    result = subprocess.run(["gh", "api", "graphql", "--input", "-"],
                            input=json.dumps(request), text=True, capture_output=True, timeout=120)
    if result.returncode:
        raise SystemExit("GitHub rejected the atomic version commit")
    print(parse_response(json.loads(result.stdout)))


if __name__ == "__main__":
    main()
