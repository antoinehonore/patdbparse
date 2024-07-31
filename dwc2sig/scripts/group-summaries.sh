#!/bin/bash
help="Group the sig_summary.txt files from subdirectory named pat... to a file called PatientsSignals.txt\nfind-summaries [input directory]"

in_dir=$1
mode=$2

outfile=$in_dir/PatientsSignals_${mode}.txt
echo "" > $outfile

i=1
while [ -d $in_dir/pat$i ]; do
    fname=$in_dir/pat$i/sig_summary_${mode}.txt 
    if [ -f $fname ]; then
        cat $fname | grep -e "^data_monitor.*$" >> $outfile
    fi
    i=$((i+1))
done
