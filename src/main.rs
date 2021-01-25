use clap::{Arg, App};
use std::{env, collections::HashMap, path::Path};
use walkdir::WalkDir;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::LineWriter;

use std::fs::File;
use std::io::prelude::*;

mod utils;

struct FileEntry {
    fullpath: String,
    filesize: u64
}

type FileTable = HashMap<u64, Vec<FileEntry>>;


impl FileEntry {
    fn new(filename: String, file: &File) -> FileEntry {
        FileEntry {
            fullpath: filename,
            filesize: file.metadata().unwrap().len()
        }
    }
}

fn write_filetable(table: &FileTable, file: &mut dyn std::io::Write) -> std::io::Result<()> {

    let mut stream = LineWriter::new(file);

    for (hash, files) in table {
        let mut i: u64 = 0;
        if files.len() < 2 {
            continue;
        }

        for f in files {
            let line = format!("{};{};{}\n", if i == 0 { format!("{:016x}", hash) } else { String::from("---.---.---.---.") }, f.fullpath, f.filesize );
            stream.write_all(line.as_bytes())?;            
            i += 1;
        }
    }
    Ok(())
}

fn file_hash(file: &mut dyn std::io::Read) -> std::io::Result<u64> {

    let mut hasher = DefaultHasher::new();
    
    let chunk_size = 0x400000;

    loop {
        let mut chunk = Vec::with_capacity(chunk_size);
        let n = file.take(chunk_size as u64).read_to_end(&mut chunk)?;
        if n == 0 { break; }
        hasher.write(&chunk);
        if n < chunk_size { break; }
    }

    Ok(hasher.finish())
}

fn main() -> Result<(), ()> {
    let args = App::new("Duplicate finder")
        .version("0.1.0")
        .author("Michael Winkelmann <michaelwinkelmann@posteo.de>")
        .about("Finds duplicated files in a directory via file checksum")
        .arg(Arg::with_name("dir")
                 .short("d")
                 .long("dir")
                 .takes_value(true)
                 .help("Directory to scan"))
        .arg(Arg::with_name("csv")
                 .short("o")
                 .long("csv")
                 .takes_value(true)
                 .help("Output CSV file"))
        .get_matches();
    
    let directory = args.value_of("dir").unwrap_or(".");
    let csv = args.value_of("csv").unwrap_or("");

    let mut files: HashMap<u64, Vec<FileEntry>> = HashMap::new();

    let mut count: u64 = 0;

    utils::for_each_file(directory, |filename: &str| {
        count += 1;
    });

    println!("{} files in directory {}", count, directory);

    utils::for_each_file(directory, |filename: &str| {
        let mut file = File::open(&filename).unwrap();
        if file.metadata().unwrap().is_dir() {
            return;
        }

        files.entry(file_hash(&mut file).unwrap())
            .or_insert(Vec::new())
            .push(FileEntry::new(filename.to_string(), &file));
        if files.len() % 100 == 0 && !csv.is_empty() {
            println!("{:.2}% files processed", ((files.len() as f64) * 100.0) / (count as f64) );
        }
    });

    if !csv.is_empty() {
        let mut file = File::create(csv).unwrap();
        write_filetable(&files, &mut file).unwrap();
    } else {
        write_filetable(&files, &mut std::io::stdout()).unwrap();
    }

    Ok(())
}
