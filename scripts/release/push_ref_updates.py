"""Parse native pre-push ref tuples without treating them as admission proof."""

from dataclasses import dataclass
import re


MAX_PUSH_INPUT_BYTES = 1_048_576
OID_RE = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")


class PushInputError(ValueError):
    """Native push metadata cannot be interpreted without ambiguity."""


@dataclass(frozen=True)
class PushRefUpdate:
    local_ref: str
    local_oid: str
    remote_ref: str
    remote_oid: str

    @property
    def deleted(self) -> bool:
        return self.local_oid == "0" * len(self.local_oid)

    @property
    def created(self) -> bool:
        return self.remote_oid == "0" * len(self.remote_oid)


def _update(line: str) -> PushRefUpdate:
    fields = line.split(" ")
    if len(fields) != 4:
        raise PushInputError("push update must have four fields")
    update = PushRefUpdate(*fields)
    if not update.local_ref:
        raise PushInputError("push source reference is empty")
    if any(ord(character) < 32 or ord(character) == 127
           for reference in (update.local_ref, update.remote_ref) for character in reference):
        raise PushInputError("push references contain control characters")
    if not OID_RE.fullmatch(update.local_oid) or not OID_RE.fullmatch(update.remote_oid):
        raise PushInputError("push object identities must be full lowercase OIDs")
    if len(update.local_oid) != len(update.remote_oid):
        raise PushInputError("push object formats do not match")
    if not update.remote_ref.startswith(("refs/heads/", "refs/tags/")):
        raise PushInputError("push destination must be a branch or tag reference")
    if update.remote_ref in ("refs/heads/", "refs/tags/"):
        raise PushInputError("push destination reference is empty")
    if update.deleted != (update.local_ref == "(delete)"):
        raise PushInputError("deletion ref and object identity disagree")
    if update.deleted and update.created:
        raise PushInputError("push update has no source or destination object")
    return update


def parse_push_updates(raw: bytes) -> tuple[PushRefUpdate, ...]:
    """Return metadata only; empty input does not prove any commit was verified."""
    if len(raw) > MAX_PUSH_INPUT_BYTES:
        raise PushInputError("push input exceeds the size limit")
    try:
        text = raw.decode("utf-8")
    except UnicodeError as error:
        raise PushInputError("push input is not UTF-8") from error
    # GitのASCII区切りだけを使い、有効なUnicode ref内の空白・改行風文字を壊さない。
    lines = (line.removesuffix("\r") for line in text.split("\n"))
    updates = tuple(_update(line) for line in lines if line)
    if len({update.remote_ref for update in updates}) != len(updates):
        raise PushInputError("push destination is repeated")
    if len({len(update.local_oid) for update in updates}) > 1:
        raise PushInputError("push input mixes object formats")
    return updates
