#!/bin/bash

# Script to download an icon into all vendor packs

set -e

function usage() {
    echo "Usage: $0"
    echo "  --category <dir>             e.g. 'system', 'navigation', 'action'"
    echo "  --name <icon_name>           e.g. 'tools', 'toc'"
    echo "  [--feather <name>]           Fallback to 'name' for feather (default: same as --name)"
    echo "  [--heroicons <name>]         Fallback to 'name' for heroicons"
    echo "  [--lucide <name>]            Fallback to 'name' for lucide"
    echo "  [--material <name>]          Fallback to 'name' for material-symbols"
    echo "  [--tabler <name>]            Fallback to 'name' for tabler-icons"
    exit 1
}

CATEGORY=""
NAME=""
FEATHER=""
HEROICONS=""
LUCIDE=""
MATERIAL=""
TABLER=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --category) CATEGORY="$2"; shift 2 ;;
    --name) NAME="$2"; shift 2 ;;
    --feather) FEATHER="$2"; shift 2 ;;
    --heroicons) HEROICONS="$2"; shift 2 ;;
    --lucide) LUCIDE="$2"; shift 2 ;;
    --material) MATERIAL="$2"; shift 2 ;;
    --tabler) TABLER="$2"; shift 2 ;;
    *) echo "Unknown parameter: $1"; usage ;;
  esac
done

if [ -z "$CATEGORY" ] || [ -z "$NAME" ]; then
    echo "Error: --category and --name are required."
    usage
fi

FEATHER=${FEATHER:-$NAME}
HEROICONS=${HEROICONS:-$NAME}
LUCIDE=${LUCIDE:-$NAME}
MATERIAL=${MATERIAL:-$NAME}
TABLER=${TABLER:-$NAME}

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BASE_DIR="$DIR/../assets/icons"

# Create directories if they don't exist
mkdir -p "$BASE_DIR/feather/$CATEGORY"
mkdir -p "$BASE_DIR/heroicons/$CATEGORY"
mkdir -p "$BASE_DIR/lucide/$CATEGORY"
mkdir -p "$BASE_DIR/material-symbols/$CATEGORY"
mkdir -p "$BASE_DIR/tabler-icons/$CATEGORY"

echo "Downloading $NAME into $CATEGORY..."

# URLs
FEATHER_URL="https://raw.githubusercontent.com/feathericons/feather/master/icons/${FEATHER}.svg"
HEROICONS_URL="https://raw.githubusercontent.com/tailwindlabs/heroicons/master/optimized/24/outline/${HEROICONS}.svg"
LUCIDE_URL="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/${LUCIDE}.svg"
MATERIAL_URL="https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/${MATERIAL}/materialsymbolsoutlined/${MATERIAL}_wght400_24px.svg"
TABLER_URL="https://raw.githubusercontent.com/tabler/tabler-icons/master/icons/outline/${TABLER}.svg"

function download_and_format() {
    local url=$1
    local out=$2
    if curl -sfL "$url" -o "$out"; then
        # Replace currentColor with #FFFFFF, and convert black to #FFFFFF for some packs
        # For mac compatibility, sed -i '' is needed.
        sed -i '' 's/"currentColor"/"#FFFFFF"/g' "$out"
        sed -i '' 's/fill="black"/fill="#FFFFFF"/g' "$out"
        echo "   -> Saved to $out"
    else
        echo "   -> Failed to fetch from $url"
    fi
}

echo " -> Feather..."
download_and_format "$FEATHER_URL" "$BASE_DIR/feather/$CATEGORY/$NAME.svg"

echo " -> Heroicons..."
download_and_format "$HEROICONS_URL" "$BASE_DIR/heroicons/$CATEGORY/$NAME.svg"

echo " -> Lucide..."
download_and_format "$LUCIDE_URL" "$BASE_DIR/lucide/$CATEGORY/$NAME.svg"

echo " -> Material Symbols..."
download_and_format "$MATERIAL_URL" "$BASE_DIR/material-symbols/$CATEGORY/$NAME.svg"

echo " -> Tabler Icons..."
download_and_format "$TABLER_URL" "$BASE_DIR/tabler-icons/$CATEGORY/$NAME.svg"

echo "Done!"
