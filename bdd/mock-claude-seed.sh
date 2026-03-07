#!/usr/bin/env bash
# Mock claude CLI for seeding tests.
# Returns initial questions from an existing spec.

WORKDIR="$(pwd)"

echo "Analyzed existing spec."
echo 'QUESTIONS:[{"id":1,"text":"What is the primary user persona?"},{"id":2,"text":"Are there performance requirements?"}]'
