use std::collections::BTreeMap;
use std::path::Path;
use std::fs::{self,File,OpenOptions};

pub mod result;
pub mod command;
pub mod kvdb;
pub mod logdb;
pub mod lines;
pub mod globber;
pub mod parts;

pub use result::*;
use kvdb::{KvDb,Visitor};
use command::Command;
use parts::{Parts,Id};

pub const DEFAULT_FILE_NAME: &str = "kvs.json";

use logdb::Offset;
type OffsetIndex = BTreeMap<String,(Id,Offset)>;
type PartitionsMap = BTreeMap<Id,KvDb>;

pub struct KvStore {
    parts: Parts,
    current_part: Id,
    kvdbs: PartitionsMap,
    store: OffsetIndex,
}

struct Loader {
    pub part: Id,
    pub index: OffsetIndex,
}

impl Loader {
    pub fn new(part: Id, index: OffsetIndex) -> Loader {
        Loader {
            part: part,
            index: index,
        }
    }
}

impl Visitor for Loader {
    fn command(&mut self, c: Command, offset: Offset) -> Result<bool> {
        match c {
            Command::Set{key,value: _value} => {
                self.index.insert(key, (self.part, offset));
            },
            Command::Remove{key} => {
                self.index.remove(&key);
            },
        }
        Ok(true)
    }
}

impl KvStore {
    pub fn cur<'a>(&'a self) -> &'a KvDb {
        self.part(self.current_part).expect("error")
    }

    pub fn cur_mut<'a>(&'a mut self) -> &'a mut KvDb {
        self.part_mut(self.current_part).expect("error")
    }

    pub fn part<'a>(&'a self, id: Id) -> Result<&'a KvDb> {
        Ok(self.kvdbs.get(&id).ok_or_else(|| KvsErrorKind::InvalidPartition(id))?)
    }

    pub fn part_mut<'a>(&'a mut self, id: Id) -> Result<&'a mut KvDb> {
        Ok(self.kvdbs.get_mut(&id).ok_or_else(|| KvsErrorKind::InvalidPartition(id))?)
    }
    
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let pos = self.cur_mut().append(Command::Set{key: key.clone(), value: value})?;
        self.store.insert(key, (self.current_part,pos));
        Ok(())
    }
    
    pub fn get_offset(&mut self, key: String) -> Result<Option<(Id,Offset)>> {
        Ok(self.store.get(&key).map(|v| v.to_owned()))
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        self.get_offset(key)
            .and_then(
                |offset| offset.map_or(Ok(None), 
                    |(id,offset)| {
                        match self.read_offset(id, offset)? {
                            Command::Set {key: _key, value} => Ok(Some(value)),
                            Command::Remove {key: _key} => Ok(None),
                        }
                    }))
    }

    pub fn read_offset(&mut self, id: Id, offset: Offset) -> Result<Command> {
        self.part_mut(id).expect("error").read_offset(offset)
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.store.remove(&key).is_none() {
            Err(KvsErrorKind::NotFound(key))?;
        } else {
            self.cur_mut().append(Command::Remove{key})?;
        }
        Ok(())
    }

    pub fn new(dir: &Path) -> Result<KvStore> {
        if !fs::metadata(dir).map_err(|e| KvsErrorKind::Io(e))?.is_dir() {
            return Err(KvsErrorKind::Config(format!("not a directory: {:?}", dir)))?;
        }

        let parts = Parts::new(dir);
        let mut kvdbs = BTreeMap::new();
        let mut max_id = None;
        for id in parts.find()? {
            let path = parts.path_for_id(id);
            let file = KvStore::open_file(&path)?;
            let kvdb = KvDb::new(file)?;
            
            kvdbs.insert(id, kvdb);
            max_id = Some(id);
        }

        let current_id = if let Some(some_id) = max_id {
            some_id
        } else {
            let (id,file) = parts.create()?;
            let kvdb = KvDb::new(file)?;
            kvdbs.insert(id, kvdb);
            id
        };

        Ok(KvStore {
            parts: parts,
            current_part: current_id,
            kvdbs: kvdbs,
            store: BTreeMap::new(),
        })
    }

    pub fn open(path: &Path) -> Result<KvStore> {
        let mut kvs = KvStore::new(path)?;
        kvs.load()?;
        Ok(kvs)
    }

    pub fn load(&mut self) -> Result<()> {
        let mut index = BTreeMap::new();

        for (id, kvdb) in self.kvdbs.iter_mut() {
            let loader = Loader::new(*id, index);
            let visitor = kvdb.visit(loader)?;
            index = visitor.index;
        }
        
        self.store = index;
        Ok(())
    }

    pub fn compact(&mut self) -> Result<()> {
        let (id,file) = self.parts.create()?;
        let kvdb = KvDb::new(file)?;
        let (index, kvdb) = self.copy(id, kvdb)?;
        
        for (id, _kvdb) in self.kvdbs.iter() {
            self.parts.remove(*id)?;
        }

        self.kvdbs = BTreeMap::new();
        self.kvdbs.insert(id, kvdb);
        self.store = index;
        self.current_part = id;

        Ok(())
    }
    
    pub fn copy(&mut self, dest_part: Id, dest: KvDb) -> Result<(OffsetIndex, KvDb)> {
        let mut index = Some(self.store.clone());
        let mut dest = Some(dest);

        for (id, kvdb) in self.kvdbs.iter_mut() {
            let copy_visitor = CopyVisitor { 
                src_part: *id, 
                src_index: index.take().unwrap(), 
                dest_part: dest_part,
                dest: dest.take().unwrap()
            };

            let copy_visitor = kvdb.visit(copy_visitor)?;

            dest = Some(copy_visitor.dest);
            index = Some(copy_visitor.src_index);
        }
        
        Ok((index.unwrap(), dest.unwrap()))
    }

    fn open_file(path: &Path) -> Result<File> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(|e| KvsErrorKind::Io(e))?;

        Ok(f)
    }
}

struct CopyVisitor {
    pub src_part: Id,
    pub src_index: OffsetIndex,
    pub dest_part: Id,
    pub dest: KvDb,
}

impl Visitor for CopyVisitor {
    fn command(&mut self, command: Command, pos: Offset) -> Result<bool> {
        let key = match command {
            Command::Set { ref key, value: ref _value } => key,
            Command::Remove { ref key } => key
        };
        if let Some((src_id, src_pos)) = self.src_index.get_mut(key) {
            if self.src_part == *src_id && pos == *src_pos {
                *src_pos = self.dest.append(command)?;
                *src_id = self.dest_part;
            }
        }
        Ok(true)
    }
}
