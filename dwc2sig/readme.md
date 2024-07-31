# Description

This repo aims at creating a summary of `pat../` folders.
The details of these folders organizations is detailed in the repository called `patdb_monout`.

## Summary
The output is a summary_file with headers:
signame,            the signal name
start,              the signal date of start
end,                the signal's date of end
duration_days,      end - start
gap_str, a string detailing the gaps in the signal which where found

### signame
The signal name is formated in `src.parseformat.join_sig_info` as follows:
```
[mode]_[BasePhysioId]_[PhysioId]_[SubPhysioI]_[Label]_[SubLabel]_[UnitLabel]_[BedLabel]_[ClinicalUnit]
```

### gap_str
Portions of signals with more than 1h of missing data are considered gaps.
The gap_str is formated as follows if two gaps longer than 1h are found in a signal:
```
[start of gap1]_[end of gap1]_[duration1 in sec]__[start of gap2]_[end of gap2]_[duration2 in sec]__
```

## Signal formatting
The data corresponding to the same signals are aggregated into vectors.

### Parsing
The time shift introduced at export time is corrected to obtain the real dates.

### Compression
The HF data of each row are compressed before being appended to the signal's vector.
This is done in `src.parseformat`.

### Write
The signame detailed above is used as a filename for the signal data.

# Build

 - Install the Rust binaries: https://www.rust-lang.org/learn/get-started  
 - You can run the tests with:
```
cargo test
```
 - Build the tool in release mode: 
 ```bash
cargo build --release
```

 - The binary file is `target/release/patdb_bin`

# Usage

## Requirements
The software expects `PatientsMapping.txt.gpg` or `PatientsMapping.txt` to be in the parent folder of the `pat../` folders.  
 
## Examples
- To summarize the LF data contained in the `data/pat1111` folder, run:
```
./patdb_bin --verbose summarize -i data/pat1111 -m LF
```
The summary file will be created in the data folder.

- To summarize and export the formatted signal data, run:
```
./patdb_bin --verbose summarize -i data/pat1111 -m LF -o out/data/folder/
```
the signal data will be saved into `out/data/folder/data_pat1111` and the summary file will be saved in `data/pat1111`

- You can specify several folders at once:
```
./patdb_bin --verbose summarize -i data/pat1111 data/pat1112 -m LF -o out/data/folder/
```
the signal data will be saved into `out/data/folder/data_pat1111` and `out/data/folder/data_pat1112`  

