use std::path::{Path};
use std::fs::{File,OpenOptions};
use std::io::{BufRead,BufReader,Seek,SeekFrom,Write};

use crate::result::*;
use crate::lines::*;

pub type Offset = u64;

pub trait Visitor {
    fn line(&mut self, line: String, offset: Offset) -> Result<bool>;
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
            .map_err(|e| KvsErrorKind::Io(e))?;
        let file = BufReader::new(&self.f);
        for line in Lines::new(file) {
            let l = line.map_err(|e| KvsErrorKind::Io(e))?;
            if !visitor.line(l.text, l.pos)? {
                break;
            }
        }
        Ok(visitor)
    }

    pub fn append(&mut self, record: String) -> Result<Offset> {
        let pos = self.f.seek(SeekFrom::End(0))
            .map_err(|e| KvsErrorKind::Io(e))?;
        
        writeln!(self.f, "{}", record)
            .map_err(|e| KvsErrorKind::Io(e))?;

        Ok(pos)
    }

    pub fn read_offset(&mut self, offset: Offset) -> Result<String> {
        self.f.seek(SeekFrom::Start(offset))
            .map_err(|e| KvsErrorKind::Io(e))?;
        let mut file = BufReader::new(&self.f);
        let mut line = String::new();
        file.read_line(&mut line)
            .map_err(|e| KvsErrorKind::Io(e))?;
        Ok(line)
    }
}