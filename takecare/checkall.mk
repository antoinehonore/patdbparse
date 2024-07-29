SHELL=/bin/bash
root_dir=/storage/projects/patdb

ifndef work_dir
	work_dir=/mnt/shared_neo/data/data_takecare/work
endif
ifndef
	nom_fname=/mnt/shared_neo/data/data_takecare/nomenclature/Nomenclature.xlsx
endif

ifndef errorfile
	errorfile=errorfile.md
endif

all_targets=$(shell find $(work_dir)/ -type f -name $(errorfile))


all: $(all_targets)

test:
	echo $(all_targets)

%/$(errorfile):
	$(MAKE) -f checkfolder.mk indir=$* nom_fname=$(nom_fname) errorfile=$(errorfile)

