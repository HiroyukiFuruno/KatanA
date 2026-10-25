"""Resolve the push destination and default branch from live GitHub identity."""

from pathlib import Path
import subprocess
import time
from urllib.parse import quote, urlsplit

from downstream_git_binding import _git
from issue_reference_live import LOOKUP_DEADLINE_SECONDS, response_payload, remaining_time
from issue_reference_syntax import parse_repository


def remote_repository(url):
    if url.startswith("git@github.com:"):
        repository = url.removeprefix("git@github.com:").removesuffix(".git")
    else:
        parsed = urlsplit(url)
        if parsed.scheme != "https" or parsed.netloc.casefold() != "github.com" or parsed.query or parsed.fragment:
            raise ValueError("push destination must be an explicit GitHub repository")
        repository = parsed.path.removeprefix("/").removesuffix(".git")
    parse_repository(repository)
    if any(part in {".", ".."} for part in repository.split("/")):
        raise ValueError("push destination contains an unsafe repository component")
    return repository


class PushRepositoryContext:
    def __init__(self, root: Path, repository: str, remote_url: str, *, created_refs=()):
        parse_repository(repository)
        if remote_repository(remote_url).casefold() != repository.casefold():
            raise ValueError("push destination and Issue repository do not match")
        self.root, self.repository = root.resolve(), repository
        self.deadline = time.monotonic() + LOOKUP_DEADLINE_SECONDS
        actual_root = Path(_git(self.root, ["rev-parse", "--show-toplevel"]).decode().strip()).resolve()
        if self.root != actual_root:
            raise ValueError("push source root is not the repository root")
        metadata = self._lookup(f"repos/{repository}")
        if not isinstance(metadata.get("full_name"), str) or metadata["full_name"].casefold() != repository.casefold():
            raise ValueError("GitHub repository identity does not match")
        default = metadata.get("default_branch")
        if not isinstance(default, str) or not default:
            raise ValueError("GitHub default branch is missing")
        self.default_ref = "refs/heads/" + default
        _git(self.root, ["check-ref-format", self.default_ref])
        needs_base = any(reference != self.default_ref for reference in created_refs)
        self.new_ref_base = self._lookup(f"repos/{repository}/commits/{quote(default, safe='')}").get("sha") if needs_base else None

    def _lookup(self, endpoint):
        timeout = remaining_time(self.deadline, time.monotonic())
        result = subprocess.run(["gh", "api", "--hostname", "github.com", "--method", "GET", "--include", endpoint],
                                capture_output=True, text=True, check=False, timeout=timeout)
        payload = response_payload(result.returncode, result.stdout)
        if payload is None:
            raise ValueError("GitHub push repository is missing")
        return payload
