#!/usr/bin/env bash
# Slow seeding mock for testing the loading status message.

WORKDIR="$(pwd)"

sleep 0.5

echo "Analyzed existing spec."
echo 'QUESTIONS:[{"id":1,"text":"What is the primary user persona?"},{"id":2,"text":"Are there performance requirements?"}]'
