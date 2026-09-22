#!/usr/bin/env bash
set -euo pipefail

project_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
target="x86_64-pc-windows-gnu"

cd "${project_dir}"
package_id="$(cargo pkgid)"
version="${package_id##*#}"
package_name="photorealism-plugin-${version}-ets2-hdr"
dist_dir="${project_dir}/dist"
output_dir="${dist_dir}/${package_name}"
archive_path="${dist_dir}/${package_name}.zip"

cargo test
cargo clippy --all-targets -- -D warnings
cargo clippy --target "${target}" -- -D warnings
cargo zigbuild --release --target "${target}"

mkdir -p "${output_dir}/photorealism-plugin"
cp "target/${target}/release/photorealism_plugin.dll" "${output_dir}/dxgi.dll"
cp "config/photorealism-plugin.cfg" "${output_dir}/photorealism-plugin/"
cp "INSTALL-HDR.txt" "${output_dir}/"
bsdtar -a -cf "${archive_path}" -C "${dist_dir}" "${package_name}"

echo "Pasta gerada em ${output_dir}"
echo "Arquivo gerado em ${archive_path}"
