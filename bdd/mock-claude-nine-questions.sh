#!/usr/bin/env bash
# Mock that returns 9 questions (at capacity).
# On subsequent calls, returns the same 9 (no new questions added).

WORKDIR="$(pwd)"
SPEC_DIR="$WORKDIR/spec"
README="$SPEC_DIR/README.md"

mkdir -p "$SPEC_DIR"

if [ -f "$README" ] && [ -s "$README" ]; then
    EXISTING=$(grep -v "^?Q" "$README" || true)
    printf '%s\n\n---\n\nUpdated.\n' "$EXISTING" > "$README"
else
    printf '# Spec\n\nLarge application requirements.\n' > "$README"
fi

echo "" >> "$README"
echo "?Q1: Q one?" >> "$README"
echo "?Q2: Q two?" >> "$README"
echo "?Q3: Q three?" >> "$README"
echo "?Q4: Q four?" >> "$README"
echo "?Q5: Q five?" >> "$README"
echo "?Q6: Q six?" >> "$README"
echo "?Q7: Q seven?" >> "$README"
echo "?Q8: Q eight?" >> "$README"
echo "?Q9: Q nine?" >> "$README"

echo "I have integrated the requirements."
