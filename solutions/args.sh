ARGS=$(jq "." ./args.json)

for i in $(seq 1 2); do
  rand_iters=$(echo $ARGS | jq ".args[$i-1].rand_iters")
  rand_max_secs=$(echo $ARGS | jq ".args[$i-1].rand_max_secs")
  descent_iters=$(echo $ARGS | jq ".args[$i-1].descent_iters")
  descent_max_secs=$(echo $ARGS | jq ".args[$i-1].descent_max_secs")
  echo
done
