#!/bin/bash
set -eu -o pipefail
for i in $(seq 1 90); do
  python3 vis_prob.py $i.svg ../problems/$i.json ../solutions/$i.json
done
