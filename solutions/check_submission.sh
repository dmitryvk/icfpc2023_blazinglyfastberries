#!/usr/bin/env sh

if [ -z "$TOKEN" ]; then
    echo "Please set env var TOKEN" >&2
    exit 1
fi

submissionid=$1

curl -H "Authorization: Bearer $TOKEN" https://api.icfpcontest.com/submission?submission_id="${submissionid}" | jq -c '.Success.submission.score'
