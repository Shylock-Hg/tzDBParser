use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

extern crate clap;
use clap::{Arg, App};

extern crate csv;

const FORMAT_CPP : &str = "cpp";

struct ZoneArea<'a>(&'a str);

const ZONE_AREAS : [ZoneArea; 12] = [
    ZoneArea("africa"),
    ZoneArea("antarctica"),
    ZoneArea("asia"),
    ZoneArea("australasia"),
    ZoneArea("backward"),  // These data for backward compatible
    ZoneArea("backzone"),  // The Zones that go back beyond the scope of the tz database
    ZoneArea("etcetera"),
    ZoneArea("europe"),
    ZoneArea("northamerica"),
    ZoneArea("pacificnew"),
    ZoneArea("southamerica"),
    ZoneArea("systemv"),
];

struct ZoneInfo {
    name: String,
    abbreviation: String,
    utc_off: i32,  // utc offset in seconds
}

impl ZoneInfo {
    fn new(n : &str, a: &str, u : i32) -> ZoneInfo {
        ZoneInfo {
            name: n.to_string(),
            abbreviation: a.to_string(),
            utc_off: u
        }
    }

    fn to_cpp_structure_literal(&self) -> String {
        "{".to_string() + "\"" + &self.name + "\"" + "," + "\"" + &self.abbreviation + "\"" + "," + &self.utc_off.to_string() + "}"
    }
}

// convert string offset to seconds
fn str_to_offset(s : &str) -> Result<i32, String> {
    let splits : Vec<&str> = s.split(':').collect();
    if splits.len() != 3 {
        return Err(format!("Invalid offset string {}.", s));
    }
    let hour = splits[0].parse::<i32>().unwrap().abs();
    let minute = splits[1].parse::<i32>().unwrap();
    let seconds = splits[2].parse::<i32>().unwrap();
    if splits[0].starts_with('-') {
        Ok(-(seconds + minute * 60 + hour * 60 * 60))
    } else {
        Ok(seconds + minute * 60 + hour * 60 * 60)
    }
}

fn main() {
    let matches = App::new("IANA timezone database transformer")
                            .version("1.0")
                            .author("tcath2s@gmail.com>")
                            .about("Parse and transform the IANA timezone database")
                            .arg(Arg::with_name("tzdb_path")
                                .short("d")
                                .long("tzdb_path")
                                .help("The path of IANA timezone database.")
                                .takes_value(true))
                            .arg(Arg::with_name("csv_path")
                                .short("c")
                                .long("csv_path")
                                .help("The path of boost date_time timezone database in CSV format.")
                                .takes_value(true))
                            .arg(Arg::with_name("format")
                                .help("The output format.")
                                .possible_values(&["cpp"])
                                .takes_value(true)
                                .default_value("cpp"))
                            .get_matches();

    let output_format = matches.value_of("format").unwrap();

    if let Some(tzdb_path) = matches.value_of("tzdb_path") {
        for zone_area in ZONE_AREAS.iter() {
            let path = Path::new(tzdb_path);
            let path_buf = path.join(zone_area.0);
            let f = File::open(path_buf).unwrap();
            let f_reader = BufReader::new(f);
            for line in f_reader.lines() {
                let line = line.unwrap();
                if line.starts_with('#') {
                    continue;
                }
                if line.starts_with("Zone") {
                    // process zone
                }
            }
        }
    } else if let Some(csv_path) = matches.value_of("csv_path") {
        let f = File::open(csv_path).unwrap();
        let mut rdr = csv::ReaderBuilder::new()
                            .has_headers(true)
                            .from_reader(f);
        let mut zones_info = vec![];
        for result in rdr.records() {
            // The iterator yields Result<StringRecord, Error>, so we check the
            // error here.
            let record = result.unwrap();
            let zone_info = ZoneInfo::new(
                record.get(0).unwrap(),
                record.get(1).unwrap(),
                str_to_offset(record.get(5).unwrap()).unwrap(),
            );
            // ignore others
            zones_info.push(zone_info)
        }
        if output_format == FORMAT_CPP {
            println!("{{");
            for zone_info in zones_info {
                println!("{},", zone_info.to_cpp_structure_literal());
            }
            println!("}}");
        }
    } else {
        panic!("Not specify timezone database source, Please check the usage.");
    }
}
