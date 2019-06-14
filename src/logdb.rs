use std::path::{Path};
use std::fs::{File,OpenOptions};
use std::io::{BufRead,BufReader,Seek,SeekFrom,Write};

use crate::result::*;

pub trait Visitor {
    fn line(&mut self, line: String) -> Result<bool>;
}

pub struct LogDb {
    f: File,
}

impl LogDb {
    pub fn open(path: &Path) -> Result<LogDb> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(|e| KvsErrorKind::Io(e))?;

        Ok(LogDb {
            f: f,
        })
    }

    pub fn visit<V: Visitor>(&mut self, mut visitor: V) -> Result<V> {
        self.f.seek(SeekFrom::Start(0))
            .map_err(|e| KvsErrorKind::Io(e))?;;
        let file = BufReader::new(&self.f);
        for line in file.lines() {
            let l = line.map_err(|e| KvsErrorKind::Io(e))?;
            if !visitor.line(l)? {
                break;
            }
        }
        Ok(visitor)
    }

    pub fn append(&mut self, record: String) -> Result<()> {
        self.f.seek(SeekFrom::End(0))
            .map_err(|e| KvsErrorKind::Io(e))?;
        writeln!(self.f, "{}", record)
            .map_err(|e| KvsErrorKind::Io(e))?;
        Ok(())
    }
}