#!/usr/bin/env bash
# Mock that returns 9 questions (at capacity).
# On subsequent calls, returns the same 9 (no new questions added).

WORKDIR="$(pwd)"
SPEC="$WORKDIR/SPEC.md"

if [ -f "$SPEC" ] && [ -s "$SPEC" ]; then
    EXISTING=$(sed '/^## Questions$/,$d' "$SPEC" | sed -e :a -e '/^\n*$/{$d;N;ba}')
    printf '%s\n\n---\n\nUpdated.\n' "$EXISTING" > "$SPEC"
else
    printf '# Spec\n\nLarge application requirements.\n' > "$SPEC"
fi

printf '\n## Questions\n\n' >> "$SPEC"
printf '### Q1 (p9): Q one?\n\nDetails.\n\n### Q2 (p8): Q two?\n\nDetails.\n\n### Q3 (p7): Q three?\n\nDetails.\n\n### Q4 (p6): Q four?\n\nDetails.\n\n### Q5 (p5): Q five?\n\nDetails.\n\n### Q6 (p4): Q six?\n\nDetails.\n\n### Q7 (p3): Q seven?\n\nDetails.\n\n### Q8 (p2): Q eight?\n\nDetails.\n\n### Q9 (p1): Q nine?\n\nDetails.\n' >> "$SPEC"

echo "I have integrated the requirements."
