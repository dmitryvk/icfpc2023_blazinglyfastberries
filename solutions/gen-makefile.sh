echo ".PHONY: solve upload check"
echo -n "solve:"
for i in $(seq 1 90); do echo -n " $i.json"; done;
echo
echo -n "upload:"
for i in $(seq 1 90); do echo -n " $i.uploadid"; done;
echo
echo -n "check:"
for i in $(seq 1 90); do echo -n " $i.status"; done;
echo

ARGS=$(jq "." ./args.json)

for i in $(seq 1 90); do
  rand_iters=$(echo $ARGS | jq ".args[$i-1].rand_iters")
  rand_max_secs=$(echo $ARGS | jq ".args[$i-1].rand_max_secs")
  descent_iters=$(echo $ARGS | jq ".args[$i-1].descent_iters")
  descent_max_secs=$(echo $ARGS | jq ".args[$i-1].descent_max_secs")
  n_threads=$(echo $ARGS | jq ".args[$i-1].n_threads")
  n_seeds=$(echo $ARGS | jq ".args[$i-1].n_seeds")
  echo -e "$i.json:"
  echo -e "\trm -f ../logs/$i.log"
  run="\tcargo +nightly run --manifest-path ../solver/Cargo.toml --release -- problem -i ../problems/$i.json -o $i.json -l ../logs/$i.log "
  run+="--rand-iters $rand_iters --rand-max-secs $rand_max_secs --descent-iters $descent_iters --descent-max-secs $descent_max_secs ";
  run+="--n-threads $n_threads --n-seeds $n_seeds";
  echo -e $run
  echo -e "\tfgrep 'score for' ../logs/$i.log"
  echo
done

for i in $(seq 1 90); do
  echo -e "$i.uploadid: $i.json"
  echo -e "\t./upload1.sh $i $i.json > $i.uploadid.tmp"
  echo -e "\tmv $i.uploadid.tmp $i.uploadid"
  echo
done

for i in $(seq 1 90); do
  echo -e "$i.status: $i.uploadid"
  echo -e "\t./check_submission.sh \$\$(cat $i.uploadid) | tee $i.status.tmp"
  echo -e "\tmv $i.status.tmp $i.status"
  echo
done
