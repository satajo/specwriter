#!/usr/bin/env bash
# Slow seeding mock for testing the loading status message.

WORKDIR="$(pwd)"
SPEC_DIR="$WORKDIR/spec"
README="$SPEC_DIR/README.md"

sleep 0.5

if [ -f "$README" ]; then
    echo "" >> "$README"
    echo "?Q1: What is the primary user persona?" >> "$README"
    echo "?Q2: Are there performance requirements?" >> "$README"
fi

echo "Analyzed existing spec."
