#!/bin/bash

default_type="varchar(64)"
infile=$1
tmp_dir=`dirname $infile`

echo `head -n 1 $infile | \
       	sed -e 's/[\(\)\. ]//g' |\
       	sed -e 's/;/,/g' | \
	sed -e 's/[,$]/ varchar(64),/g'` $default_type > `dirname $infile`/overview.headers.tmp
wait

cat $tmp_dir/overview.headers.tmp | sed -e 's/BW varchar(64)/BW float/g' \
                                   -e 's/GAW varchar(64)/GAW float/g' \
                                   -e 's/BirthDate varchar(64)/BirthDate timestamp/g'\
                                   -e 's/\(Headcircum\) varchar(64)/\1 float/g'\
                                   -e 's/\(apgar[0-9]*\) varchar(64)/\1 float/g'
