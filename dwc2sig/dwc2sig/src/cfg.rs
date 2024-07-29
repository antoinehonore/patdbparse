
pub use rayon::prelude::*;
pub use rayon::*;
use std::path::PathBuf;

pub struct Opts {
    pub verbose:bool,
    pub summary_fname:String,
    pub mode:String,
    pub date_fmt:String,
    pub lim_line:usize,
    pub export_flag:bool,
    pub output_dir:PathBuf
}


pub fn default_headers(mode:&String) -> Vec<String> {
    let mut h_str:Vec<&str>=vec![];

    if mode == "LF" {
        h_str = vec!["Timestamp", "SequenceNumber", "Value", "Id",
                     "BasePhysioId", "PhysioId", "SubPhysioId",
                     "Label", "SubLabel", "UnitLabel"];
    }
    else if mode == "HF" {
        h_str = vec!["Timestamp",
        "SequenceNumber","WaveSamples","WaveId","BasePhysioId","PhysioId","Label","UnitLabel","UnitCode","Channel","SamplePeriod","IsSlowWave","ScaleLower"
        ,"ScaleUpper","CalibrationScaledLower","CalibrationScaledUpper","CalibrationAbsLower","CalibrationAbsUpper","CalibrationType","EcgLeadPlacement",
        "LowEdgeFrequency","HighEdgeFrequency","IsDerived"]
    }
    else {
        println!("error: unknown mode {}. Exit.", mode);
        std::process::exit(1);
    }

    return h_str.iter().map(|&s| String::from(s)).collect()
}

impl Opts {
    pub fn new(verbose:bool, par:bool, mode:String, lim_line:usize,export_flag:bool,output_dir:PathBuf) -> Opts {
        let ncpus = num_cpus::get_physical();
        rayon::ThreadPoolBuilder::new().num_threads(ncpus).build_global().unwrap();
        return Opts{verbose:verbose,
            summary_fname:String::from(format!("sig_summary_{}.txt",mode)),
            mode: mode,
            date_fmt:String::from("%Y-%m-%d %H:%M:%S.%f"),
            lim_line:lim_line,
            export_flag:export_flag,
            output_dir:output_dir
        }
    }
    pub fn default() -> Opts {
                return Opts{verbose:true,
            summary_fname:String::from("sig_summary_LF.txt"),
            mode: String::from("LF"),
            date_fmt:String::from("%Y-%m-%d %H:%M:%S.%f"),
            lim_line:1_000_000,export_flag:false,output_dir:PathBuf::from("None")}
    }
}
