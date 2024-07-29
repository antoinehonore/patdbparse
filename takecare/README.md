[![License](https://img.shields.io/badge/License-GNU%20AGPL%20V3-green.svg?style=flat)](https://www.gnu.org/licenses/agpl-3.0.en.html) 

# Environment setup
Setup the environment, by reading !(README)[../README.md]

# Overview
This folder contains the code necessary to:

- Parse *_takecare.xlsx* files.
- Run *_takecare.xlsx* files automatic sanity check.
- Load parsed files into SQL database.



### Takecare
All the events occuring in the clinical journal of the recruited patients are labeled according to a  predefined nomenclature.

### *Nomenclature.xlsx*, *Sheet 1*

**Details**

The first sheet contains one category per row.
Each category is defined with three entries.
Each category possess a validity period defined after the three entries.
One can group the different categories.
Each category belongs to group 0 by default.
To create more groups, add columns with the name: "Group *n* - details".
Each category is defined using three entries.
Each entry may include references to several possible words.
This done by matching keywords in the category name with drop down lists names.
The second sheet contains the drop down lists.

**Layout**

Event Category|Event|Specificities|Notes|Date|Beginning|End|Group 0 - population|Group 1 - sepsis
---|---|---|---|---|---|---|---|---
Infection-related event|LOS|Type of Microbe|Culture sample|Time of first culture|-72|24|1|1


### *Nomenclature.xlsx*, *Sheet 2*

**Details**

In the nomenclature above, "Type of Microbe" is a keyword for a drop down list.

**Layout**

|Type of Microbe|
|---|
|unknown|
|unclear|
|rs-virus|

## Takecare
### Parsing a single file (not recommended)
```bash
$ usage: xlsx2csv_takecare.py [-h] -i filename -nom filename

Parse a takecare rawfile. Writes the error log to stderr and the parsed data
to stdout

optional arguments:
  -h, --help     show this help message and exit
  -i filename    Input raw takecare data file
  -nom filename  Input ground truth nomenclature file

$ python takecare/xlsx2csv_takecare.xlsx -i env/1000_takecare.xlsx -nom env/Nomenclature.xlsx > tmp/wharf/1000_takecare.csv 2> tmp/wharf/error.log
```