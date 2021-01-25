use walkdir::WalkDir;

pub fn for_each_file(directory: &str, mut func: impl FnMut(&str)) {
    for entry in WalkDir::new(directory)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| !e.file_type().is_dir() && !e.path_is_symlink() ) {   
        match entry.path().to_str() {
            Some(x) => func(x),
            None => continue
        }
    }
}