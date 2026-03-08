#!/usr/bin/env bash
# Mock claude CLI for BDD tests.
# Writes to SPEC.md and places questions under ## Questions heading.

PROMPT="${@: -1}"
WORKDIR="$(pwd)"
SPEC="$WORKDIR/SPEC.md"

# Build content section (strip existing Questions heading and below)
if [ -f "$SPEC" ] && [ -s "$SPEC" ]; then
    EXISTING=$(sed '/^## Questions$/,$d' "$SPEC" | sed -e :a -e '/^\n*$/{$d;N;ba}')
    printf '%s\n\n---\n\nUpdated with new requirements from the latest integration.\n' "$EXISTING" > "$SPEC"
else
    printf '# Spec\n\nRequirements integrated from user input.\n' > "$SPEC"
fi

# Extract keywords from the prompt to make spec reflective
echo "$PROMPT" | grep -qi "login" && echo "login" >> "$SPEC"
echo "$PROMPT" | grep -qi "password" && echo "password" >> "$SPEC"
echo "$PROMPT" | grep -qi "authentication" && echo "authentication" >> "$SPEC"
echo "$PROMPT" | grep -qi "search" && echo "search" >> "$SPEC"
echo "$PROMPT" | grep -qi "filter" && echo "filtering" >> "$SPEC"

# Place questions under ## Questions heading as ### subheadings
printf '\n## Questions\n\n' >> "$SPEC"
if echo "$PROMPT" | grep -qi "OAuth"; then
    # Answered Q1 (auth question), keep Q2, Q3, add Q4
    printf '### Q2 (p6): Should there be role-based access?\n\nDo different users need different permissions?\n\n### Q3 (p4): What is the target platform?\n\nWeb, mobile, or desktop?\n\n### Q4 (p7): What OAuth providers should be supported?\n\nGoogle, GitHub, etc.?\n' >> "$SPEC"
elif echo "$PROMPT" | grep -qi "search"; then
    # Keep Q1, Q2, remove Q3, add Q4
    printf '### Q1 (p8): What are the authentication requirements?\n\nHow should users authenticate?\n\n### Q2 (p6): Should there be role-based access?\n\nDo different users need different permissions?\n\n### Q4 (p5): What search fields are needed?\n\nWhich fields should be searchable?\n' >> "$SPEC"
else
    # Default: Q1, Q2, Q3
    printf '### Q1 (p8): What are the authentication requirements?\n\nHow should users authenticate?\n\n### Q2 (p6): Should there be role-based access?\n\nDo different users need different permissions?\n\n### Q3 (p4): What is the target platform?\n\nWeb, mobile, or desktop?\n' >> "$SPEC"
fi

echo "I have integrated the requirements."
