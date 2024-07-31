#!/bin/bash

in_dir=$1
mode=$2

i=1
while [ -d $in_dir/pat$i ]; do
    rm -f $in_dir/pat$i/sig_summary_$mode.txt 
	#rm -f $in_dir/pat$i/LF_1*.csv
    i=$((i+1))
done
