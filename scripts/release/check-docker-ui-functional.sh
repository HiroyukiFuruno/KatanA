#!/usr/bin/env bash
set -euo pipefail

DOCUMENT_MODE=false
DOCUMENT_SOURCE=
DOCUMENT_FORMAT=
PREPARED_CONFIG=
ACCEPT_TERMS=false
DRIVER_TERMS_ARGS=()
if [[ ${1:-} == --document ]]; then
    DOCUMENT_MODE=true
    DOCUMENT_SOURCE=${2:?--document requires a source file}
    DOCUMENT_FORMAT=${3:?--document requires docx, xlsx, pptx, or pdf}
    shift 3
    if [[ ${1:-} == --prepared-config ]]; then
        PREPARED_CONFIG=${2:?--prepared-config requires an isolated directory}
        shift 2
    fi
    if [[ ${1:-} == --accept-terms ]]; then
        ACCEPT_TERMS=true
        shift
    fi
    if [[ "$ACCEPT_TERMS" == true && -n "$PREPARED_CONFIG" ]]; then
        echo "FAIL: --accept-terms requires the fresh isolated config" >&2
        exit 2
    fi
fi
ARCHIVE=${1:?Usage: check-docker-ui-functional.sh [--document source docx|xlsx|pptx|pdf [--prepared-config dir] [--accept-terms]] archive observer-image app-image}
OBSERVER_IMAGE=${2:?observer image is required}
APP_IMAGE=${3:?application runtime image is required}
ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
ARCHIVE=$(cd "$(dirname "$ARCHIVE")" && printf '%s/%s' "$PWD" "$(basename "$ARCHIVE")")
[[ -f "$ARCHIVE" ]] || { echo "FAIL: archive missing" >&2; exit 1; }
RUN_DIR=$(mktemp -d "$ROOT_DIR/tmp/docker-ui-functional.XXXXXX")
RUN_ID=$(basename "$RUN_DIR")
OBSERVER="katana-${RUN_ID}-observer"
APP="katana-${RUN_ID}-app"
X11_VOLUME="katana-${RUN_ID}-x11"
EVIDENCE_VOLUME="katana-${RUN_ID}-evidence"
create_owned_volume() {
    local name=$1 purpose=$2 labels
    docker volume create \
        --label "io.katana.owner=HiroyukiFuruno/KatanA" \
        --label "io.katana.purpose=$purpose" \
        --label "io.katana.run-id=$RUN_ID" \
        "$name" >/dev/null
    labels=$(docker volume inspect -f '{{json .Labels}}' "$name")
    python3 - "$labels" "$purpose" "$RUN_ID" <<'PY'
import json
import sys
labels = json.loads(sys.argv[1] or "{}")
expected = {
    "io.katana.owner": "HiroyukiFuruno/KatanA",
    "io.katana.purpose": sys.argv[2],
    "io.katana.run-id": sys.argv[3],
}
if any(labels.get(key) != value for key, value in expected.items()):
    raise SystemExit("Docker volume name already exists without the expected owner/purpose labels")
PY
}
echo "RUN_DIR=$RUN_DIR"
docker image inspect "$OBSERVER_IMAGE" "$APP_IMAGE" >"$RUN_DIR/images.json"
[[ "$(docker image inspect -f '{{.Architecture}}' "$APP_IMAGE")" == amd64 ]] || {
    echo "FAIL: application runtime image is not amd64" >&2
    exit 1
}
python3 "$ROOT_DIR/scripts/release/verify-binary-architecture.py" linux "$ARCHIVE" \
    | tee "$RUN_DIR/architecture.log"
