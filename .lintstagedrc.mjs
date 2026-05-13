/** @type {import('lint-staged').Config} */
export default {
  '*.{ts,vue}': ['eslint --fix', 'prettier --write'],
  '*.{js,mjs,cjs}': ['eslint --fix', 'prettier --write'],
  '*.{json,md,css,html}': ['prettier --write'],
  '*.rs': ['cargo fmt --manifest-path src-tauri/Cargo.toml -- --check'],
};
