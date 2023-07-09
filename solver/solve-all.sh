#!/bin/bash
set -eu -o pipefail
cargo build --release
for num in $(cat ../problems/stats.txt | grep -oE '^[0-9]+'); do
  echo "Solving $num"
  mkdir -p ../logs
  rm -f ../logs/$num.log
  time cargo -q run --release -- problem -i ../problems/$num.json -o ../solutions/$num.json -l ../logs/$num.log --rand-iters 1000000 --rand-max-secs 600
done;