docker info --format '{{.OSType}}/{{.Architecture}}' >"$RUN_DIR/host.txt"
shasum -a 256 "$ARCHIVE" >"$RUN_DIR/archive.sha256"
EXPECTED_SHA256=$(tar -xOf "$ARCHIVE" KatanA | shasum -a 256 | awk '{print $1}')
OBSERVER_DOCUMENT_MOUNTS=()
APP_DOCUMENT_MOUNTS=()
DRIVER_PREPARED_ARGS=()
APP_RECEIPT_ENV=()
APP_CONFIG_DIR=/evidence/config
RECEIPT_DIR=/evidence/resource-receipt
if [[ "$DOCUMENT_MODE" == true ]]; then
    DOCUMENT_SOURCE=$(cd "$(dirname "$DOCUMENT_SOURCE")" && printf '%s/%s' "$PWD" "$(basename "$DOCUMENT_SOURCE")")
    [[ -f "$DOCUMENT_SOURCE" ]] || { echo "FAIL: document source missing" >&2; exit 1; }
    case "$DOCUMENT_FORMAT" in docx|xlsx|pptx|pdf) ;; *) echo "FAIL: document format must be docx, xlsx, pptx, or pdf" >&2; exit 1;; esac
    DOCUMENT_NAME=$(basename "$DOCUMENT_SOURCE")
    DOCUMENT_PATH="/fixture/$DOCUMENT_NAME"
    OBSERVER_DOCUMENT_MOUNTS+=(--mount "type=bind,src=$DOCUMENT_SOURCE,dst=$DOCUMENT_PATH,readonly")
    APP_DOCUMENT_MOUNTS+=(--mount "type=bind,src=$DOCUMENT_SOURCE,dst=$DOCUMENT_PATH,readonly")
    if [[ -n "$PREPARED_CONFIG" ]]; then
        PREPARED_CONFIG=$(cd "$PREPARED_CONFIG" && pwd)
        [[ -f "$PREPARED_CONFIG/settings.json" ]] || { echo "FAIL: prepared settings missing" >&2; exit 1; }
        python3 "$ROOT_DIR/scripts/release/docker-ui-document-log-parser.py" \
            --validate-prepared-config "$PREPARED_CONFIG/settings.json" --document "$DOCUMENT_PATH" \
            >"$RUN_DIR/prepared-terms-version.before.txt"
        shasum -a 256 "$PREPARED_CONFIG/settings.json" >"$RUN_DIR/prepared-settings.before.sha256"
        cp "$PREPARED_CONFIG/settings.json" "$RUN_DIR/prepared-settings.before.json"
        OBSERVER_DOCUMENT_MOUNTS+=(--mount "type=bind,src=$PREPARED_CONFIG,dst=/prepared-config,readonly")
        APP_DOCUMENT_MOUNTS+=(--mount "type=bind,src=$PREPARED_CONFIG,dst=/prepared-config")
        DRIVER_PREPARED_ARGS=(--prepared-settings /prepared-config/settings.json)
        APP_CONFIG_DIR=/prepared-config
    fi
fi

cleanup() {
    local status=$?
    trap - EXIT
    set +e
    python3 "$ROOT_DIR/scripts/docker/ui-functional-cleanup.py" \
        --root "$ROOT_DIR" --run-dir "$RUN_DIR" --run-id "$RUN_ID" \
        --app-name "$APP" --observer-name "$OBSERVER" \
        --app-id-file "$RUN_DIR/app.id" --observer-id-file "$RUN_DIR/observer.id" \
        --x11-volume "$X11_VOLUME" --evidence-volume "$EVIDENCE_VOLUME" \
        --initial-status "$status"
    exit $?
}
trap cleanup EXIT

create_owned_volume "$X11_VOLUME" ui-x11
create_owned_volume "$EVIDENCE_VOLUME" ui-evidence

docker run -d --name "$OBSERVER" --label "io.katana.owner=HiroyukiFuruno/KatanA" \
    --label "io.katana.run-id=$RUN_ID" --cpus 1 --memory 1g \
    --mount "type=volume,src=$X11_VOLUME,dst=/tmp/.X11-unix" \
    --mount "type=volume,src=$EVIDENCE_VOLUME,dst=/evidence" \
    --mount "type=bind,src=$ROOT_DIR,dst=/app,readonly" --env DISPLAY=:99 \
    --env OMP_THREAD_LIMIT=1 \
    "${OBSERVER_DOCUMENT_MOUNTS[@]}" \
    "$OBSERVER_IMAGE" bash -c '
        set -e
        Xvfb :99 -screen 0 1280x900x24 -nolisten tcp -ac &
        for attempt in {1..50}; do
            if xdpyinfo >/dev/null 2>&1; then exec openbox; fi
            sleep 0.1
        done
        exit 1
    ' >"$RUN_DIR/observer.id"
if [[ "$DOCUMENT_MODE" == true && -z "$PREPARED_CONFIG" ]]; then
    docker exec "$OBSERVER" mkdir -p /evidence/config
    docker exec "$OBSERVER" python3 /app/scripts/release/docker-ui-document-log-parser.py \
        --write-fresh-config /evidence/config/settings.json /evidence/config/workspace.json \
        --document "$DOCUMENT_PATH"
    docker exec "$OBSERVER" python3 /app/scripts/release/docker-ui-document-log-parser.py \
        --validate-fresh-config /evidence/config/settings.json --document "$DOCUMENT_PATH" \
        >"$RUN_DIR/fresh-config.validation.txt"
    docker exec "$OBSERVER" sha256sum /evidence/config/settings.json \
        >"$RUN_DIR/fresh-config.prelaunch.sha256"
    docker exec "$OBSERVER" cp /evidence/config/settings.json /evidence/fresh-config.prelaunch.json
    printf '%s\n' "fresh-config-strict-validation=passed" >"$RUN_DIR/fresh-config.prelaunch.marker"
fi
if [[ "$ACCEPT_TERMS" == true ]]; then
    DRIVER_TERMS_ARGS=(--accept-terms)
fi
if [[ "$DOCUMENT_MODE" == true ]]; then
    docker exec "$OBSERVER" mkdir -p "$RECEIPT_DIR"
    APP_RECEIPT_ENV=(--env "KATANA_DOCUMENT_RESOURCE_RECEIPT_DIR=$RECEIPT_DIR" --env "KATANA_DOCUMENT_RESOURCE_RUN_ID=$RUN_ID")
