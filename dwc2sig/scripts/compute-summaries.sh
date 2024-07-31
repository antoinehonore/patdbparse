#!/bin/bash

in_dir=$1
mode=$2

out_dir=$3
SHELL=$4

i=1
while [ -d $in_dir/pat$i ]; do
    patdir=$in_dir/pat$i
    $SHELL --verbose summarize -i $patdir -m $mode -o $out_dir || true
    i=$((i+1))
done

