echo ".PHONY: all"
echo -n "all:"
for i in $(seq 1 90); do echo -n " $i.json"; done;
echo

for i in $(seq 1 90); do echo -e "$i.json:\n\tcargo +nightly run --manifest-path ../solver/Cargo.toml --release -- problem -i ../problems/$i.json -o $i.json -l ../logs/$i.log --rand-iters 1000000 --rand-max-secs 600"; done
