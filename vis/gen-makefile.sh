echo ".PHONY: all"
echo -n "all:"
for i in $(seq 1 55); do echo -n " $i.svg"; done;
echo
echo

for i in $(seq 1 55); do echo -e "$i.svg: ../problems/$i.json ../solutions/$i.json\n\t. venv/bin/activate && python3 vis_prob.py $i.svg ../problems/$i.json ../solutions/$i.json"; done
