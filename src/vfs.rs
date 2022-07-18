use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;

use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub enum VFSError {
    CreatePathAlreadyExists,
    CreatePathDoesNotExist,
    FilePathIsFolder,
    FolderPathIsFile,
    PathInvalid,
}

lazy_static! {
    static ref PATH_REGEX: Regex = Regex::new(r"[^a-zA-Z0-9_\-\$]").expect("Failed to compile path regex!");
}

fn validate_path(path: Vec<String>) -> Result<Vec<String>, VFSError> {
    for sub in &path {
        if PATH_REGEX.is_match(sub) {
            eprintln!("path: {:?}", path);
            return Err(VFSError::PathInvalid);
        }
    }
    Ok(path)
}

fn clean_path<S: Into<String>>(path: S) -> String {
    let mut path = path.into();
    path = path.replace(".", "$");
    if path.starts_with("/") {
        path.remove(0);
    }
    if path.ends_with("/") {
        path.pop();
    }
    path
}

#[derive(Serialize, Deserialize)]
pub struct Vfs {
    disk_path: String,
    pub root: Folder,
}

impl Vfs {
    pub fn load<S: Into<String>>(path: S) -> Self {
        let path = path.into();
        if let Ok(vfs) = Self::load_internal(path.clone()) {
            vfs
        } else {
            Self::new(path)
        }
    }

    pub fn flush(&self) {
        use std::io::Write;
        let data = serde_json::to_string(self).expect("Failed to serialize filesystem to json!");
        let mut f = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(&self.disk_path).expect("Failed to open filesystem on disk!");
        write!(f, "{}", data);
        f.flush().expect("Failed to flush filesystem disk file!");
    }

    fn load_internal(path: String) -> anyhow::Result<Self> {
        let bufreader = std::io::BufReader::new(std::fs::File::open(path).map_err(|e| { eprintln!("error: {}", e); e})?);
        let vfs = serde_json::from_reader(bufreader).map_err(|e| { eprintln!("error: {}", e); e})?;
        Ok(vfs)
    }

    fn new(path: String) -> Self {
        println!("Creating new filesystem from scratch!");
        let mut vfs = Self {
            disk_path: path,
            root: Folder::empty(),
        };
        vfs.create_folder("", "test").unwrap();
        vfs.create_file("test", "test_file.txt").unwrap();
        vfs.flush();
        vfs
    }

    pub fn create_folder<S: Into<String>>(&mut self, path: S, name: S) -> Result<(), VFSError> {
        let name = clean_path(name);
        let path = clean_path(path);
        if path == "" {
            return self.root.create_folder_local(name.clone());
        }
        let mut path_split = validate_path(path.split("/").map(|s| s.to_owned()).collect::<Vec<String>>())?.into_iter().rev();
        self.root.create_folder(&mut path_split, name)
    }

    pub fn create_file<S: Into<String>>(&mut self, path: S, name: S) -> Result<(), VFSError> {
        let name = clean_path(name);
        let path = clean_path(path);
        let full_path = format!("{}/{}", path.clone(), name.clone());
        if path == "" {
            return self.root.create_file_local(name.clone(), full_path);
        }
        let mut path_split = validate_path(path.split("/").map(|s| s.to_owned()).collect::<Vec<String>>())?.into_iter().rev();
        self.root.create_file(&mut path_split, name, full_path)
    }
}

#[derive(Serialize, Deserialize)]
pub enum VFSItem {
    Folder(Folder),
    File(File),
}

#[derive(Serialize, Deserialize)]
pub struct Folder {
    children: HashMap<String, VFSItem>,
}

impl Folder {
    pub fn empty() -> Self {
        Self {
            children: HashMap::new(),
        }
    }

    fn get_folder_mut<S: Into<String>>(&mut self, name: S) -> Result<&mut Folder, VFSError> {
        let name = name.into();
        match self.children.get_mut(&name).ok_or(VFSError::CreatePathDoesNotExist)? {
            VFSItem::Folder(folder) => Ok(folder),
            VFSItem::File(_) => Err(VFSError::FolderPathIsFile),
        }
    }

    fn create_folder<P: Iterator<Item = String>, S: Into<String>>(&mut self, path_iter: &mut P, name: S) -> Result<(), VFSError> {
        if let Some(next) = path_iter.next() {
            self.get_folder_mut(next)?.create_folder(path_iter, name)
        } else {
            // There is no next, so we must insert into ourself
            self.create_folder_local(name)
        }
    }

    fn create_folder_local<S: Into<String>>(&mut self, name: S) -> Result<(), VFSError> {
        let name = name.into();
        if self.children.contains_key(&name) {
            return Err(VFSError::CreatePathAlreadyExists);
        }
        self.children.insert(name, VFSItem::Folder(Folder::empty()));
        Ok(())
    }

    fn create_file<P: Iterator<Item = String>, S: Into<String>>(&mut self, path_iter: &mut P, name: S, full_path: S) -> Result<(), VFSError> {
        if let Some(next) = path_iter.next() {
            self.get_folder_mut(next)?.create_file(path_iter, name, full_path)
        } else {
            // There is no next, so we must insert into ourself
            self.create_file_local(name, full_path)
        }
    }

    fn create_file_local<S: Into<String>>(&mut self, name: S, full_path: S) -> Result<(), VFSError> {
        let name = name.into();
        let full_path = full_path.into();
        if self.children.contains_key(&name) {
            return Err(VFSError::CreatePathAlreadyExists);
        }
        self.children.insert(name, VFSItem::File(File::empty(full_path)));
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct File {
    disk_path: String,
}

impl File {
    pub fn empty(vfs_path: String) -> Self {
        Self {
            disk_path: vfs_path,
        }
    }
}
