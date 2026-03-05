#!/usr/bin/env bash
# Mock claude that produces no questions
WORKDIR="$(pwd)"
cat > "$WORKDIR/SPEC.md" << EOF
# Spec

Simple requirements.
EOF

echo "Done integrating."
