#rdir=..
#include ../cfg.mk

PYTHON=python

ifndef clin_loc
	clin_loc=/storage/raw/clinisoft
endif

ifndef tmp_loc
	tmp_loc=$(TMP)/clinisoft
endif

ifndef n_jobs
	n_jobs=1
endif

all_clin_file=$(shell find $(clin_loc)/ -type f -name "*.xlsx")

all_clin_file_csv=$(foreach clin_file,$(all_clin_file),$(shell basename $(clin_file) | sed 's/.xlsx/_read.csv.loaded/g'))
all_clin_file_csv_parsed=$(foreach clin_file,$(all_clin_file),$(shell basename $(clin_file) | sed 's/.xlsx/_read.csv/g'))


parse: $(addprefix $(tmp_loc)/, $(all_clin_file_csv_parsed))

# Parsing
$(tmp_loc)/%_read.csv: $(clin_loc)/%.xlsx
	$(PYTHON) bin/read_clin.py -i $^ -o $@
	touch $@

