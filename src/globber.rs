
use glob::glob;
use std::path::PathBuf;
use crate::result::*;

pub struct Globber {
    pub pattern: String,
}

impl Globber {
    pub fn find(&self) -> Result<Vec<PathBuf>> {
        let paths = glob(&self.pattern)
            .map_err(|e| KvsErrorKind::GlobError(format!("pattern error: {}", e)))?;
        
        let mut result = vec![];

        for entry in paths {
            result.push(entry.map_err(|e| KvsErrorKind::GlobError(format!("pattern error: {}", e)))?);
        }

        Ok(result)
    }
}