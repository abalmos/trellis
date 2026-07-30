set -euo pipefail

npm_tag=latest
if [[ "${RELEASE_TAG:-}" == *-* ]]; then
  npm_tag=rc
fi
for package in \
  js/packages/result/npm \
  js/packages/trellis/npm \
  js/packages/trellis-svelte/npm
do
  npm publish --dry-run --access public --tag "$npm_tag" "$package"
done

for package in \
  js/packages/result \
  js/packages/trellis \
  js/packages/trellis-test
do
  (cd "$package" && deno publish --dry-run --allow-slow-types --allow-dirty)
done
