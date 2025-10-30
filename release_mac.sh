#!/usr/bin/env bash
set -euo pipefail

version=$(awk -F ' = ' '/^version = /{gsub(/"/,"",$2); print $2; exit}' Cargo.toml)
host_target=$(rustc -vV | sed -n 's/^host: //p')
if [[ -n "${TARGETS:-}" ]]; then
  read -r -a targets <<< "${TARGETS}"
else
  targets=("${host_target}")
fi

crate_name="sf-assistant"
product_name="SFAssistant"

if command -v sha256sum >/dev/null 2>&1; then
  checksum_cmd="sha256sum"
  checksum_ext="sha256sum"
else
  checksum_cmd="shasum -a 256"
  checksum_ext="sha256"
fi

mkdir -p dist
for target in "${targets[@]}"; do
  echo "Building ${product_name} ${version} for ${target}..."
  cargo build --release --quiet --target "${target}"

  bin_path="target/${target}/release/${crate_name}"
  [[ -f "${bin_path}" ]] || { echo "Error: binary not found at ${bin_path}"; exit 1; }

  workdir="${product_name}_v${version}_${target}"
  rm -rf "${workdir}"
  mkdir -p "${workdir}"

  cp "${bin_path}" "${workdir}/${product_name}"
  cp LICENSE "${workdir}/LICENSE.txt"
  cp README.md "${workdir}/README.md"
  [[ -f THIRD_PARTY_LICENSES.txt ]] && cp THIRD_PARTY_LICENSES.txt "${workdir}/THIRD_PARTY_LICENSES.txt"
  [[ -f THIRD_PARTY_NOTICES.txt ]] && cp THIRD_PARTY_NOTICES.txt "${workdir}/THIRD_PARTY_NOTICES.txt"

  outfile="${workdir}.zip"
  zip -rq "${outfile}" "${workdir}"
  ${checksum_cmd} "${outfile}" > "dist/${outfile}.${checksum_ext}"
  mv "${outfile}" "dist/${outfile}"
  rm -rf "${workdir}"
done

echo "Done. Artifacts and checksums are in ./dist"
