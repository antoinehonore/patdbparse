use chrono::{NaiveDateTime, Duration,Datelike};
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;

use crate::PatInfo;
use crate::Opts;
use crate::LfFileData;

pub struct Signal {
    pub tl:Vec<NaiveDateTime>,
    pub data:Vec<String>,
    pub dtl:Vec<f64>,
    pub gaps:Vec<f64>
}

pub(crate) fn concat_signals(s_vector:&Vec<&Signal>, gopts:&Opts) -> Signal {
    /*
    From a vector of signals, returns the concatenation of the signals.
       To avoid duplicated: if a sample is found among the 100 previous samples, the new sample is discarded.
    */
    let est_capacity:usize=s_vector.len()*s_vector[0].tl.len();
    let mut out_signal=Signal::with_capacity(est_capacity);
    //if gopts.verbose {println!("info: nfiles {}",s_vector.len())}

    for ifile in 0..s_vector.len() {
        //if gopts.verbose {println!("length of file {}: {}",ifile,s_vector[ifile].tl.len())}

        for isample in 0..s_vector[ifile].tl.len() {
            let is_dupl_tl:bool=sample_is_in_tl(&out_signal.tl,&s_vector[ifile].tl[isample]);
            let is_dupl_data:bool=sample_is_in_data(&out_signal.data,&s_vector[ifile].data[isample]);

            if !(is_dupl_data && is_dupl_tl) {
                //println!("Append: {}, {}",s_vector[ifile].tl[isample],s_vector[ifile].data[isample]);
                out_signal.tl.push(s_vector[ifile].tl[isample]);
                out_signal.data.push(s_vector[ifile].data[isample].clone());
            }
            else {
                //println!("Skip: {}, {}",s_vector[ifile].tl[isample],s_vector[ifile].data[isample]);
            }
            //else {
             //       out_signal.tl.push(s_vector[ifile].tl[isample]);
              //      out_signal.data.push(s_vector[ifile].data[isample].clone());
           // }
        }
    }
    //std::process::exit(1);
    out_signal.shrink_to_fit();
    return out_signal
}


impl Signal {
    pub(crate) fn diff(&mut self) {
        self.dtl=sig_diff(&self.tl);
    }
    pub fn with_capacity(c:usize) -> Signal {
        return Signal{tl:Vec::with_capacity(c),
                     data:Vec::with_capacity(c),
                    dtl:Vec::with_capacity(c),
                    gaps:Vec::with_capacity(c)}
    }
    pub fn shrink_to_fit(&mut self) {
        self.tl.shrink_to_fit();
        self.data.shrink_to_fit();
        self.dtl.shrink_to_fit();
        self.gaps.shrink_to_fit();
    }

    pub fn to_csv(&self, fname:&Path, key:&Duration) {
        // Let mut out_str= String::with_capacity(self.tl.len()*2);
        // Open a file in write-only mode, returns `io::Result<File>`

        let mut file = File::create(fname)
            .expect(format!("couldn't create {}", fname.display()).as_str());

        for i in 0..self.tl.len() {
            file.write(format!("{};{}\n", translate_date(&self.tl[i], key),self.data[i]).as_bytes()).expect("Unable to write.");
        }
    }
}


fn translate_date(d1:&NaiveDateTime, key:&Duration) -> NaiveDateTime {
    if d1.year() > 2030 {
        return *d1 - *key
    }
    else {
        return *d1
    }
}

pub fn find_gaps(v:&Vec<f64>) -> Vec<f64> {
    // Find the gaps larger than 10 minutes
    v.into_iter().filter(|&&p| p>5.).cloned().collect::<Vec<f64>>()
}


pub fn format_gaps(v:&Vec<NaiveDateTime>) -> Vec<(NaiveDateTime,NaiveDateTime,f64)> {
    /* Given
            A vector of NaiveDateTime, compute the successive distances
       Returns
            A vector with the intervals where the length of the interval is larger than 1h
    */
    let mut gaps_vec:Vec<(NaiveDateTime,NaiveDateTime,f64)>=Vec::with_capacity(v.len());
    let mut dt:f64 = 0.;

    for i in 1..v.len() {
        dt= get_n_secs(&v[i-1], &v[1]);
        if dt > 3600. {
            gaps_vec.push((v[i-1],v[1],dt));
        }
    }
    gaps_vec.shrink_to_fit();
    return gaps_vec
}

pub fn format_gap_string(v:&Vec<NaiveDateTime>,patinfo:&PatInfo) -> String {
    /*
    Given
        v: a vector of NaiveDatetime
    Returns
        A formatted string containing the gap information,
            the string format is:   startgap1_endgap1_durationgap1__startgap2_endgap2_durationgap2__
          with the duration in seconds
    */

    let gaps_vec = format_gaps(v);
    let mut out_str= String::with_capacity(200);

    for (d1,d2,dt) in gaps_vec {
        out_str.push_str(format!("{}_{}_{}__",translate_date(&d1,&patinfo.key).to_string(),translate_date(&d2,&patinfo.key).to_string(),dt).as_str());
    }

    return out_str
}

pub fn format_origin_string(s:&Signal) -> String {
    let gaps_vec = format_gaps(&s.tl);
    let mut out_str=String::with_capacity(200);

    for (d1,d2,dt) in gaps_vec {
        out_str.push_str(format!("{}_{}_{}",d1.to_string(),d2.to_string(),dt).as_str());
    }
    return out_str
}



