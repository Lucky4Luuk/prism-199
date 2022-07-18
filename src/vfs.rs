use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub enum VFSError {
    CreatePathAlreadyExists,
    CreatePathDoesNotExist,
    FilePathIsFolder,
    FolderPathIsFile,
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
        let bufreader = std::io::BufReader::new(std::fs::File::open(path).unwrap());
        let vfs = serde_json::from_reader(bufreader).unwrap();
        Ok(vfs)
    }

    fn new(path: String) -> Self {
        println!("Creating new filesystem from scratch!");
        let vfs = Self {
            disk_path: path,
            root: Folder::empty(),
        };
        vfs.flush();
        vfs
    }

    pub fn create_folder<S: Into<String>>(&mut self, path: S, name: S) -> Result<(), VFSError> {
        let name = name.into();
        let mut path = path.into();
        if path.starts_with("/") {
            path.remove(0);
        }
        if path.ends_with("/") {
            path.pop();
        }
        if path == "" {
            return self.root.create_folder_local(name.clone());
        }
        let mut path_split = path.split("/").map(|s| s.to_owned()).collect::<Vec<String>>().into_iter().rev();
        self.root.create_folder(&mut path_split, name)
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

    fn create_file<S: Into<String>>(&mut self, name: S, full_path: S) -> Result<(), VFSError> {
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
