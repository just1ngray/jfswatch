#[cfg(test)]
pub mod utils {
    use std::fs;
    use std::path::PathBuf;

    pub fn make_files(basedir: &PathBuf, files: Vec<&str>) -> Vec<PathBuf> {
        let mut fullpaths = Vec::with_capacity(files.len());

        for file in files {
            let path = basedir.join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::File::create(&path).unwrap();
            eprintln!("Created file: {:?}", path);
            fullpaths.push(path);
        }

        return fullpaths;
    }
}
