#!/usr/bin/bash

for i in {1..30}
do
    echo "Task $i..."
    python3 api.py download $i outputs/$i.isl
done