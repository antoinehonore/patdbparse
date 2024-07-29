SHELL=/bin/bash

ifndef root_dir
	root_dir=/storage/projects/patdb
endif

ifndef PYTHON
	PYTHON=$(root_dir)/pyenv/bin/python
endif

ifndef indir
	indir=/mnt/shared_neo/data/data_takecare/wharf
endif

ifndef errorfile
	errorfile=errorfile.md
endif

ifndef nom_fname
	nom_fname=/mnt/shared_neo/data/data_takecare/nomenclature/Nomenclature.xlsx
endif

indir_nc_relative=$(shell nextcloud-path-relative $(realpath $(indir)))

all_files=$(shell find $(indir)/ -maxdepth 1 -type f -name "*_takecare.xlsx")

out_fname=$(indir)/error.log
readme_fname=$(indir)/$(errorfile)

parse-errors: find-errors
	./format_error_log.sh $(out_fname)  $(readme_fname)
	nextcloud-file-scan $(indir_nc_relative)/$(errorfile)
	nextcloud-file-scan $(indir_nc_relative)/error.log


# The files are processed sequentially to keep the clinIDs in order
find-errors: init
	for fname in $(all_files) ; do \
		$(PYTHON) $(root_dir)/takecare/xlsx2csv_takecare.py -i $$fname -nom $(nom_fname) > /dev/null 2>> $(out_fname) ; \
	done;

init:
	@echo "" > $(out_fname)

test:
	echo $(indir)
	echo $(realpath $(indir)/)
	echo $(indir_nc_relative)

