use std::str::FromStr;
use std::fs::File;
use std::io::{stdout, stderr};
use std::io::prelude::*;
use std::{io, fs};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

extern crate argparse;
extern crate rayon;


use argparse::{ArgumentParser, StoreTrue, Store, List};
use glob::glob;
use chrono::{NaiveDateTime, Duration};

mod readwrite;
use readwrite::*;

mod parseformat;
use parseformat::*;

mod signal;
use signal::*;

mod cfg;
use cfg::*;

use std::time::SystemTime;

#[allow(non_camel_case_types)]
#[derive(Debug)]
enum Command {
    summarize,
    record,
}

impl FromStr for Command {
    type Err = ();
    fn from_str(src: &str) -> Result<Command, ()> {
        return match src {
            "summarize" => Ok(Command::summarize),
            "record" => Ok(Command::record),
            _ => Err(()),
        };
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_infer_pat_monid() {
        let mut p = PathBuf::new();
        p.push(Path::new("the"));
        p.push(Path::new("path"));
        p.push(Path::new("data_monitor_12ts34"));
        p.push(Path::new("pat12"));
        p.push(Path::new("*LF*.csv*"));
        assert_eq!(infer_pat_monid(&p), (String::from("data_monitor_12ts34_pat12"), 12))
    }

    #[test]
    fn test_format_find_file_ord_key() {
        let fname=PathBuf::from("pat1125/pat1125_export_LF_start_282120411216672_next_500000.csv");
        assert_eq!(find_file_ord_key(&fname), 282120411216672)
    }
}

fn find_file_ord_key(p:&PathBuf) -> u128 {
    // to avoid sorting the whole patient data, we pre-order the minibatch files. 
    // This function finds the ordering key from a, 
    //    In previous data files version, the key was the first  sequence number in the file, 
    //    In recent versions it is a date formated as "YYYY-mm-dd-HH-MM-SS", by removing the "-" and converting to u128, this can be used to sort the files.

    let s:Vec<&str>= p.file_stem().unwrap().to_str().unwrap().split("_").collect();
    //println!("{:?}, {}", &(s[4])[..19],(&s[4][..19]).replace("-","").parse::<u128>().unwrap());
    return (&s[4][..19]).replace("-","").parse::<u128>().unwrap()
}

fn parse_date(s: &String, date_fmt: &String) -> NaiveDateTime {
    let thedate = NaiveDateTime::parse_from_str(s.as_str(), date_fmt);
    
    if !thedate.is_ok() {
        //println!("Cannot parse: {:?} with fmt: {:?}", s, date_fmt);
        let mut new_datefmt = String::from(date_fmt.as_str());
        new_datefmt.push_str(" %#z");

        let thedate = NaiveDateTime::parse_from_str(s.as_str(), new_datefmt.as_str()).expect(format!("Cannot parse: {:?} with fmt: {:?}", s, new_datefmt).as_str());
        return thedate
    }
    else {
        return thedate.unwrap()
    }
}

fn parse_data_file(fname:&PathBuf, i:Arc<Mutex<usize>>, nfiles:&usize, gopts:&Opts) -> Option<LfFileData> {
    let d_str:String= readwrite::read_mon_data(fname);

    let (signames, signals)=parse_data_string(&d_str,gopts);
    if signames[0] != String::from("dummy") {
        let all_starts = signals.iter().map(|sig| *sig.tl.first().unwrap()).collect();
        let all_ends = signals.iter().map(|sig| *sig.tl.last().unwrap()).collect();
        let all_gaps = signals.iter().map(|sig| vec_sum(&sig.gaps)).collect();
        let nsigs = &signames.len();

        if gopts.verbose {
            let mut file_counter = i.lock().unwrap();
            *file_counter += 1;
            println!("done: {}/{}", file_counter, nfiles);
        }

        return Some(LfFileData {
            fname: fname.to_str().unwrap().to_string(),
            sig_names: signames,
            signals: signals,
            sig_starts: all_starts,
            sig_ends: all_ends,
            sig_nb: *nsigs,
            sig_gaps: all_gaps
            })
    }
    else {
        return None
    }
}


fn vec_sum(v:&Vec<f64>) -> f64 {
    let mut sum:f64=0.;
    for vv in v {
        sum = sum + *vv;
    }
    return sum
}

struct LfData {
    data:Vec<LfFileData>
}

pub struct LfFileData {
    pub fname:String,
    pub sig_names:Vec<String>,
    pub signals:Vec<Signal>,
    pub sig_starts:Vec<NaiveDateTime>,
    pub sig_ends:Vec<NaiveDateTime>,
    pub sig_gaps:Vec<f64>,
    pub sig_nb:usize
}


fn find_modified_date(fname:&Path) -> Option<NaiveDateTime> {
    if fname.exists() {
        Some(NaiveDateTime::from_timestamp(
            fs::metadata(fname).unwrap()
                .modified().unwrap()
                .duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64
            , 0))
    }
    else {
        None
    }
}

fn is_newer(ref_date:NaiveDateTime, most_recent_date:NaiveDateTime) -> bool {
    most_recent_date > ref_date
}

pub struct PatInfo {
    pub pn:String,
    pub key:Duration,
    pub idx:usize
}

impl PatInfo {
    fn new(pn:String, key:i64, folder:String) -> PatInfo {
        //println!("{}",pn);
        return PatInfo{pn:pn, key:Duration::days(key), idx:pat_dir2idx(&folder)}
    }
    fn to_string(&self) -> String {
        String::from(format!("idx: {}, pn: {}, key: {:?}", self.idx, self.pn, self.key.num_days()))
    }
}

fn build_out_path(output_dir:&Path, pat_mon_id:&String, fname:&String) -> PathBuf {
    return output_dir
            .join(Path::new(pat_mon_id.as_str()))
            .join(Path::new(fname.as_str()))
}

fn pat_folder_data(input_dir:&String, gopts:&Opts) -> Option<PatMon> {
    /* Given
            an input patient data directory, reads, parses and summarizes the LF data.

       Returns,
            None if the summary file already exists and if it is newer than the most recent datafile
            A PatMon instance containing the summarized data
     */

    let input_path:&Path = Path::new(input_dir);
    let verbose= gopts.verbose;
    let export_flag= &gopts.export_flag;
    let output_dir = &gopts.output_dir;

    if !input_path.is_dir() {
        println!("error: Path not found: {}", input_path.display());
        std::process::exit(1)
    }

    // Find the patient datafiles
    // define the path
    let mut pattern:PathBuf=PathBuf::new();
    pattern.push(&input_path);
    let mut  out_file= pattern.clone();
    out_file.push(&gopts.summary_fname);

    //  Find timestamp of the output file
    let T_modified_output= find_modified_date(&out_file);


    // Define the wildcard
    let mut wildcard=String::new();
    wildcard.push_str("*");
    wildcard.push_str(gopts.mode.as_str());
    wildcard.push_str("*.csv*");

    pattern.push(wildcard);

    if verbose { println!("[info] Looking for {}", pattern.to_str().unwrap()); }

    // Find all data file
    let mut pat_data_file_vec:Vec<PathBuf> = glob(pattern.to_str().unwrap()).unwrap().filter_map(Result::ok).collect();
    // sort them
    pat_data_file_vec.sort_by_cached_key(|s| find_file_ord_key(s));

    // Find when they were last modified
    let pat_data_modif:Vec<NaiveDateTime> = pat_data_file_vec.iter().map(|f| find_modified_date(f).unwrap()).collect();

    // Determine whether or not we should run based on the timestamps
    let mut run_flag:bool = false;

    match T_modified_output {
        Some(t_modified_output) => {
            let d_most_recent:NaiveDateTime=*pat_data_modif.iter().max().unwrap();
            if is_newer(t_modified_output, d_most_recent) {
                run_flag = true;
            } else {
                if gopts.verbose { println!("[info] Summary file ({}) is more recent: summary {} vs datafile {}",out_file.display(),t_modified_output,d_most_recent) }
            }
        }

        None => run_flag=true
    };

    if run_flag {

            // Read the map file
            let mapfile_fname= infer_map_fname(&input_path);
            if verbose { println!("[info] mapfile name {}", mapfile_fname.display())}
            let mapfile_str= read_mapfile(&mapfile_fname);
            //println!("patinfo: {}", mapfile_str);
            // Parse
            let mapfile_data= parse_mapfile(&mapfile_str);

            // pat_mon_id is the monitor_id, idx is the row number in the mapfile
            let (pat_mon_id, idx)= infer_pat_monid(&pattern);
            //println!("pat[info] {}, idx {}",pat_mon_id, idx);
            let patinfo:&PatInfo = &mapfile_data[idx-1];
            if verbose {println!("patinfo: {}", patinfo.to_string());}

            let nfiles=pat_data_file_vec.len();
            let theiterator:Vec<usize>=(0..nfiles).collect();
            let finished_files_count:Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

            // Parse all the signals from the datafiles
            let data_vec_opt: Vec<Option<LfFileData>> =
                theiterator.par_iter().map(|&i| parse_data_file(&pat_data_file_vec[i],
                                                                Arc::clone(&finished_files_count),
                                                                &nfiles, gopts)).collect();
            // https://stackoverflow.com/questions/36020110/how-do-i-avoid-unwrap-when-converting-a-vector-of-options-or-results-to-only-the
            let data_vec:Vec<LfFileData> = data_vec_opt.into_iter().flatten().collect();
            if data_vec.len() > 0 {
                // Format the signals
                let mut all_sig_names:Vec<String>=Vec::with_capacity(100);
                let mut all_sig_vectors:Vec<Vec<&Signal>>=Vec::with_capacity(100);

                for fdata in &data_vec {
                    for i_file_sig in 0..fdata.signals.len() {
                        let sig_name=&fdata.sig_names[i_file_sig];
                        if !(all_sig_names.contains(sig_name)) {
                            all_sig_names.push(sig_name.clone());
                            let mut sig_vec:Vec<&Signal>=Vec::with_capacity(100);
                            sig_vec.push(&fdata.signals[i_file_sig]);
                            all_sig_vectors.push(sig_vec);
                        }
                        else {
                            let sig_idx:usize = all_sig_names.iter().position(|r| r == sig_name).unwrap();
                            all_sig_vectors[sig_idx].push(&fdata.signals[i_file_sig])
                        }
                    }
                }

                let mut all_sig:Vec<Signal>=Vec::with_capacity(100);
                for isig in 0..all_sig_vectors.len() {
                    all_sig.push(concat_signals(&all_sig_vectors[isig], gopts));
                }
                all_sig.shrink_to_fit();

                let mut summary_str=String::with_capacity(1024);
                let out_dir= Path::new(output_dir).join(&pat_mon_id);
                if *export_flag {
                    match fs::create_dir_all(&out_dir) {
                        Err(_) => if verbose { println!("[info] problem creating directory {}", out_dir.display()) },
                        _ => ()
                    }
                }
                for isig in 0..all_sig.len() {
                    let mut fname= all_sig_names[isig].clone();
                    fname.replace(|c: char| !c.is_ascii(), "_");
                    fname.push_str(".csv");

                    summary_str.push_str(signal_summary_line(&all_sig[isig],&all_sig_names[isig],&patinfo, &pat_mon_id,gopts).as_str());
                    summary_str.push_str("\n");

                    if *export_flag {
                        let thepath:PathBuf = out_dir.join(Path::new(fname.as_str()));
                        all_sig[isig].to_csv(&thepath.as_path(), &patinfo.key);
                    }
                }
                summary_str.shrink_to_fit();

                let pat = PatMon {
                    id: pat_mon_id,
                    sigs: all_sig_names,
                    data_folder: input_dir.clone(),
                    data_vec: data_vec,
                    summary_string:summary_str
                };
                return Some(pat)
            }
            else {
                Some(PatMon{
                    id: pat_mon_id,
                    sigs: vec![String::from("empty")],
                    data_folder: input_dir.clone(), data_vec: data_vec,
                summary_string:String::from(";;;;;")
                })
            }
        }
    else { return None }
}

pub struct PatMon {
    pub id:String,
    pub data_folder:String,
    pub sigs:Vec<String>,
    pub data_vec:Vec<LfFileData>,
    pub summary_string:String
}

fn summarize_command(verbose: bool, args: Vec<String>)-> io::Result<()> {
    // Parse options
    let mut input_dir:Vec<String> = vec![];
    let mut mode:String = String::new();
    let mut output_dir:String = String::from("None");
    let mut debug_mode:bool=false;

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Summarizes the data in input directory");
        ap.refer(&mut input_dir)
            .add_option(&["-i", "--input"], List,
                        r#"Input directory to summarize"#);
        ap.refer(&mut mode)
            .add_option(&["-m", "--mode"], Store,
                        r#"Data type to summarize (HF or LF)"#);
        ap.refer(&mut output_dir)
            .add_option(&["-o","--output"], Store,
                        r#"output folder"#);
        ap.refer(&mut debug_mode)
            .add_option(&["--debug"], StoreTrue,
                        r#"Debug mode"#);

        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => {
                std::process::exit(x);
            }
        }
    }

    let mut export_flag:bool=false;

    if output_dir != String::from("None") {
        export_flag=true;
        if verbose {
            println!("[info] export to {}", output_dir);
        }
    }

    if verbose {
        println!("[info] export flag is {:?}", export_flag);
    }
    let mut lim_line=1_000_000;
    if debug_mode {
        if verbose {println!("[info] debug mode");}
        lim_line=5_000;
    }

    let gopts = &Opts::new(verbose, true, mode, lim_line, export_flag,PathBuf::from(output_dir));

    const SEP:&char = &';';

     // Read, parse and summarize data in parallel
     let pats_data:Vec<Option<PatMon>> = input_dir.par_iter().map(|p| pat_folder_data(&p, &gopts)).collect();

     // Format & Write to file
     for pat_opt in &pats_data {
         match pat_opt {
             Some(pat) => {

                 let out_headers= format!("monid;signame;start;end;duration_days;gap_str");
                 let out_str= &pat.summary_string;

                 let mut out_file = PathBuf::new();
                 out_file.push(&pat.data_folder);
                 out_file.push(format!("{}", &gopts.summary_fname));

                 let mut file = File::create(out_file)?;
                 file.write(out_headers.as_ref())?;
                 file.write("\n".as_ref())?;
                 file.write(out_str.as_ref())?;

                 if verbose {
                     println!("{}", out_headers);
                     println!("{}", out_str);
                     println!("");
                 }
             }
             None => if verbose {println!("[info] nothing to do. Exit.")}
         }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let mut verbose = false;
    let mut subcommand = Command::summarize;
    let mut args = vec!();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Plays or records sound");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue,
            "Be verbose");
        ap.refer(&mut subcommand).required()
            .add_argument("command", Store,
                r#"Command to run (either "summarize")"#);
        ap.refer(&mut args)
            .add_argument("arguments", List,
                r#"Arguments for command"#);
        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }

    //let ncpus = num_cpus::get_physical();
    //rayon::ThreadPoolBuilder::new().num_threads(ncpus).build_global().unwrap();

    args.insert(0, format!("subcommand {:?}", subcommand));
    match subcommand {
        Command::summarize => summarize_command(verbose, args),
        _ => Ok(())
    }
}