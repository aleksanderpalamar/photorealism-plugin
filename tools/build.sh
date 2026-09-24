#!/usr/bin/env bash
set -Eeuo pipefail

readonly TARGET="x86_64-pc-windows-gnu"
readonly PACKAGE_PREFIX="photorealism-plugin"
readonly LEGACY_SUFFIX="ets2-hdr"

project_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
backup_dir=""

usage() {
    cat <<'TEXT'
Uso: tools/build.sh [major|minor|patch] [--dry-run]

Calcula a proxima versao a partir da maior versao conhecida no Cargo.toml,
nos pacotes ja presentes em dist e nas tags do git, grava essa versao no
Cargo.toml e gera o pacote correspondente.

  major      0.2.1 -> 1.0.0
  minor      0.2.1 -> 0.3.0
  patch      0.2.1 -> 0.2.2  (padrao)
  --dry-run  mostra a versao calculada sem alterar nem construir nada

Se qualquer etapa falhar, Cargo.toml e Cargo.lock voltam ao estado anterior.
TEXT
}

manifest_version() {
    awk -F'"' '/^version = "/ { print $2; exit }' Cargo.toml
}

packaged_versions() {
    [[ -d dist ]] || return 0
    find dist -maxdepth 1 -name "${PACKAGE_PREFIX}-*" -printf '%f\n' |
        sed -e "s/^${PACKAGE_PREFIX}-//" -e "s/\.zip$//" -e "s/-${LEGACY_SUFFIX}$//"
}

tagged_versions() {
    git tag --list 'v*' 2>/dev/null | sed 's/^v//' || true
}

highest_version() {
    { manifest_version; packaged_versions; tagged_versions; } |
        grep -E '^[0-9]+\.[0-9]+\.[0-9]+$' |
        sort -V |
        tail -n 1
}

next_version() {
    local base="${1}" level="${2}" major minor patch
    IFS=. read -r major minor patch <<<"${base}"
    case "${level}" in
        major) echo "$((major + 1)).0.0" ;;
        minor) echo "${major}.$((minor + 1)).0" ;;
        patch) echo "${major}.${minor}.$((patch + 1))" ;;
    esac
}

package_name() {
    echo "${PACKAGE_PREFIX}-${1}"
}

assert_package_is_new() {
    local name
    name="$(package_name "${1}")"
    [[ -e "dist/${name}" || -e "dist/${name}.zip" ]] || return 0
    echo "O pacote ${name} ja existe em dist." >&2
    return 1
}

write_version() {
    backup_dir="$(mktemp -d)"
    cp Cargo.toml Cargo.lock "${backup_dir}/"
    sed -i "0,/^version = \".*\"$/s//version = \"${1}\"/" Cargo.toml
}

discard_backup() {
    [[ -n "${backup_dir}" ]] || return 0
    rm -rf "${backup_dir}"
    backup_dir=""
}

restore_version() {
    [[ -n "${backup_dir}" ]] || return 0
    cp "${backup_dir}/Cargo.toml" "${backup_dir}/Cargo.lock" "${project_dir}/"
    discard_backup
    echo "Cargo.toml e Cargo.lock restaurados." >&2
}

run_checks() {
    cargo test
    cargo clippy --all-targets -- -D warnings
    cargo clippy --target "${TARGET}" -- -D warnings
    cargo zigbuild --release --target "${TARGET}"
}

build_package() {
    local name output_dir archive_path
    name="$(package_name "${1}")"
    output_dir="${project_dir}/dist/${name}"
    archive_path="${project_dir}/dist/${name}.zip"

    mkdir -p "${output_dir}/photorealism-plugin"
    cp "target/${TARGET}/release/photorealism_plugin.dll" "${output_dir}/dxgi.dll"
    cp "config/photorealism-plugin.cfg" "${output_dir}/photorealism-plugin/"
    cp "INSTALL.txt" "${output_dir}/"
    bsdtar -a -cf "${archive_path}" -C "${project_dir}/dist" "${name}"

    echo "Pasta gerada em ${output_dir}"
    echo "Arquivo gerado em ${archive_path}"
}

level="patch"
dry_run="false"
for argument in "$@"; do
    case "${argument}" in
        major | minor | patch) level="${argument}" ;;
        --dry-run) dry_run="true" ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            usage >&2
            exit 1
            ;;
    esac
done

cd "${project_dir}"

base="$(highest_version)"
version="$(next_version "${base}" "${level}")"

if [[ "${dry_run}" == "true" ]]; then
    echo "Versao no Cargo.toml: $(manifest_version)"
    echo "Maior versao conhecida: ${base}"
    echo "Proxima versao (${level}): ${version}"
    exit 0
fi

trap restore_version ERR INT TERM

assert_package_is_new "${version}"
write_version "${version}"
run_checks
build_package "${version}"

trap - ERR INT TERM
discard_backup

echo "Cargo.toml e Cargo.lock atualizados para ${version}."
echo "Sugestao: git commit -am 'chore(release): v${version}'"
