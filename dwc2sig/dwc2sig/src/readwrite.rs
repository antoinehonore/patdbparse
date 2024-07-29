use std::process::{Command as syscommand};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::prelude::*;


fn read_encrypted(fpath:&PathBuf) -> String {
    /*
        Given the path of an pgp-encrypted utf8 encoded file, decrypt using a syscall to gpg.
        The syscall reads the passphrase using an absolute path to a file.
     */
    let output = syscommand::new("/usr/bin/gpg")
                 .arg("-d")
                 .arg("--passphrase-file")
                 .arg("/opt/psql/gpg_antoine_pfile.txt")
                 .arg("--batch")
                 .arg("--yes")
                 .arg(fpath.to_str().unwrap())
                 .output()
                 .expect("failed to execute process");

    //println!("{}", String::from_utf8_lossy(&output.stderr));
    let out_str= String::from_utf8_lossy(&output.stdout).into_owned();
    //println!("{}",out_str);
    return out_str
}

fn read_plain(fpath:&PathBuf) -> String {
    /*
    Given a filepath to a plain utf8 encoded file,
    Returns the file content as a String.
     */

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(fpath) {
        Err(why) => panic!("couldn't open {}: {}", fpath.display(), why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", fpath.display(), why),
        Ok(_) => s,
    }
}

pub fn read_mon_data(fpath:&PathBuf) -> String {
    /* Given a filepath to an encrypted or not utf8 encoded file,
    Returns the file content as a String. */
    let fname:&str = fpath.file_name().unwrap().to_str().unwrap();
    let out:String;
    if is_encrypted(fname) {
        out=read_encrypted(fpath);
    }
    else {
        out=read_plain(fpath);
    }
    let s:String = out.replace("\"","");
    return s
}


pub fn read_mapfile(map_fname:&PathBuf) -> String {

    read_encrypted(map_fname)
}


fn is_encrypted(fname:&str) -> bool {
    /*
    Given a filename, determine if the file is encrypted by checking if it contains ".gpg"
     */
    fname.contains(".gpg")
}


pub fn infer_map_fname(thepath:&Path) -> PathBuf {
    /*
    Given the input directory, infer the location of the mapfile.
     */
    let thedir=thepath.parent().unwrap();
    let thestem=thepath.file_name().unwrap().to_str().unwrap();
    let mapfile_stem:String=String::from("PatientsMapping.txt.gpg");

    if thestem.starts_with("pat") {
        return thedir.join(mapfile_stem);
    }
    else {
        return thepath.join(mapfile_stem);
    }
}

pub(crate) fn pat_dir2idx(s:&String) -> usize {
    return s.replace("pat","").parse::<usize>().unwrap()
}

pub(crate) fn infer_pat_monid(f_pattern:&PathBuf) -> (String,usize) {
    /*
    Given
        the pattern to a data folder, for instance: path/to/data_monitor/pat13
    Returns
        a tuple with
            the monitor_id (data_monitor_pat13:String)
            the index in map (13:usize)
    */

    let mut out=String::with_capacity(100);
    let pat_dir:PathBuf=f_pattern.as_path().parent().unwrap().canonicalize().unwrap();

    out.push_str(pat_dir.parent().unwrap().file_stem().unwrap().to_str().unwrap());

    out.push('_');
    let pat_dir_str=pat_dir.file_stem().unwrap().to_str().unwrap();
    out.push_str(pat_dir_str);
    out.shrink_to_fit();

    return (out, pat_dir2idx(&String::from(pat_dir_str)));
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_infer_mapfilename() {
        let map_fname=infer_map_fname(Path::new("thedata/data_monitor/pat10"));
        assert_eq!(map_fname,Path::new("thedata/data_monitor/PatientsMapping.txt.gpg"))
    }
    #[test]
    fn test_infer_mapfilename2() {
        let map_fname=infer_map_fname(Path::new("thedata/data_monitor"));
        assert_eq!(map_fname,Path::new("thedata/data_monitor/PatientsMapping.txt.gpg"))
    }

    #[test]
    fn test_read_mapfile() {
        let map_fname = infer_map_fname(Path::new("thedata/data_monitor"));
        let output = syscommand::new("/usr/bin/gpg")
         .arg("-d")
         .arg("--passphrase-file")
         .arg("/home/anthon@ad.cmm.se/pfile.txt")
         .arg("/mnt/cmm-share/Private/patients_data/data_monitor/PatientsMapping.txt.gpg")
         .output()
         .expect("failed to execute process");
        //let buf=String::from_utf8_lossy(&output.stdout);

        let  p= String::from_utf8_lossy(&output.stdout);
        //println!("{}",p);
    }

    fn test_encrypted1() {
        assert_eq!(is_encrypted("file.csv.gpg"), true);
    }

    #[test]
    fn test_encrypted2() {
        assert_eq!(is_encrypted("file.csv"), false);
    }

    #[test]
    fn test_encrypted3() {
        let fpath = Path::new("tests/test_enc_file.csv.gpg");
        let mut fpathbuf = PathBuf::new();
        fpathbuf.push(fpath);
        let dec_str = read_encrypted(&fpathbuf);
        //println!("stdout: {}",dec_str);
        //println!("stderr: {}",String::from_utf8_lossy(&output.stderr));
        assert_eq!(dec_str, String::from(";thetestsecret;;\n"))
    }
}
