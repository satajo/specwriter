#!/usr/bin/env bash
# Mock that produces questions with priority values.
# Q1 has highest priority (p9), Q2 medium (p5), Q3 lowest (p2).

WORKDIR="$(pwd)"
SPEC_DIR="$WORKDIR/specs"
README="$SPEC_DIR/README.md"

mkdir -p "$SPEC_DIR"

printf '# Spec\n\nApp requirements.\n' > "$README"

printf '\n## Questions\n\n' >> "$README"
printf '### Q1 (p9): What is the core feature?\n\nWhat single feature defines the product?\n\n### Q2 (p5): What is the target audience?\n\nWho are the primary users?\n\n### Q3 (p2): What color scheme?\n\nAny brand colors to follow?\n' >> "$README"

echo "I have integrated the requirements."
