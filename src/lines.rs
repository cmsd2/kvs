use std::io::BufRead;
use std::io::Result;

#[derive(Debug,PartialEq,PartialOrd)]
pub struct Line {
    pub pos: u64,
    pub text: String,
}

#[derive(Debug)]
pub struct Lines<B> {
    pos: u64,
    buf: B,
}

impl<B: BufRead> Iterator for Lines<B> {
    type Item = Result<Line>;

    fn next(&mut self) -> Option<Result<Line>> {
        let pos = self.pos;
        let mut buf = String::new();
        
        match self.buf.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                self.pos += buf.len() as u64;
                if buf.ends_with("\n") {
                    buf.pop();
                    if buf.ends_with("\r") {
                        buf.pop();
                    }
                }
                Some(Ok(Line { pos: pos, text: buf }))
            }
            Err(e) => Some(Err(e))
        }
    }
}

impl <B> Lines<B> {
    pub fn new(buf: B) -> Lines<B> {
        Lines {
            pos: 0,
            buf: buf,
        }
    }
}
