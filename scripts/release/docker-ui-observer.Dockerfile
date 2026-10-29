ARG BASE_IMAGE=debian:trixie-slim
FROM ${BASE_IMAGE}
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates python3 python3-pil procps xvfb xauth x11-utils xdotool openbox wmctrl \
    imagemagick tesseract-ocr
