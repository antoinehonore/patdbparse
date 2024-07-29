rdir=..
include ../cfg.mk

ifndef tcare_loc
	tcare_loc=/mnt/shared_neo/data/data_takecare/wharf
endif

ifndef nom_fname
	nom_fname=/mnt/shared_neo/data/data_takecare/nomenclature/Nomenclature.xlsx
endif

ifndef n_jobs
	n_jobs=2
endif


all_tcare_file=$(shell find $(tcare_loc)/ -type f -name "*_takecare.xlsx")
all_tcare_file_parsed_tmp=$(foreach tc_file, $(all_tcare_file), $(shell basename $(tc_file) | sed 's/.xlsx/.csv/g'))
all_tcare_file_parsed=$(addprefix $(TMP)/wharf/,$(all_tcare_file_parsed_tmp))

ifndef tk_table
	tk_table=takecare_data
endif


usage="Parse the takecare data."

help:
	echo -e $(usage)

parse: $(all_tcare_file_parsed)
	echo "Done parsing"

# Parsing
$(TMP)/wharf/%_takecare.csv: $(tcare_loc)/%_takecare.xlsx
	$(PYTHON) xlsx2csv_takecare.py -i $< -nom $(nom_fname) > $@ 2>> $(TMP)/takecare/error.log
	touch $@


clean:
	$(SQL_query) -c "drop table if exists $(tk_table);"
	$(SQL_query) -c "drop table if exists tktime;"
	rm -f $(TMP)/wharf/*.csv	
	echo "" > $(TMP)/takecare/error.log


.PRECIOUS:

.SECONDARY:
