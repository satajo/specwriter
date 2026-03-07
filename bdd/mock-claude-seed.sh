#!/usr/bin/env bash
# Mock claude CLI for seeding tests.
# Embeds initial questions in existing spec files.

WORKDIR="$(pwd)"
SPEC_DIR="$WORKDIR/spec"
README="$SPEC_DIR/README.md"

if [ -f "$README" ]; then
    echo "" >> "$README"
    echo "?Q1: What is the primary user persona?" >> "$README"
    echo "?Q2: Are there performance requirements?" >> "$README"
fi

echo "Analyzed existing spec."
