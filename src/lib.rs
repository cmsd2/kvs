use std::collections::BTreeMap;
use std::path::Path;
use std::fs;

pub mod result;
pub mod command;
pub mod kvdb;
pub mod logdb;
pub mod lines;

pub use result::*;
use kvdb::{KvDb,Visitor};
use command::Command;

pub const DEFAULT_FILE_NAME: &str = "kvs.json";

use logdb::Offset;
type OffsetIndex = BTreeMap<String,Offset>;

pub struct KvStore {
    kvdb: KvDb,
    store: OffsetIndex,
}

struct Loader {
    pub index: OffsetIndex,
}

impl Loader {
    pub fn new() -> Loader {
        Loader {
            index: BTreeMap::new(),
        }
    }
}

impl Visitor for Loader {
    fn command(&mut self, c: Command, offset: Offset) -> Result<bool> {
        match c {
            Command::Set{key,value} => {
                self.index.insert(key,offset);
            },
            Command::Remove{key} => {
                self.index.remove(&key);
            },
        }
        Ok(true)
    }
}

impl KvStore {
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let pos = self.kvdb.append(Command::Set{key: key.clone(), value: value})?;
        self.store.insert(key, pos);
        Ok(())
    }
    
    pub fn get_offset(&mut self, key: String) -> Result<Option<Offset>> {
        Ok(self.store.get(&key).map(|v| v.to_owned()))
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        self.get_offset(key)
            .and_then(
                |offset| offset.map_or(Ok(None), 
                    |offset| {
                        match self.read_offset(offset)? {
                            Command::Set {key: _key, value} => Ok(Some(value)),
                            Command::Remove {key: _key} => Ok(None),
                        }
                    }))
    }

    pub fn read_offset(&mut self, offset: Offset) -> Result<Command> {
        self.kvdb.read_offset(offset)
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.store.remove(&key).is_none() {
            Err(KvsErrorKind::NotFound(key))?;
        } else {
            self.kvdb.append(Command::Remove{key})?;
        }
        Ok(())
    }

    pub fn new(kvdb: KvDb) -> KvStore {
        KvStore {
            kvdb: kvdb,
            store: BTreeMap::new(),
        }
    }

    pub fn open(path: &Path) -> Result<KvStore> {
        let path = if fs::metadata(path).map_err(|e| KvsErrorKind::Io(e))?.is_dir() {
            path.to_owned().join(crate::DEFAULT_FILE_NAME)
        } else {
            path.to_owned()
        };

        let kvdb = KvDb::open(&path)?;
        let mut store = KvStore::new(kvdb);
        store.load()?;
        Ok(store)
    }

    pub fn load(&mut self) -> Result<()> {
        let visitor = self.kvdb.visit(Loader::new())?;
        self.store = visitor.index;
        Ok(())
    }
}