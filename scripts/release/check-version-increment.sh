#!/bin/bash
set -euo pipefail

error() { printf '[ERROR] %s\n' "$*" >&2; }
success() { printf '[OK]    %s\n' "$*"; }

TARGET_VERSION=${1:-}
CHANGELOG_PATH=${2:-CHANGELOG.md}
LATEST_RELEASE_VERSION=${3:-}

if [ -z "$TARGET_VERSION" ]; then
    error "Target version is required."
    exit 1
fi

TARGET_VERSION="${TARGET_VERSION#v}"

if [[ ! "$TARGET_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    error "Target version must be SemVer x.y.z: ${TARGET_VERSION}."
    exit 1
fi

if [ -z "$LATEST_RELEASE_VERSION" ]; then
    if ! command -v gh >/dev/null 2>&1; then
        error "gh is required to resolve the latest published GitHub Release."
        exit 1
    fi

    if ! LATEST_RELEASE_VERSION=$(gh release view --json tagName --jq '.tagName'); then
        error "Failed to resolve the latest published GitHub Release."
        exit 1
    fi
fi

LATEST_RELEASE_VERSION="${LATEST_RELEASE_VERSION#v}"

if [[ ! "$LATEST_RELEASE_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    error "Latest published release must be SemVer x.y.z: ${LATEST_RELEASE_VERSION}."
    exit 1
fi

if [ ! -r "$CHANGELOG_PATH" ]; then
    error "Changelog not found: $CHANGELOG_PATH"
    exit 1
fi

versions=()
while IFS= read -r version; do
    versions+=("$version")
    if [ "${#versions[@]}" -ge 2 ]; then
        break
    fi
done < <(sed -nE 's/^## \[([0-9]+\.[0-9]+\.[0-9]+)\].*/\1/p' "$CHANGELOG_PATH")

if [ "${#versions[@]}" -lt 2 ]; then
    error "CHANGELOG.md must contain the target release and one previous release heading."
    exit 1
fi

changelog_target="${versions[0]}"
base_version="${versions[1]}"

if [ "$changelog_target" != "$TARGET_VERSION" ]; then
    error "Latest CHANGELOG.md release (v${changelog_target}) does not match target version (v${TARGET_VERSION})."
    exit 1
fi

if [ "$base_version" != "$LATEST_RELEASE_VERSION" ]; then
    error "Previous CHANGELOG.md release (v${base_version}) does not match latest published release (v${LATEST_RELEASE_VERSION})."
    exit 1
fi

IFS=. read -r base_major base_minor base_patch <<<"$LATEST_RELEASE_VERSION"

allowed_patch="${base_major}.${base_minor}.$((base_patch + 1))"

if [ "$TARGET_VERSION" = "$allowed_patch" ]; then
    success "Version increment is valid: v${LATEST_RELEASE_VERSION} -> v${TARGET_VERSION}."
    exit 0
fi

error "Version increment is invalid: v${LATEST_RELEASE_VERSION} -> v${TARGET_VERSION}."
error "The only allowed next version is v${allowed_patch}."
exit 1
