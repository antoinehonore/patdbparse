rdir=..
include ../cfg.mk

overview=$(ENV)/PatientOverview.xlsx

load: format
	$(MAKE) -f Makefile init
	$(SQL_query) -c "\copy raw_overview from '$(tsDB_wd)/tmp/overview.csv' with DELIMITER ';' CSV HEADER;"
	$(SQL_query) -c "create table unique_pn as (select distinct ov.personnummer from raw_overview as ov);"
	$(SQL_query) -c "ALTER TABLE unique_pn ADD PRIMARY KEY (personnummer);"
	$(SQL_query) -c "ALTER TABLE raw_overview ADD CONSTRAINT constraint_fk FOREIGN KEY (personnummer) REFERENCES unique_pn (personnummer);"


# expand ORs
format:
	$(PYTHON) xlsx2csv_overview.py $(overview) -o - | $(CARGO_RUN) --manifest-path $(UTILS)/split_or_/Cargo.toml > $(TMP)/overview.csv


init:
	mkdir -p $(TMP)
	$(eval raw_overview_headers=$(shell ./inferheaders.sh $(TMP)/overview.csv))
	$(SQL_query) -c "create table if not exists raw_overview ($(raw_overview_headers));"
	$(SQL_query) -c "ALTER TABLE raw_overview ADD PRIMARY KEY (clinisoftid);"
	$(SQL_query) -c "CREATE INDEX on raw_overview(personnummer);"


clean:
	$(SQL_query) -c "drop table if exists raw_overview;"
	$(SQL_query) -c "drop table if exists unique_pn;"
	rm -f $(TMP)/overview.csv


