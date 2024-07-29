# Overview
This project parses clinisoft data 

# Parsing
## Description
The parsing script takes as input a clinisoft Excel file.
The excel files is decomposed into .csv files.
Each .csv file contains a certain type of variables.

For instance if the input file is called 89213.xlsx
The script outputs:

|Output file| Type |
| -- | --------- |
|89213_read_vikt.csv|Weight|
|89213_read_vatska.csv|Fluids injection|
|89213_read_tempaxil.csv|Temperature|
|89213_read_respirator.csv|Ventilator mode|
|89213_read_pressure.csv|Ventilator pressure|
|89213_read_med.csv|Medication injections|
|89213_read_lab.csv|Lab values|
|89213_read_hpn.csv|Encrypted PN|
|89213_read_fio2.csv|fio2|
|89213_read.csv|Flag|

## Usage
### Parsing a single file (not recommended)
```bash
$ python bin/read_clin.py -h
usage: read_clin.py [-h] [-i Input] [-o Output]

Read a clinisoft file into csv

optional arguments:
  -h, --help  show this help message and exit
  -i Input    Input Clinisoft Excel file.
  -o Output   Out Clinisoft .csv file.

$ python clinisoft/read_clin.py -i env/clinisoft/1000.xlsx -o tmp/clinisoft/1000_read.csv
```

### Parse and load only new clinisoft files (recommended)
A typical use case is:
```bash
$ make parse -j [n_jobs]  clin_loc=path/to/all/files tmp_loc=path/to/tmpfolder
```
