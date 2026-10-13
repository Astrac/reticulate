#!/usr/bin/env bash
# Generate the Zola pages that republish the root markdown files, so the
# project docs remain the single source of truth. Output is git-ignored.
set -euo pipefail
cd "$(dirname "$0")/../.."

out=site/content

# Rewrite links to the root files into Zola internal links, and collapse runs
# of dashes in in-page anchors (GitHub slugs keep them, Zola's do not).
fix_links() {
  perl -pe '1 while s/(\(#[^)]*?)--+/$1-/; s/(\(#[^)]*)meshingbitfields/$1meshing-bitfields/' | sed -E \
    -e 's#\(\./?CONTRIBUTING\.md(\.md)?\)#(@/contributing.md)#g' \
    -e 's#\(\./?LICENSE\.md\)#(@/license.md)#g'
}

{ printf '+++\ntitle = "Home"\n+++\n\n'; fix_links < README.md; } > "$out/_index.md"
{ printf '+++\ntitle = "Contributing"\nweight = 2\n+++\n\n'; fix_links < CONTRIBUTING.md; } > "$out/contributing.md"
{ printf '+++\ntitle = "License"\nweight = 3\n+++\n\n'; cat LICENSE.md; } > "$out/license.md"
