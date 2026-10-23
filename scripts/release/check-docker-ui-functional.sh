#!/usr/bin/env bash
set -euo pipefail

ARCHIVE=${1:?Usage: check-docker-ui-functional.sh archive observer-image app-image}
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

remove_owned_resources() {
    local id_file container_id
    for id_file in "$RUN_DIR/app.id" "$RUN_DIR/observer.id"; do
        [[ -s "$id_file" ]] || continue
        container_id=$(tr -d '\n' <"$id_file")
        [[ "$container_id" =~ ^[a-f0-9]{64}$ ]] || return 1
        docker rm "$container_id" >/dev/null || return 1
    done
    docker volume rm "$X11_VOLUME" "$EVIDENCE_VOLUME" >"$RUN_DIR/removed-volumes.txt"
}

cleanup() {
    local status=$?
    trap - EXIT
    docker logs "$APP" >"$RUN_DIR/application.log" 2>&1 || true
    docker inspect "$OBSERVER" "$APP" >"$RUN_DIR/containers.json" 2>/dev/null || true
    mkdir -p "$RUN_DIR/evidence"
    if ! docker exec "$OBSERVER" tar --exclude='./package' -C /evidence -cf - . \
        | tar -xf - -C "$RUN_DIR/evidence"; then
        echo "FAIL: functional evidence export failed" >&2
        status=1
    fi
    if [[ "$(docker inspect -f '{{.State.Running}}' "$APP" 2>/dev/null)" == true ]]; then
        echo "Forced cleanup, not normal exit" >"$RUN_DIR/forced-cleanup.txt"
        status=1
        docker stop -t 5 "$APP" >/dev/null || true
    fi
    docker stop -t 5 "$OBSERVER" >/dev/null 2>&1 || true
    if [[ "$status" == 0 ]] && ! remove_owned_resources; then
        echo "FAIL: owned Docker resource cleanup failed" >&2
        status=1
    fi
    printf '%s\n' "$status" >"$RUN_DIR/driver-exit.txt"
    echo "Functional evidence retained: $RUN_DIR"
    exit "$status"
}
trap cleanup EXIT

docker run -d --name "$OBSERVER" --cpus 1 --memory 1g \
    --mount "type=volume,src=$X11_VOLUME,dst=/tmp/.X11-unix" \
    --mount "type=volume,src=$EVIDENCE_VOLUME,dst=/evidence" \
    --mount "type=bind,src=$ROOT_DIR,dst=/app,readonly" --env DISPLAY=:99 \
    "$OBSERVER_IMAGE" bash -c '
        set -e
        Xvfb :99 -screen 0 1280x900x24 -nolisten tcp -ac &
        for attempt in {1..50}; do
            if xdpyinfo >/dev/null 2>&1; then exec openbox; fi
            sleep 0.1
        done
        exit 1
    ' >"$RUN_DIR/observer.id"
docker run -d --name "$APP" --platform linux/amd64 --pid="container:$OBSERVER" \
    --cpus 3 --memory 3g \
    --mount "type=volume,src=$X11_VOLUME,dst=/tmp/.X11-unix" \
    --mount "type=volume,src=$EVIDENCE_VOLUME,dst=/evidence" \
    --mount "type=bind,src=$ARCHIVE,dst=/run/candidate.tar.gz,readonly" \
    --env DISPLAY=:99 --env LIBGL_ALWAYS_SOFTWARE=1 --env LANG=C.UTF-8 \
    --env KATANA_CONFIG_DIR=/evidence/config \
    --env KATANA_STARTUP_HEARTBEAT=/evidence/heartbeat.txt --env DEBUG=true \
    "$APP_IMAGE" bash -c '
        set -e
        [[ "$(uname -m)" == x86_64 ]]
        printf "APP_RUNTIME_ARCH=%s\n" "$(uname -m)"
        mkdir /evidence/package
        tar -xzf /run/candidate.tar.gz -C /evidence/package
        sha256sum /run/candidate.tar.gz /evidence/package/KatanA
        printf "%s\n" "$$" > /evidence/app.pid
        exec /evidence/package/KatanA
    ' >"$RUN_DIR/app.id"
docker exec "$OBSERVER" python3 /app/scripts/release/docker-ui-functional.py \
    --evidence /evidence --expected-sha256 "$EXPECTED_SHA256" | tee "$RUN_DIR/functional.log"
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
echo "PASS: Docker functional UI and normal exit; native_performance=false"
