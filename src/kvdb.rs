use std::fs::File;

use crate::logdb::{self,LogDb,Offset};
use crate::result::*;
use crate::command::*;

#[derive(Copy,Clone,PartialEq,Debug)]
struct Parser;

impl Parser {
    pub fn parse(&self, line: &str) -> Result<Command> {
        let command = serde_json::from_str(line)
            .map_err(|e| KvsErrorKind::ParserError(e))?;
        Ok(command)
    }

    pub fn encode(&self, c: Command) -> Result<String> {
        let s = serde_json::to_string(&c)
            .map_err(|e| KvsErrorKind::ParserError(e))?;
        Ok(s)
    }
}

pub trait Visitor {
    fn command(&mut self, command: Command, pos: Offset) -> Result<bool>;
}

struct ParserVisitor<V: Visitor> {
    parser: Parser,
    inner: V,
}

impl <V: Visitor> logdb::Visitor for ParserVisitor<V> {
    fn line(&mut self, line: String, pos: Offset) -> Result<bool> {
        let obj = self.parser.parse(&line)?;
        self.inner.command(obj, pos)
    }
}

pub struct KvDb {
    parser: Parser,
    logdb: LogDb,
}

impl KvDb {
    pub fn new(file: File) -> Result<KvDb> {
        Ok(KvDb {
            parser: Parser,
            logdb: LogDb::new(file)?,
        })
    }

    pub fn visit<V: Visitor>(&mut self, visitor: V) -> Result<V> {
        let parser = ParserVisitor { parser: self.parser, inner: visitor };
        let parser = self.logdb.visit(parser)?;
        Ok(parser.inner)
    }

    pub fn append(&mut self, command: Command) -> Result<Offset> {
        self.logdb.append(self.parser.encode(command)?)
    }

    pub fn read_offset(&mut self, offset: Offset) -> Result<Command> {
        let line = self.logdb.read_offset(offset)?;
        let command = self.parser.parse(&line)?;
        Ok(command)
    }
}
