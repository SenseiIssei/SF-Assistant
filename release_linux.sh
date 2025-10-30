#!/usr/bin/env bash
set -euo pipefail

version=$(awk -F ' = ' '/^version = /{gsub(/"/,"",$2); print $2; exit}' Cargo.toml)
project_name="ShakesAutomation"

targets=("x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu")

if command -v sha256sum >/dev/null 2>&1; then
  checksum_cmd="sha256sum"
  checksum_ext="sha256sum"
else
  checksum_cmd="shasum -a 256"
  checksum_ext="sha256"
fi

mkdir -p dist
for target in "${targets[@]}"; do
  echo "Building ${project_name} ${version} for ${target}..."
  cargo build --release --quiet --target "${target}"

  bin_path="target/${target}/release/${project_name}"
  if [[ ! -f "${bin_path}" ]]; then
    echo "Error: binary not found at ${bin_path}"
    exit 1
  fi

  workdir="${project_name}_v${version}_${target}"
  rm -rf "${workdir}"
  mkdir -p "${workdir}"

  cp "${bin_path}" "${workdir}/${project_name}"
  cp LICENSE "${workdir}/LICENSE.txt"
  cp README.md "${workdir}/README.md"
  [[ -f THIRD_PARTY_LICENSES.txt ]] && cp THIRD_PARTY_LICENSES.txt "${workdir}/THIRD_PARTY_LICENSES.txt"
  [[ -f THIRD_PARTY_NOTICES.txt ]] && cp THIRD_PARTY_NOTICES.txt "${workdir}/THIRD_PARTY_NOTICES.txt"

  outfile="${workdir}.tar.gz"
  tar -C . -czf "${outfile}" "${workdir}"
  ${checksum_cmd} "${outfile}" > "dist/${outfile}.${checksum_ext}"
  mv "${outfile}" "dist/${outfile}"
  rm -rf "${workdir}"

done

echo "Done. Artifacts and checksums are in ./dist"