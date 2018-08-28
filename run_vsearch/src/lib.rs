extern crate clap;
extern crate walkdir;

use clap::{App, Arg};
use std::error::Error;
//use std::process::{Command, Stdio};
use std::{
    env, fs::{self, DirBuilder, File}, io::Write, path::{Path, PathBuf},
};
use walkdir::WalkDir;

// --------------------------------------------------
#[derive(Debug)]
pub struct Config {
    action: String,
    input: String,
    centroids: Option<String>,
    db_file: Option<String>,
    id_value: Option<String>,
    fastq_ascii: Option<String>,
    bin_dir: Option<String>,
    out_dir: PathBuf,
}

// --------------------------------------------------
type MyResult<T> = Result<T, Box<Error>>;

// --------------------------------------------------
pub fn get_args() -> MyResult<Config> {
    let matches = App::new("VSEARCH")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@email.arizona.edu")
        .about("Run VSEARCH")
        .arg(
            Arg::with_name("query")
                .short("q")
                .long("query")
                .value_name("FILE_OR_DIR")
                .help("File or input directory")
                .required(true)
                .min_values(1),
        )
        .arg(
            Arg::with_name("centroids")
                .short("e")
                .long("centroids")
                .value_name("FILE")
                .help("Centroids"),
        )
        .arg(
            Arg::with_name("db_file")
                .short("d")
                .long("db")
                .value_name("FILE")
                .help("Database"),
        )
        .arg(
            Arg::with_name("out_dir")
                .short("o")
                .long("out_dir")
                .value_name("DIR")
                .help("Output directory"),
        )
        .arg(
            Arg::with_name("command")
                .short("c")
                .long("command")
                .value_name("FILE")
                .help("Aliases for sample names"),
        )
        .arg(
            Arg::with_name("id_value")
                .short("i")
                .long("id")
                .value_name("INT")
                .default_value("")
                .help("ID value"),
        )
        .arg(
            Arg::with_name("fastq_ascii")
                .short("f")
                .long("fastq_ascii")
                .value_name("INT")
                .default_value("")
                .help("ID value"),
        )
        .arg(
            Arg::with_name("bin_dir")
                .short("b")
                .long("bin_dir")
                .value_name("DIR")
                .help("Location of binaries"),
        )
        .get_matches();

    let out_dir = match matches.value_of("out_dir") {
        Some(x) => PathBuf::from(x),
        //_ => None,
        _ => {
            let cwd = env::current_dir()?;
            cwd.join(PathBuf::from("mash-out"))
        }
    };

    let alias = match matches.value_of("alias") {
        Some(x) => Some(x.to_string()),
        _ => None,
    };

    let bin_dir = match matches.value_of("bin_dir") {
        Some(x) => Some(x.to_string()),
        _ => None,
    };

    let num_threads: u32 = match matches.value_of("num_threads") {
        Some(x) => match x.trim().parse() {
            Ok(n) if n > 0 && n < 64 => n,
            _ => 0,
        },
        _ => 0,
    };

    let config = Config {
        action: matches.values_of_lossy("action").unwrap(),
        db_file: matches.values_of_lossy("db_file").unwrap(),
        command: matches.values_of_lossy("command").unwrap(),
        id_value: matches.values_of_lossy("id_value").unwrap(),
        fastq_ascii: matches.values_of_lossy("fastq_ascii").unwrap(),
        bin_dir: bin_dir,
        out_dir: out_dir,
        query: matches.values_of_lossy("query").unwrap(),
        centroids: matches.values_of_lossy("centroids").unwrap(),
    };

    Ok(config)
}
