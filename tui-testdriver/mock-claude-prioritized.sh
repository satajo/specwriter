#!/usr/bin/env bash
# Mock that produces questions with priority values.
# Q1 has highest priority (p9), Q2 medium (p5), Q3 lowest (p2).

WORKDIR="$(pwd)"
SPEC="$WORKDIR/SPEC.md"

printf '# Spec\n\nApp requirements.\n' > "$SPEC"

printf '\n## Questions\n\n' >> "$SPEC"
printf '### Q1 (p9): What is the core feature?\n\nWhat single feature defines the product?\n\n### Q2 (p5): What is the target audience?\n\nWho are the primary users?\n\n### Q3 (p2): What color scheme?\n\nAny brand colors to follow?\n' >> "$SPEC"

echo "I have integrated the requirements."
