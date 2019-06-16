use std::path::{Path,PathBuf};
use std::fs::{self,File,OpenOptions};

use crate::result::*;
use crate::globber::*;

pub type Id = usize;

pub struct Parts {
    pub dir: PathBuf,
    pub ext: String,
    pub globber: Globber,
}

impl Parts {
    pub fn new(dir: &Path) -> Parts {
        let ext = "kvs";
        let pattern = dir.join(format!("*.{}", ext));
        Parts {
            dir: dir.to_owned(),
            ext: ext.to_owned(),
            globber: Globber { pattern: pattern.to_str().unwrap().to_owned() },
        }
    }

    pub fn create(&self) -> Result<(Id,File)> {
        let id = self.next_id()?;

        let path = self.path_for_id(id);

        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| KvsErrorKind::Io(e))?;

        Ok((id,f))
    }

    pub fn remove(&self, id: Id) -> Result<()> {
        let path = self.path_for_id(id);

        fs::remove_file(path)
            .map_err(|e| KvsErrorKind::Io(e))?;
        
        Ok(())
    }

    pub fn find(&self) -> Result<Vec<Id>> {
        let mut result = vec![];

        for path in self.globber.find()? {
            result.push(self.id_for_path(&path)?);
        }

        result.sort();

        Ok(result)
    }

    pub fn open(&self, id: Id) -> Result<File> {
        let path = self.path_for_id(id);
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|e| KvsErrorKind::Io(e))?;
        Ok(f)
    }

    pub fn path_for_id(&self, id: Id) -> PathBuf {
        let name = PathBuf::from(format!("{}.{}", id, self.ext));
        self.dir.join(name)
    }

    pub fn id_for_path(&self, path: &Path) -> Result<Id> {
        let name = path.file_stem()
            .ok_or_else(|| KvsErrorKind::GlobError(format!("error parsing file name {:?}", path.to_str())))?;
        let id =  usize::from_str_radix(&name.to_string_lossy(), 10)
            .map_err(|e| KvsErrorKind::ParseIntError(e))?;
        Ok(id)
    }

    pub fn next_id(&self) -> Result<Id> {
        let paths = self.globber.find()?;
        let mut max_id = 0;
        for path in paths {
            let id = self.id_for_path(&path)?;
            max_id = std::cmp::max(id, max_id);
        }
        Ok(max_id+1)
    }
}