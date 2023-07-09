#!/usr/bin/env sh

if [ -z "$TOKEN" ]; then
    echo "Please set env var TOKEN" >&2
    exit 1
fi

prob=$1
sol_file=$2

curl -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" https://api.icfpcontest.com/submission --data "{\"problem_id\": $prob, \"contents\": $(jq '. | tostring' $sol_file)}" | jq -r '.'
