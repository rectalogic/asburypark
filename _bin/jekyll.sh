#!/usr/bin/env bash

MOUNT="$(cd "${BASH_SOURCE%/*}/.."; pwd)"
exec docker run --rm \
    --volume="$MOUNT:/srv/jekyll:Z" \
    --publish [::1]:4000:4000 \
    jekyll/jekyll \
    jekyll serve
