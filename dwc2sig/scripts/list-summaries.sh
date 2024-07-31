#!/bin/bash

in_dir=$1

i=1

i=1
while [ -d $in_dir/pat$i ]; do
    fname=$in_dir/pat$i/sig_summary.txt 
    if [ -f $fname ]; then
        echo $fname
    fi
    i=$((i+1))
done

