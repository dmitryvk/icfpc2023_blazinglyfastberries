#!/bin/bash
set -eu -o pipefail
N_PROBLEMS=$(curl https://api.icfpcontest.com/problems | jq '.number_of_problems')
for i in $(seq 1 $N_PROBLEMS); do
  curl https://api.icfpcontest.com/problem?problem_id=$i | jq -r '.Success' > $i.json
done