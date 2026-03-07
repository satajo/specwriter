#!/usr/bin/env bash
# Mock claude that produces no questions
WORKDIR="$(pwd)"
SPEC_DIR="$WORKDIR/specs"
mkdir -p "$SPEC_DIR"
cat > "$SPEC_DIR/README.md" << EOF
# Spec

Simple requirements.
EOF

echo "Done integrating."
