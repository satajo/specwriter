#!/usr/bin/env bash
# Mock that returns 9 questions (at capacity).
# On subsequent calls, returns the same 9 (no new questions added).

WORKDIR="$(pwd)"
SPEC_DIR="$WORKDIR/spec"
README="$SPEC_DIR/README.md"

mkdir -p "$SPEC_DIR"

if [ -f "$README" ] && [ -s "$README" ]; then
    EXISTING=$(sed '/^## Questions$/,$d' "$README" | sed -e :a -e '/^\n*$/{$d;N;ba}')
    printf '%s\n\n---\n\nUpdated.\n' "$EXISTING" > "$README"
else
    printf '# Spec\n\nLarge application requirements.\n' > "$README"
fi

printf '\n## Questions\n\n' >> "$README"
printf 'Q1 (p9): Q one?\n\nQ2 (p8): Q two?\n\nQ3 (p7): Q three?\n\nQ4 (p6): Q four?\n\nQ5 (p5): Q five?\n\nQ6 (p4): Q six?\n\nQ7 (p3): Q seven?\n\nQ8 (p2): Q eight?\n\nQ9 (p1): Q nine?\n' >> "$README"

echo "I have integrated the requirements."
