use crate::{PatInfo,Opts, Signal,default_headers,parse_date,find_gaps,vec_sum};
use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;

//use std::io::prelude::*;
use std::io::Write;
use base64;

extern crate unidecode;
use unidecode::unidecode;

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub fn join_sig_info(line_split:&Vec<String>, h:&Vec<String>, gopts:&Opts) -> String {
    let mut out_str=String::with_capacity(100);
    let mut i=0;
    let mut found_labels:bool = false;
    //println!("ncol:{}, nheaders:{}", line_split.len(),h.len());
    for f in line_split {
        if  //h[i].starts_with("Id") ||
            h[i].contains("PhysioId") ||
            h[i].starts_with("Sub") || h[i].ends_with("UnitLabel") ||
            h[i].starts_with("Label") {
                out_str.push_str(f.as_str());
                out_str.push('_');
                out_str.push('_');
        }
        if h[i].contains("BedLabel") || h[i].contains("ClinicalUnit") {
            out_str.push_str(f.as_str());
            out_str.push('_');
            out_str.push('_');
            found_labels=true;
        }
        i=i+1;
    }
    if !found_labels {
        out_str.push_str("unknown__unknown__");
    }

    out_str.pop();
    out_str.pop();
    out_str=format!("{}__{}", gopts.mode, out_str);

    return String::from(unidecode(out_str
        .replace(" ","-").replace("(","_")
        .replace(")","_").replace("?","-").replace("/", "per")
        .as_str()));
}

pub fn parse_mapfile(mapfile_str:&String) -> Vec<PatInfo> {
    let nlines = mapfile_str.lines().count();
    //println!("patinfo: {}", nlines);
    let mut out:Vec<PatInfo>=Vec::with_capacity(nlines);
    let mut i:usize = 0;
    for line in mapfile_str.lines() {
        
        if i > 0 {
            let thepatinfo=parse_mapfile_line(String::from(line));
            //println!("patinfo: {}", line);

            out.push(thepatinfo);

        }
        else {
            i = 1;
        }
    }
    return out
}


fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}


pub fn parse_data_string(data_str:&String, gopts:&Opts) -> (Vec<String>,Vec<Signal>) {
    let lines= data_str.lines();
    let nlines:usize = lines.count();

    //if gopts.verbose && nlines < 10 {println!("nlines: {}", nlines); println!("lines:\n {}", data_str)}
    let mut i:usize=0;
    let mut headers:Vec<String>=vec![];
    let mut all_sigs:Vec<Signal>=vec![];
    let mut all_sigs_names:Vec<String>=vec![];
    let mut col_offset:usize = 0;

    // file too short
    if nlines < 10 {
        all_sigs_names.push(String::from("dummy"));
        all_sigs.push(Signal{ tl:Vec::with_capacity(1),
                                            data:Vec::with_capacity(1),
                                            dtl:vec![],
                                            gaps:vec![]})
    }
    else {
        for l in data_str.lines() {
            if i==0 {
                if l.contains("Timestamp") {
                    headers=l.split(";").map(|s| String::from(s)).collect();
                    if headers[0].contains("PSComputerName") {
                        col_offset = 3;
                    }
                }
                else {
                    headers=default_headers(&gopts.mode);
                }
            }

            else {
                let line_split:Vec<String>=l.split(";").skip(col_offset).map(|p| String::from(p)).collect();//;
                //println!("{} {:?}", col_offset, line_split);

                let thesigname= join_sig_info(&line_split,&headers,gopts);

                // Find the index of the signal
                let index = all_sigs_names.iter().position(|r| r == &thesigname);

                let mut sig_index:usize;

                // If not found
                if index == None {
                    all_sigs.push(Signal{ tl:Vec::with_capacity(nlines),
                                                data:Vec::with_capacity(nlines),
                                                dtl:vec![],
                                                gaps:vec![]}
                    );
                    all_sigs_names.push(thesigname);
                    sig_index=all_sigs.len()-1;
                }
                else {
                    sig_index=index.unwrap();
                }

                all_sigs[sig_index].tl.push(parse_date(&line_split[0], &gopts.date_fmt));

                if  gopts.mode.as_str() == "HF" { // Reduce the size of the data string
                    all_sigs[sig_index].data.push(compress_hex(rescale(&line_split)).to_string());
                }
                else {
                    all_sigs[sig_index].data.push(line_split[2].clone().replace(",","."));
                }
            }
            i = i + 1;

            // Temporary limit
            if i > gopts.lim_line {
                break;
            }
        }

        for s in &mut all_sigs {
            s.diff();
            let tmp= find_gaps(&s.dtl);
            s.gaps.push(vec_sum(&tmp));
        }
    }

    return (all_sigs_names, all_sigs)

}

