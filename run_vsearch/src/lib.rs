extern crate clap;

use clap::{App, Arg};
use std::error::Error;
use std::process::{Command, Stdio};
use std::{
    env, fs::{self, DirBuilder}, io::Write, path::{Path, PathBuf},
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
    num_threads: u32,
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
        .arg(
            Arg::with_name("num_threads")
                .short("t")
                .long("num_threads")
                .value_name("INT")
                .default_value("12")
                .help("Number of threads"),
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
            cwd.join(PathBuf::from("vsearch-out"))
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

    let num_threads: u32 = match matches.value_of("num_threads") {
        Some(x) => match x.trim().parse() {
            Ok(n) if n > 0 && n < 64 => n,
            _ => 0,
        },
        _ => 0,
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
        num_threads: num_threads,
    };

    Ok(config)
}

// --------------------------------------------------
pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);

    let out_dir = &config.out_dir;
    if !out_dir.is_dir() {
        DirBuilder::new().recursive(true).create(&out_dir)?;
    }

    if let Some(id_value) = &config.id_value {
        if id_value < &0.0 || id_value > &1.0 {
            let msg = format!("Bad --id ({:?}), must be between 0 and 1", id_value);
            return Err(From::from(msg));
        }
    }

    let files = find_files(&config.query)?;
    let jobs = make_jobs(&config, &files)?;
    run_jobs(&jobs, "Running VSEARCH", 12);

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
        let msg = format!("No input files can be found in {}", paths.join(", "));
        return Err(From::from(msg));
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

// --------------------------------------------------
fn run_jobs(jobs: &Vec<String>, msg: &str, num_concurrent: u32) -> MyResult<()> {
    let num_jobs = jobs.len();

    if num_jobs > 0 {
        println!(
            "{} (# {} job{} @ {})",
            msg,
            num_jobs,
            if num_jobs == 1 { "" } else { "s" },
            num_concurrent
        );

        let mut process = Command::new("parallel")
            .arg("-j")
            .arg(num_concurrent.to_string())
            .arg("--halt")
            .arg("soon,fail=1")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()?;

        {
            let stdin = process.stdin.as_mut().expect("Failed to open stdin");
            stdin
                .write_all(jobs.join("\n").as_bytes())
                .expect("Failed to write to stdin");
        }

        let result = process.wait()?;
        if !result.success() {
            return Err(From::from("Failed to run jobs in parallel"));
        }
    }

    Ok(())
}

// --------------------------------------------------
fn make_jobs(config: &Config, files: &Vec<String>) -> Result<Vec<String>, Box<Error>> {
    let command = validate_command(&config.command)?;

    let needs_id_value = vec![
        "allpairs_global",
        "cluster_fast",
        "cluster_size",
        "cluster_smallmem",
        "usearch_global",
    ];

    if needs_id_value.contains(&command.as_str()) && config.id_value.is_none() {
        let msg = format!("--{} requires --id value (0 < id < 1)", command);
        return Err(From::from(msg));
    }

    let vsearch = "vsearch";
    let vsearch_path = match &config.bin_dir {
        Some(path) => Path::new(&path).join(vsearch),
        _ => PathBuf::from(vsearch),
    };

    let id_value = match config.id_value {
        Some(x) => x,
        _ => 0.0,
    };

    let mut jobs = vec![];
    for file in files.iter() {
        let basename = basename(&file)?;
        let out_file = config.out_dir.join(basename);
        jobs.push(format!(
            "{} --{} {} --id {:?} --alnout {} --threads {}",
            vsearch_path.to_string_lossy(),
            command,
            file,
            id_value,
            out_file.to_string_lossy(),
            config.num_threads,
        ));
    }
    println!("{:?}", jobs);

    Ok(jobs)
}

// --------------------------------------------------
fn basename(fname: &String) -> MyResult<String> {
    let buf = PathBuf::from(fname);
    match buf.file_name() {
        Some(x) => Ok(x.to_string_lossy().to_string()),
        _ => {
            let msg = format!("Cannot get basename from file \"{}\"", fname);
            Err(From::from(msg))
        }
    }
}
