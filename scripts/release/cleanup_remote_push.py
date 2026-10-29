"""Bind a cleanup record to this push's native advertised deletion tuple."""

from dataclasses import dataclass
from pathlib import Path

from cleanup_intent import REMOTE_DELETE, read_intent
from cleanup_merge_binding import require_merge_binding
from cleanup_target import release_branch_version
from push_ref_updates import PushInputError, parse_push_updates
from release_cleanup_audit import CleanupRefused, audit_branch, common_directory, git, require_controller
from release_cleanup_publication import cleanup_destination


@dataclass(frozen=True)
class RemoteCleanupPush:
    root: Path
    repository: str
    default_ref: str
    remote_name: str
    remote_url: str

    def _record(self, path: Path, digest: str) -> dict[str, object]:
        value = read_intent(path, digest, operation=REMOTE_DELETE)
        if (str(self.root.resolve()) != value["root"]
                or str(common_directory(self.root)) != value["common_git_dir"]
                or self.repository.casefold() != value["repository"].casefold()):
            raise CleanupRefused("remote cleanup intent belongs to another repository")
        return value

    def _destination(self) -> None:
        if self.remote_name != "origin":
            raise CleanupRefused("remote cleanup must use the audited origin")
        cleanup_destination(self.root, self.root, self.repository)
        actual = git(self.root, "remote", "get-url", "--push", "--all", "origin").decode().splitlines()
        if actual != [self.remote_url]:
            raise CleanupRefused("remote cleanup push URL changed")

    def _merge_binding(self, value: dict[str, object]) -> str | None:
        merge_commit, pull_request = value.get("merge_commit"), value.get("pull_request")
        if merge_commit is None and pull_request is None:
            return None
        version = release_branch_version(value["branch"])
        if version is None:
            raise CleanupRefused("remote cleanup merge binding requires a versioned release branch")
        return require_merge_binding(
            self.repository, version, merge_commit or value["expected"],
            value["branch"], value["expected"], merge_commit,
            int(pull_request) if pull_request is not None else None)

    def _audit(self, value: dict[str, object]) -> None:
        if not self.default_ref.startswith("refs/heads/"):
            raise CleanupRefused("remote cleanup default is not a branch")
        default = self.default_ref.removeprefix("refs/heads/")
        require_controller(self.root, self.root, default)
        merge_commit = self._merge_binding(value)
        record = audit_branch(self.root, default, value["branch"], value["expected"],
                              allow_absent=True, integrated_oid=merge_commit)
        if record["state"] != "eligible":
            raise CleanupRefused("remote cleanup must retain this target: " + record["reason"])

    def verify(self, payload: bytes, path: Path | None, digest: str | None) -> None:
        if path is None and digest is None:
            return
        if path is None or digest is None:
            raise CleanupRefused("remote cleanup requires both intent path and digest")
        value = self._record(path, digest)
        try:
            updates = parse_push_updates(payload)
        except PushInputError as error:
            raise CleanupRefused("remote cleanup metadata is invalid") from error
        if len(updates) != 1:
            raise CleanupRefused("remote cleanup requires exactly one native deletion")
        update = updates[0]
        if (not update.deleted or update.remote_ref != "refs/heads/" + value["branch"]
                or update.remote_oid != value["expected"]):
            raise CleanupRefused("remote cleanup differs from the advertised deletion")
        self._destination()
        self._audit(value)
