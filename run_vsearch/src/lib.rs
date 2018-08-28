extern crate clap;

use clap::{App, Arg};
use std::error::Error;
//use std::process::{Command, Stdio};
use std::{
    env, fs::{self, DirBuilder, File}, io::Write, path::{Path, PathBuf},
};

// --------------------------------------------------
#[derive(Debug)]
pub struct Config {
    command: String,
    query: Vec<String>,
    centroids: Option<String>,
    db_file: Option<String>,
    id_value: Option<f64>,
    fastq_ascii: Option<u32>,
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
            Arg::with_name("command")
                .short("c")
                .long("command")
                .value_name("STR")
                .help("VSEARCH command")
                .required(true)
                .takes_value(true),
        )
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
                .help("Centroids")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("db_file")
                .short("d")
                .long("db")
                .value_name("FILE")
                .help("Database")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("out_dir")
                .short("o")
                .long("out_dir")
                .value_name("DIR")
                .help("Output directory")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("id_value")
                .short("i")
                .long("id")
                .value_name("INT")
                .help("ID value (e.g., 0.97")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("fastq_ascii")
                .short("f")
                .long("fastq_ascii")
                .value_name("INT")
                .help("FASTQ ASCII value (e.g., 64)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("bin_dir")
                .short("b")
                .long("bin_dir")
                .value_name("DIR")
                .help("Location of binaries")
                .takes_value(true),
        )
        .get_matches();

    let centroids = match matches.value_of("centroids") {
        Some(x) => Some(x.to_string()),
        _ => None,
    };

    let db_file = match matches.value_of("db_file") {
        Some(x) => Some(x.to_string()),
        _ => None,
    };

    let out_dir = match matches.value_of("out_dir") {
        Some(x) => PathBuf::from(x),
        _ => {
            let cwd = env::current_dir()?;
            cwd.join(PathBuf::from("mash-out"))
        }
    };

    let bin_dir = match matches.value_of("bin_dir") {
        Some(x) => Some(x.to_string()),
        _ => None,
    };

    let id_value = match matches.value_of("id_value") {
        Some(x) => match x.trim().parse::<f64>() {
            Ok(n) => Some(n),
            _ => None,
        },
        _ => None,
    };

    let fastq_ascii = match matches.value_of("fastq_ascii") {
        Some(x) => match x.trim().parse::<u32>() {
            Ok(n) => Some(n),
            _ => None,
        },
        _ => None,
    };

    let config = Config {
        query: matches.values_of_lossy("query").unwrap(),
        command: matches.value_of("command").unwrap().to_string(),
        centroids: centroids,
        db_file: db_file,
        bin_dir: bin_dir,
        out_dir: out_dir,
        id_value: id_value,
        fastq_ascii: fastq_ascii,
    };

    Ok(config)
}

// --------------------------------------------------
pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    let files = find_files(&config.query)?;
    println!(
        "Will process {} file{}",
        files.len(),
        if files.len() == 1 { "" } else { "s" }
    );

    let command = validate_command(&config.command)?;

    println!("Will do {}", command);

    Ok(())
}

// --------------------------------------------------
fn find_files(paths: &Vec<String>) -> Result<Vec<String>, Box<Error>> {
    let mut files = vec![];
    for path in paths {
        let meta = fs::metadata(path)?;
        if meta.is_file() {
            files.push(path.to_owned());
        } else {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let meta = entry.metadata()?;
                if meta.is_file() {
                    files.push(entry.path().display().to_string());
                }
            }
        };
    }

    if files.len() == 0 {
        return Err(From::from("No input files"));
    }

    Ok(files)
}

// --------------------------------------------------
fn validate_command(command: &String) -> Result<&String, Box<Error>> {
    let commands = vec![
        "allpairs_global",
        "cluster_fast",
        "cluster_size",
        "cluster_smallmem",
        "derep_fulllength",
        "derep_prefix",
        "fastq_chars",
        "fastq_convert",
        "fastq_eestats",
        "fastq_eestats2",
        "fastq_mergepairs",
        "fastq_stats",
        "fastx_filter",
        "fastx_mask",
        "fastx_revcomp",
        "fastx_subsample",
        "rereplicate",
        "search_exact",
        "shuffle",
        "sortbylength",
        "sortbysize",
        "uchime_denovo",
        "uchime_ref",
        "usearch_global",
    ];

    match commands.contains(&command.as_str()) {
        true => Ok(command),
        _ => {
            let msg = format!(
                "--command \"{}\" invalid, choose from:\n{}",
                command,
                commands
                    .iter()
                    .map(|s| format!(" - {}", s))
                    .collect::<Vec<String>>()
                    .join("\n")
            );
            return Err(From::from(msg));
        }
    }
}
