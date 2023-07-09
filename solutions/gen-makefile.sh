echo ".PHONY: all"
echo -n "all:"
for i in $(seq 1 90); do echo -n " $i.json $i.uploadid"; done;
echo

for i in $(seq 1 90); do
  echo -e "$i.json:"
  echo -e "\trm -f ../logs/$i.log"
  echo -e "\tcargo +nightly run --manifest-path ../solver/Cargo.toml --release -- problem -i ../problems/$i.json -o $i.json -l ../logs/$i.log --rand-iters 1000000 --rand-max-secs 60 --descent-iters 1000 --descent-max-secs 60";
  echo -e "\tfgrep 'score for' ../logs/$i.log"
  echo
done

for i in $(seq 1 90); do
  echo -e "$i.uploadid: $i.json"
  echo -e "\t./upload1.sh $i $i.json > $i.uploadid.tmp"
  echo -e "\tmv $i.uploadid.tmp $i.uploadid"
  echo
done
