#!/usr/bin/env bash
# Mock that returns 9 questions (at capacity).
# On subsequent calls, returns the same 9 (no new questions added).

WORKDIR="$(pwd)"
SPEC="$WORKDIR/SPEC.md"

if [ -f "$SPEC" ] && [ -s "$SPEC" ]; then
    EXISTING=$(cat "$SPEC")
    cat > "$SPEC" << EOF
$EXISTING

---

Updated.
EOF
else
    cat > "$SPEC" << EOF
# Spec

Large application requirements.
EOF
fi

echo "I have integrated the requirements into SPEC.md."
echo 'QUESTIONS:[{"id":1,"text":"Q one?"},{"id":2,"text":"Q two?"},{"id":3,"text":"Q three?"},{"id":4,"text":"Q four?"},{"id":5,"text":"Q five?"},{"id":6,"text":"Q six?"},{"id":7,"text":"Q seven?"},{"id":8,"text":"Q eight?"},{"id":9,"text":"Q nine?"}]'