fn cal_values(v:&Vec<String>) -> (f32,f32,i32) {
    let CSL:f32=v[14].parse::<f32>().unwrap();
    let CSU:f32=v[15].parse::<f32>().unwrap();
    let CAL:Result<f32,ParseFloatError>=v[16].replace(",",".").parse::<f32>();
    let CAU:Result<f32,ParseFloatError>=v[17].replace(",",".").parse::<f32>();
    let Te:i32=v[10].parse::<i32>().unwrap();

    let mut m:f32=1.;
    let mut b:f32=0.;

    match CAL {
        Ok(cal) => {
            match CAU {
                Ok(cau) => {
                    m=(cau-cal)/(CSU-CSL);
                    b=cau-m*CSU;
                },
                Err(_) =>()
            }
        },
        Err(_) => ()
    }
    return (m,b,Te)
}

fn rescale(v:&Vec<String>) -> String {
    let mut o:Vec<String> = Vec::with_capacity(v[2].len()/4);
    let (m,b,Te)=cal_values(&v);
    //println!("m {}, b {}",m,b);
    //

    //println!("{}",slice);
    let mut z: f32;
    for i in (2..v[2].len()).step_by(4) {
        let slice = &v[2][i..(i+4)];
        z = i64::from_str_radix(&format!("{}{}",&slice[2..4],&slice[..2]).to_string(),16).unwrap() as f32;
        o.push(format!("{}",m*z+b))
    }
    //println!("{}",format!("{};{}",Te,o.join(";")) );
    return format!("{};{}",Te,o.join(";"))
}

use std::i64;
use std::num::ParseFloatError;

fn hex2int(s:String) -> i32 {
    //let z = i64::from_str_radix(s.substring(2,4) + s.substring(0,2), 16);
    return 0
}

fn parse_mapfile_line(s:String) -> PatInfo {
    let v:Vec<&str> = s.split(";").collect();
    //println!("pn:{}:   {}, |{}| , {:?}",v[1],format_pn(&String::from(v[1])),v[2],"11451".parse::<i64>());
    let out = PatInfo::new(format_pn(&String::from(v[1])),
                 v[2].parse::<i64>().unwrap(),
                 String::from(v[0]));
    //
    return out
}

fn format_pn(s_:&String) -> String {
    let s = s_.replace(" ", "").to_string();
    let mut out=String::with_capacity(13);
    out.push_str(s.as_str());

    // Direct exclusion
    if s.len() < 10 || s.contains(".") {
        return out
    }

    let idx:usize = out.find('-').unwrap_or(999);

    if s.starts_with("99") {
        if idx == 6 {
            return out
        }
        else if idx == 999 {
            out.insert(6,'-');
            return out
        }
        else {
            return out
        }
    }
    else {
        if idx == 4 { //2020-121212 or 1930-121212
            out.insert_str(0,"99");
            return out
        }
        else if idx == 9 { //20201212-1212
            return out
        }
        else if idx == 999 {//202012121212
            if out.len() == 12 {
                out.insert(8, '-');
                return out
            }
            else if out.len() == 10 { // 2012121212
                let mut out2=out.clone();
                out2.truncate(2);
                let first_two_digits=
                    match out2.parse::<i32>() {
                        Ok(n) => n,
                        Err(_) => return out
                };
                //
                if first_two_digits > 20 {
                    out.insert_str(0,"19");
                }
                else {
                    out.insert_str(0,"20");
                }

                if !out.contains("-") {// make sure: 2012121212
                    out.insert(8,'-');
                    return out
                }
                else { // Strange
                    return out
                }
            }
        }
        else { // Strange
            return out
        }
    }
    return out
}

