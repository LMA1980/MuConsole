#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
#
# Fetch, verify and subset the MuConsole fonts.
#
#   scripts/fetch-fonts.sh            # writes dist/fonts/*.woff2 + licences
#
# - Downloads pinned Nerd Fonts release zips (Hermit, OpenDyslexic, HeavyData)
#   from https://github.com/ryanoasis/nerd-fonts/releases.
# - Verifies each zip against the SHA-256 recorded below (fails closed).
# - Subsets with pyftsubset (fontTools, in a local venv) using the code point
#   list generated from muconsole-theme's glyph table, then checks that every
#   required code point (Nerd icons, state glyphs, ellipsis) is present.
# - Writes WOFF2 subsets, each family's licence file and a manifest with
#   content hashes (for ?v= cache busting) into dist/fonts/.
#
# Font binaries are NEVER committed (see .gitignore). HeavyData is Vic Fieger
# freeware, not OFL: its licence ships next to the subset in dist/fonts/.
set -euo pipefail

NF_VERSION="v3.5.1"
NF_BASE="https://github.com/ryanoasis/nerd-fonts/releases/download/${NF_VERSION}"

# SHA-256 of the release assets (match GitHub's published asset digests).
sha_for() {
  case "$1" in
    Hermit.zip) echo "fd78d27e2a4f1b92182ae56d2a6833028bc1314f629d773790516c7489602f3e" ;;
    OpenDyslexic.zip) echo "3b03b18b5c2436490d4f4df0d51f74cb0416ee74ddb189e3b698fd8b70807697" ;;
    HeavyData.zip) echo "d8a15e63ebc3aa433fe4d11dfae10e06bd53060e2d9dec0f25043c0ee4507907" ;;
    *) echo "unknown asset $1" >&2; return 1 ;;
  esac
}

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE="${ROOT}/.fonts-cache/${NF_VERSION}"
OUT="${ROOT}/dist/fonts"
VENV="${ROOT}/.venv"
mkdir -p "${CACHE}" "${OUT}"

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

fetch() {
  local asset="$1" want got
  want="$(sha_for "${asset}")"
  if [[ -f "${CACHE}/${asset}" ]] && [[ "$(sha256 "${CACHE}/${asset}")" == "${want}" ]]; then
    echo "cached   ${asset}"
  else
    echo "download ${asset}"
    curl -fsSL --retry 3 -o "${CACHE}/${asset}.part" "${NF_BASE}/${asset}"
    mv "${CACHE}/${asset}.part" "${CACHE}/${asset}"
  fi
  got="$(sha256 "${CACHE}/${asset}")"
  if [[ "${got}" != "${want}" ]]; then
    echo "SHA-256 mismatch for ${asset}: got ${got}, want ${want}" >&2
    rm -f "${CACHE}/${asset}"
    exit 1
  fi
  echo "verified ${asset} ${got}"
}

# extract <zip> <member> <dest>
extract() {
  unzip -p "${CACHE}/$1" "$2" > "$3"
  [[ -s "$3" ]] || { echo "missing $2 in $1" >&2; exit 1; }
}

for z in Hermit.zip OpenDyslexic.zip HeavyData.zip; do fetch "$z"; done

if [[ ! -x "${VENV}/bin/pyftsubset" ]]; then
  python3 -m venv "${VENV}"
  "${VENV}/bin/pip" install --quiet --disable-pip-version-check "fonttools[woff]==4.*"
fi

CRT_UNICODES="$(cd "${ROOT}/code" && cargo run --quiet -p mu-console-theme --bin mu-console-subset-unicodes -- crt)"
ETCH_UNICODES="$(cd "${ROOT}/code" && cargo run --quiet -p mu-console-theme --bin mu-console-subset-unicodes -- etch)"
CRT_REQUIRED="$(cd "${ROOT}/code" && cargo run --quiet -p mu-console-theme --bin mu-console-subset-unicodes -- crt --required | tr '\n' ' ')"
ETCH_REQUIRED="$(cd "${ROOT}/code" && cargo run --quiet -p mu-console-theme --bin mu-console-subset-unicodes -- etch --required | tr '\n' ' ')"

WORK="$(mktemp -d)"
trap 'rm -rf "${WORK}"' EXIT

extract Hermit.zip HurmitNerdFontMono-Regular.otf "${WORK}/HurmitNerdFontMono-Regular.otf"
extract Hermit.zip HurmitNerdFontMono-Bold.otf "${WORK}/HurmitNerdFontMono-Bold.otf"
extract OpenDyslexic.zip OpenDyslexicMNerdFontMono-Regular.otf "${WORK}/OpenDyslexicMNerdFontMono-Regular.otf"
extract HeavyData.zip HeavyDataNerdFont-Regular.ttf "${WORK}/HeavyDataNerdFont-Regular.ttf"

extract Hermit.zip LICENSE "${OUT}/Hurmit-LICENSE.txt"
extract OpenDyslexic.zip LICENSE.txt "${OUT}/OpenDyslexicM-LICENSE.txt"
extract HeavyData.zip "Vic Fieger License.txt" "${OUT}/HeavyData-Vic-Fieger-License.txt"

# subset <src> <stem> <unicodes> <required>
subset() {
  local src="$1" stem="$2" unicodes="$3" required="$4"
  "${VENV}/bin/pyftsubset" "${src}" \
    --unicodes="${unicodes}" \
    --flavor=woff2 \
    --layout-features='kern' \
    --no-hinting \
    --desubroutinize \
    --name-IDs='*' \
    --name-legacy \
    --notdef-outline \
    --output-file="${OUT}/${stem}.woff2"
  "${VENV}/bin/python" - "${OUT}/${stem}.woff2" ${required} <<'PY'
import sys
from fontTools.ttLib import TTFont
path, wanted = sys.argv[1], sys.argv[2:]
cmap = TTFont(path).getBestCmap()
missing = [cp for cp in wanted if int(cp, 16) not in cmap]
if missing:
    sys.exit(f"{path}: missing required code points: {' '.join('U+' + m for m in missing)}")
PY
}

subset "${WORK}/HurmitNerdFontMono-Regular.otf" HurmitNerdFontMono-Regular "${CRT_UNICODES}" "${CRT_REQUIRED}"
subset "${WORK}/HurmitNerdFontMono-Bold.otf" HurmitNerdFontMono-Bold "${CRT_UNICODES}" "${CRT_REQUIRED}"
subset "${WORK}/OpenDyslexicMNerdFontMono-Regular.otf" OpenDyslexicMNerdFontMono-Regular "${CRT_UNICODES}" "${CRT_REQUIRED}"
subset "${WORK}/HeavyDataNerdFont-Regular.ttf" HeavyDataNerdFont-Regular "${ETCH_UNICODES}" "${ETCH_REQUIRED}"

{
  echo "# MuConsole font subsets from Nerd Fonts ${NF_VERSION}"
  echo "# file  bytes  sha256  v (?v= cache-busting hash)"
  for f in "${OUT}"/*.woff2; do
    h="$(sha256 "$f")"
    printf '%s  %s  %s  %s\n' "$(basename "$f")" "$(wc -c < "$f" | tr -d ' ')" "${h}" "${h:0:12}"
  done
} > "${OUT}/manifest.txt"

cat "${OUT}/manifest.txt"
