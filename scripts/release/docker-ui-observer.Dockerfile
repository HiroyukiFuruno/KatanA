ARG BASE_IMAGE=debian:trixie-slim
FROM ${BASE_IMAGE}
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates python3 python3-venv procps xvfb xauth x11-utils xdotool openbox wmctrl \
    imagemagick tesseract-ocr
COPY scripts/release/evidence-requirements.txt /tmp/evidence-requirements.txt
RUN python3 -m venv /opt/katana-pip \
    && /opt/katana-pip/bin/python -m pip install --disable-pip-version-check \
        --no-cache-dir --only-binary=:all: -r /tmp/evidence-requirements.txt \
    && rm /tmp/evidence-requirements.txt
ENV PATH="/opt/katana-pip/bin:${PATH}"