pub(crate) fn sig_diff(v:&Vec<NaiveDateTime>) -> Vec<f64> {
    let mut dv:Vec<f64>=Vec::with_capacity(v.len());
    dv.push(0.);
    for i in 1..v.len() {
        dv.push(get_n_secs(&v[i-1], &v[i]));
    }
    return dv
}

pub(crate) fn secs2days(s:f64) -> f64 {
    return s/60./60./24.
}

pub fn get_n_secs(d1:&NaiveDateTime, d2:&NaiveDateTime) -> f64 {
    return (*d2 - *d1).num_seconds() as f64
}

fn get_n_days(d1:&NaiveDateTime, d2:&NaiveDateTime) -> f64 {
    return secs2days(get_n_secs(d1,d2))
}


fn sample_is_in_data(ref_vec:&Vec<String>, sample:&String) -> bool {
    /* This function is used to find duplicated sample in the tail of a vector to avoid scanning
    the vector completely:

    Given
        a reference vector and a sample
    Returns
        true if the sample is found among the last 100 in the reference vector
        false if not
    */
    let mut j=0;
    for v in ref_vec.iter().rev() {
        //println!("j:{}, v:{}",j,v);
        if j > 100 {
            return false
        }
        if v == sample {
            return true
        }
        j = j + 1;
    }
    return false
}

pub(crate) fn signal_summary_line(s:&Signal, name:&String, patinfo:&PatInfo, pat_mon_id:&String,gopts:&Opts) -> String {
    let mut out_string=String::with_capacity(1024);
    // PATID
    out_string.push_str(pat_mon_id);
    out_string.push_str(";");
    // NAME
    out_string.push_str(name);
    out_string.push_str(";");

    // START
    if s.tl.len() > 0 {out_string.push_str(translate_date(&s.tl[0], &patinfo.key).to_string().as_str());}
    out_string.push_str(";");
    // END
    if s.tl.len() > 0 {out_string.push_str(translate_date(&s.tl[s.tl.len()-1], &patinfo.key).to_string().as_str());}
    out_string.push_str(";");

    // DURATION
    if s.tl.len() > 0 {out_string.push_str(format!("{:.4}",get_n_days(&s.tl[0],&s.tl[s.tl.len()-1])).as_str());}
    out_string.push_str(";");

    if s.tl.len() > 0 {out_string.push_str(format_gap_string(&s.tl, patinfo).as_str());}
    out_string.shrink_to_fit();
    return out_string
}

fn sample_is_in_tl(ref_vec:&Vec<NaiveDateTime>, sample:&NaiveDateTime) -> bool {
    /* This function is used to find duplicated dates in the tail of a vector to avoid scanning
    the vector completely:

    Given
        a reference vector and a sample
    Returns
        true if the sample is found among the last 100 in the reference vector
        false if not
    */
    let mut j=0;
    for v in ref_vec.iter().rev() {
        if j > 100 {
            return false
        }
        if v == sample {
            return true
        }
        j = j + 1;
    }
    return false
}

pub(crate) fn aggregate_sig_stats(data_vec:&Vec<LfFileData>, gopts:&Opts) -> (Vec<String>, Vec<f64>, Vec<f64>) {
    let mut agg_signames:Vec<String>=vec![];
    let mut agg_start:Vec<NaiveDateTime>=vec![];
    let mut agg_end:Vec<NaiveDateTime>=vec![];
    let mut agg_gaps:Vec<f64>=vec![];

    for data in data_vec {
        let mut isig:usize=0;
        for signame in &data.sig_names {
            let index = agg_signames.iter().position(|r| r == signame);
            if index == None {
                agg_signames.push(signame.clone());
                agg_start.push(data.sig_starts[isig]);
                agg_end.push(data.sig_ends[isig]);
                agg_gaps.push(data.sig_gaps[isig]);
            }
            else {
                // the signal found starts before what was saved so far
                if agg_start[index.unwrap()] > data.sig_starts[isig] {
                    agg_start[index.unwrap()] = data.sig_starts[isig]
                }
                if agg_end[index.unwrap()] < data.sig_ends[isig] {
                    agg_end[index.unwrap()] = data.sig_ends[isig]
                }
                agg_gaps[index.unwrap()] = agg_gaps[index.unwrap()] + data.sig_gaps[isig];
            }
            isig=isig+1;
        }
    }

    let agg_durations:Vec<f64>=(0..agg_signames.len()).map(|i| get_n_secs(&agg_start[i], &agg_end[i])).collect();

    return (agg_signames, agg_durations, agg_gaps)
}



#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_sig_diff() {
        let mut v: Vec<NaiveDateTime> = vec![];
        v.push(NaiveDateTime::from_timestamp(1000, 0));
        v.push(NaiveDateTime::from_timestamp(1001, 0));
        v.push(NaiveDateTime::from_timestamp(2000, 0));

        let dv = sig_diff(&v);
        assert_eq!(dv, vec![0.0, 1.0, 999.0])
    }

    #[test]
    fn test_sample_is_in_data() {
        let v:Vec<String> = (0..200).map(|i| i.to_string()).collect();
        assert_eq!(sample_is_in_data(&v,&(198.).to_string()), true)
    }

    #[test]
    fn test_sample_is_in_data2() {
        let v:Vec<String> = (0..200).map(|i| i.to_string()).collect();
        assert_eq!(sample_is_in_data(&v,&(97.).to_string()), false)
    }
}