fn parse_value(line_split:&Vec<String>) -> f32 {
    if !(line_split[2].starts_with("0x")) {
        return line_split[2].replace(",",".").parse::<f32>().unwrap()
    }
    else {
        return 1.
    }
}

fn encode_base64(c:Vec<u8>) -> String {
    let mut wrapped_writer = Vec::new();
    {
        let mut enc = base64::write::EncoderWriter::new(&mut wrapped_writer, base64::STANDARD);

        // handle errors as you normally would
        enc.write_all(c.as_slice()).unwrap();
        // could leave this out to be called by Drop, if you don't care
        // about handling errors
        enc.finish().unwrap();
    }
    return String::from_utf8(Vec::from(&wrapped_writer[..])).unwrap();
}

fn decode_base64(s:String) -> Vec<u8> {
    let bytes = base64::decode(s.as_str()).unwrap();
    return bytes
}

fn compress_hex(s:String) -> String {
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(s.as_bytes());
    encode_base64(e.finish().unwrap())
}

fn decompress_hex(s_in_b64:String) -> String {
    let s_in=decode_base64(s_in_b64).clone();
    let mut d = ZlibDecoder::new(s_in.as_slice());
    let mut s_out = String::new();
    d.read_to_string(&mut s_out).unwrap();
    s_out
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_join_sig_info1() {
        let mut h = default_headers(&String::from("LF"));
        h.push(String::from("BedLabel"));
        h.push(String::from("ClinicalUnit"));

        let line_split: Vec<String> = vec!["thetime", "theseqid", "thevalue", "1", "2", "3", "4", "5", "6", "7", "bed1", "unitb"]
            .iter()
            .map(|&p| String::from(p))
            .collect();
        let gopts=&Opts::default();

        assert_eq!(join_sig_info(&line_split, &h,gopts).as_str(), "LF__2__3__4__5__6__7__bed1__unitb");
    }

    #[test]
    fn test_join_sig_info2() {
        let mut h = default_headers(&String::from("LF"));
        let line_split: Vec<String> = vec!["thetime", "theseqid", "thevalue", "1", "2", "3", "4", "5", "6", "7"]
            .iter()
            .map(|&p| String::from(p))
            .collect();
        let gopts=&Opts::default();

        assert_eq!(join_sig_info(&line_split, &h,gopts).as_str(), "LF__2__3__4__5__6__7__unknown__unknown");
    }

    #[test]
    fn test_format_pn1() {
        let in1 = String::from("992020-010101");
        assert_eq!(format_pn(&String::from("992020-010101")), String::from("992020-010101"))
    }

    #[test]
    fn test_format_pn2() {
        assert_eq!(format_pn(&String::from("2020-010101")), String::from("992020-010101"))
    }
    #[test]
    fn test_format_pn3() {
        assert_eq!(format_pn(&String::from("20201212-0101")), String::from("20201212-0101"))
    }
    #[test]
    fn test_format_pn4() {
        assert_eq!(format_pn(&String::from("202012120101")), String::from("20201212-0101"))
    }
    #[test]
    fn test_format_pn5() {
        assert_eq!(format_pn(&String::from("nawak")), String::from("nawak"))
    }
    #[test]
    fn test_format_pn6() {
        assert_eq!(format_pn(&String::from("2012121212")), String::from("20201212-1212"))
    }
    #[test]
    fn test_format_pn7() {
        assert_eq!(format_pn(&String::from("2112121212")), String::from("19211212-1212"))
    }
    #[test]
    fn test_format_pn8() {
        assert_eq!(format_pn(&String::from("21  12121  212")), String::from("19211212-1212"))
    }
    #[test]
    fn test_compression() {
        let s_in = String::from("0x21080308E007B707870754072207F606D70621080308E007B707870754072207F606D70621080308E007B707870754072207F606D706");
        let c_s = compress_hex(s_in.clone());
        let s_out = decompress_hex(c_s);
        assert_eq!(s_out,s_in);
    }

    #[test]
    fn test_enc_dec_b64() {
        let s_in = String::from("hello world");

        let c_s_b64=encode_base64(Vec::from(s_in.as_bytes()));
        let c_v=decode_base64(c_s_b64);

        let s_out = String::from_utf8(c_v).unwrap();
        assert_eq!(s_out,s_in);
    }
}