fi
docker run -d --name "$APP" --label "io.katana.owner=HiroyukiFuruno/KatanA" \
    --label "io.katana.run-id=$RUN_ID" --platform linux/amd64 --pid="container:$OBSERVER" \
    --cpus 3 --memory 3g \
    --mount "type=volume,src=$X11_VOLUME,dst=/tmp/.X11-unix" \
    --mount "type=volume,src=$EVIDENCE_VOLUME,dst=/evidence" \
    --mount "type=bind,src=$ARCHIVE,dst=/run/candidate.tar.gz,readonly" \
    "${APP_DOCUMENT_MOUNTS[@]}" \
    --env DISPLAY=:99 --env LIBGL_ALWAYS_SOFTWARE=1 --env LANG=C.UTF-8 \
    --env KATANA_CONFIG_DIR="$APP_CONFIG_DIR" --env DOCUMENT_MODE="$DOCUMENT_MODE" \
    --env DOCUMENT_PATH="${DOCUMENT_PATH:-}" \
    "${APP_RECEIPT_ENV[@]}" \
    --env KATANA_STARTUP_HEARTBEAT=/evidence/heartbeat.txt --env DEBUG=true \
    "$APP_IMAGE" bash -c '
        set -e
        [[ "$(uname -m)" == x86_64 ]]
        printf "APP_RUNTIME_ARCH=%s\n" "$(uname -m)"
        mkdir /evidence/package
        tar -xzf /run/candidate.tar.gz -C /evidence/package
        sha256sum /run/candidate.tar.gz /evidence/package/KatanA
        if [[ "${DOCUMENT_MODE:-false}" == true ]]; then
            : > /evidence/application-debug.log
            exec > >(tee /evidence/application-debug.log) 2>&1
        fi
        printf "%s\n" "$$" > /evidence/app.pid
        exec /evidence/package/KatanA
    ' >"$RUN_DIR/app.id"
if [[ "$DOCUMENT_MODE" == true ]]; then
    set +e
    docker exec "$OBSERVER" python3 /app/scripts/release/docker-ui-document-functional.py \
        --evidence /evidence --expected-sha256 "$EXPECTED_SHA256" \
        --document "$DOCUMENT_PATH" --format "$DOCUMENT_FORMAT" \
        --debug-log /evidence/application-debug.log \
        --receipt-dir "$RECEIPT_DIR" --receipt-run-id "$RUN_ID" \
        "${DRIVER_TERMS_ARGS[@]}" \
        "${DRIVER_PREPARED_ARGS[@]}" \
        | tee "$RUN_DIR/document-functional.log"
    DRIVER_EXIT=${PIPESTATUS[0]}
    set -e
else
    docker exec "$OBSERVER" python3 /app/scripts/release/docker-ui-functional.py \
        --evidence /evidence --expected-sha256 "$EXPECTED_SHA256" | tee "$RUN_DIR/functional.log"
fi
for _ in {1..300}; do
    if [[ "$(docker inspect -f '{{.State.Running}}' "$APP")" == false ]]; then
        break
    fi
    sleep 0.1
done
[[ "$(docker inspect -f '{{.State.Running}}' "$APP")" == false ]] || {
    echo "FAIL: application did not exit after WM close" >&2
    exit 1
}
APP_EXIT=$(docker wait "$APP")
printf '%s\n' "$APP_EXIT" >"$RUN_DIR/application-exit.txt"
[[ "$APP_EXIT" == 0 ]] || { echo "FAIL: application exit=$APP_EXIT" >&2; exit 1; }
shasum -a 256 -c "$RUN_DIR/archive.sha256"
if [[ "$DOCUMENT_MODE" == true && -n "$PREPARED_CONFIG" ]]; then
    shasum -a 256 "$PREPARED_CONFIG/settings.json" >"$RUN_DIR/prepared-settings.after.sha256"
    cp "$PREPARED_CONFIG/settings.json" "$RUN_DIR/prepared-settings.after.json"
    python3 "$ROOT_DIR/scripts/release/docker-ui-document-log-parser.py" \
        --read-terms-version "$PREPARED_CONFIG/settings.json" \
        >"$RUN_DIR/prepared-terms-version.after.txt"
    cmp "$RUN_DIR/prepared-terms-version.before.txt" "$RUN_DIR/prepared-terms-version.after.txt" \
        || { echo "FAIL: Terms version changed" >&2; exit 1; }
fi
if [[ "$DOCUMENT_MODE" == true ]]; then
    if [[ "${DRIVER_EXIT:-1}" == 2 ]]; then
        echo "BLOCKED: real Terms modal requires human action" >&2
        exit 2
    fi
    [[ "${DRIVER_EXIT:-1}" == 0 ]] || {
        echo "FAIL: document driver exit=${DRIVER_EXIT:-1}" >&2
        exit 1
    }
    echo "PARTIAL: packaged document UI mechanics and raw resource receipt observed; performance/coverage not asserted"
    exit 0
fi
echo "PASS: Docker functional UI and normal exit; native_performance=false"
