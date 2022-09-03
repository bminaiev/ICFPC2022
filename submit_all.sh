#!/usr/bin/bash

for i in {1..25}
do
    echo "Task $i..."
    python3 api.py submit $i outputs/$i.isl
done