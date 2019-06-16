use clap::{App,Arg,ArgMatches,SubCommand};
use std::process;
use std::path::PathBuf;

static VERSION: &str = env!("CARGO_PKG_VERSION");
static AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
static DEFAULT_PATH: &str = ".";

fn main() {

    let matches = App::new("kvs")
                          .version("1.0")
                          .author(AUTHOR)
                          .about("kv store")
                          .arg(Arg::with_name("version")
                              .short("V")
                              .help("version"))
                          .arg(Arg::with_name("path")
                              .short("p")
                              .takes_value(true)
                              .required(false)
                              .help("path to database file"))
                          .subcommand(SubCommand::with_name("get")
                                      .about("get value from the kv store")
                                      .version(VERSION)
                                      .author(AUTHOR)
                                      .arg(Arg::with_name("key")
                                          .short("k")
                                          .index(1)
                                          .required(true)
                                          .help("key")))
                          .subcommand(SubCommand::with_name("set")
                                      .about("set value from the kv store")
                                      .version(VERSION)
                                      .author(AUTHOR)
                                      .arg(Arg::with_name("key")
                                          .short("k")
                                          .index(1)
                                          .required(true)
                                          .help("key"))
                                      .arg(Arg::with_name("value")
                                          .short("v")
                                          .index(2)
                                          .required(true)
                                          .help("value")))
                          .subcommand(SubCommand::with_name("rm")
                                      .about("remove value from the kv store")
                                      .version(VERSION)
                                      .author(AUTHOR)
                                      .arg(Arg::with_name("key")
                                          .short("k")
                                          .index(1)
                                          .required(true)
                                          .help("key")))
                          .subcommand(SubCommand::with_name("compact")
                                      .about("compact log files")
                                      .version(VERSION)
                                      .author(AUTHOR)
                                      )
                          .get_matches();

    

    match run(matches) {
        Ok(()) => {},
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}

fn run(matches: ArgMatches) -> kvs::Result<()> {
    let path = matches.value_of("path").unwrap_or(DEFAULT_PATH);

    if let Some(matches) = matches.subcommand_matches("get") {
        let key = matches.value_of("key").unwrap();
            
        if let Some(value) = get(path, key)? {
            println!("{}", value);
            Ok(())
        } else {
            Err(kvs::KvsErrorKind::NotFound(key.to_owned()))?
        }
    } else if let Some(matches) = matches.subcommand_matches("set") {
        let key = matches.value_of("key").unwrap();
        let value = matches.value_of("value").unwrap();

        set(path, key, value)?;

        Ok(())
    } else if let Some(matches) = matches.subcommand_matches("rm") {
        let key = matches.value_of("key").unwrap();

        remove(path, key)?;

        Ok(())
    } else if let Some(_matches) = matches.subcommand_matches("compact") {
        compact(path)?;

        Ok(())
    } else {
        if matches.is_present("version") {
            println!("{}", VERSION);
            Ok(())
        } else {
            println!("{}", matches.usage());
            process::exit(1);
        }
    }
}

pub fn get(path: &str, key: &str) -> kvs::Result<Option<String>> {
    let mut store = kvs::KvStore::open(&PathBuf::from(&path))?;

    let value = store.get(key.to_owned())?;

    Ok(value)
}

pub fn set(path: &str, key: &str, value: &str) -> kvs::Result<()> {
    let mut store = kvs::KvStore::open(&PathBuf::from(&path))?;

    store.set(key.to_owned(), value.to_owned())?;

    Ok(())
}

pub fn remove(path: &str, key: &str) -> kvs::Result<()> {
    let mut store = kvs::KvStore::open(&PathBuf::from(&path))?;

    store.remove(key.to_owned())?;

    Ok(())
}

pub fn compact(path: &str) -> kvs::Result<()> {
    let mut store = kvs::KvStore::open(&PathBuf::from(&path))?;

    store.compact()?;

    Ok(())
}