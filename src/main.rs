use clap::{Arg, App};
use std::{collections::HashMap, path::Path};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::LineWriter;

use std::fs::{File, metadata};
use std::io::prelude::*;

mod utils;

#[derive(Clone)]
struct FileEntry {
    fullpath: String,
    filesize: u64,
}

type FileHashTable = HashMap<u64, Vec<FileEntry>>;
type HashFn = fn(&FileEntry) -> Result<u64,()>;

impl FileEntry {
    fn new(filename: &str) -> FileEntry {
        FileEntry {
            fullpath: String::from(filename),
            filesize: metadata(filename).unwrap().len()
        }
    }
}

fn write_filetable(table: &FileHashTable, file: &mut dyn std::io::Write) -> std::io::Result<()> {

    let mut stream = LineWriter::new(file);

    for (hash, files) in table {
        let mut i: u64 = 0;
        if files.len() < 2 {
            continue;
        }

        for f in files {
            let line = format!("{};\"{}\";{}\n", if i == 0 { format!("{:016x}", hash) } else { String::from("---.---.---.---.") }, f.fullpath, f.filesize );
            stream.write_all(line.as_bytes())?;            
            i += 1;
        }
    }
    Ok(())
}

fn file_hash(file: &mut dyn std::io::Read) -> Result<u64,()> {
    let mut hasher = DefaultHasher::new();
    let chunk_size = 0x400000;

    loop {
        let mut chunk = Vec::with_capacity(chunk_size);
        let n = file.take(chunk_size as u64).read_to_end(&mut chunk);

        match n {
            Ok(n) => {
                if n == 0 { break; }
                hasher.write(&chunk);
                if n < chunk_size { break; }
            }
            Err(_) => return Err(())
        }
    }

    Ok(hasher.finish())
}

fn find_equal_files_by_hash(files: &Vec<FileEntry>, hash_fun: HashFn) -> FileHashTable {
    let mut hash_table: FileHashTable = HashMap::new();

    for file in files {
         let hash = hash_fun(&file);
         match hash {
            Ok(hash) => 
                hash_table.entry(hash)
                    .or_insert(Vec::new())
                    .push(file.clone()),
            Err(_) => continue
        }
    }

    hash_table
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
        .arg(Arg::with_name("mode")
                 .short("m")
                 .long("mode")
                 .takes_value(true)
                 .help("Mode (default: filename)"))
        .get_matches();
    
    let directory = args.value_of("dir").unwrap_or(".");
    let csv = args.value_of("csv").unwrap_or("");
    let mode = String::from(args.value_of("mode").unwrap_or("filename_filesize")).to_lowercase();
    let mut files: Vec<FileEntry> = Vec::new();

    let hash_fun: HashFn;
    let mut writer: Box<dyn std::io::Write> = if csv.is_empty() { Box::new(std::io::stdout()) } else { Box::new(File::create(csv).unwrap()) };

    match &mode[..] {
        "filename" => hash_fun = |fileentry| -> Result<u64,()> {
            let mut hasher = DefaultHasher::new();
            let filename = Path::new(&fileentry.fullpath).file_name();
            match filename {
                Some(filename) => hasher.write(filename.to_string_lossy().as_bytes()),
                None => return Err(())
            }

            Ok(hasher.finish())
        },
        "filename_filesize" => hash_fun = |fileentry| -> Result<u64,()> {
            let mut hasher = DefaultHasher::new();
            let fname = Path::new(&fileentry.fullpath).file_name();
            match fname {
                Some(fname) => {
                    hasher.write(fname.to_string_lossy().as_bytes());
                    hasher.write(format!("{}", fileentry.filesize).as_bytes());
                },
                None => return Err(())
            }

            Ok(hasher.finish())
        },
        "exhaustive" => hash_fun = |fileentry| -> Result<u64,()> {
            let mut f = File::open(&fileentry.fullpath).unwrap();
            file_hash(&mut f)
        },
        _ => {
            eprintln!("Invalid mode: {}", mode);
            return Err(())
        }
    }

    utils::for_each_file(directory, |filename: &str| {
        files.push(FileEntry::new(filename));
    });

    eprintln!("{} files in directory {}", files.len(), directory);
    if files.len() > 1 {
        let hash_table = find_equal_files_by_hash(&files, hash_fun);
        write_filetable(&hash_table, &mut *writer).unwrap();
    }

    Ok(())
}
