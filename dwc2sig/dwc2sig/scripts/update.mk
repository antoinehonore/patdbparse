

data_root_folder=/mnt/cmm-share/Private/patients_data
data_out_folder=/storage/raw/parsed_monitor

ifndef data_folders_stem
	data_folders_stem= data_monitor_191SG-1 data_monitor_NEO_191SG-1 data_monitor_NEO_1901 data_monitor_COVID_191SG-1_DB2 data_monitor_COVID_1901 data_monitor data_monitor_past data_monitor_offset_queries
	#data_folders_stem= data_monitor_COVID_191SG-1_DB2 data_monitor_COVID_1901 data_monitor_2021__COVID_191SG_DB2
	#data_monitor_offset_queries data_monitor_past data_monitor
endif

ifndef mode
	mode=LF
endif

ifndef SHELL
	SHELL=../bin/patdb_bin
endif


data_folders=$(addprefix $(data_root_folder)/,$(data_folders_stem))
targets=$(addsuffix /PatientsSignals_$(mode).txt,$(data_folders))

all: $(targets)

test:
	@echo $(data_root_folder)
	@echo $(data_folders)


$(data_root_folder)/%/PatientsSignals_$(mode).txt: %_all_sig_summaries
	./group-summaries.sh $(data_root_folder)/$* $(mode)

%_all_sig_summaries:
	./compute-summaries.sh $(data_root_folder)/$* $(mode) $(data_out_folder) $(SHELL)


clean: $(addprefix clean-,$(data_folders_stem))


clean-%:
	rm -f $(data_root_folder)/$*/PatientsSignals_$(mode).txt
	./clear-summaries.sh $(data_root_folder)/$* $(mode)

	
.SECONDARY:
