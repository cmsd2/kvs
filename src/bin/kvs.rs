use clap::{App,Arg,SubCommand};
use std::process;

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let author = env!("CARGO_PKG_AUTHORS");

    let matches = App::new("kvs")
                          .version("1.0")
                          .author(author)
                          .about("kv store")
                          .arg(Arg::with_name("version")
                              .short("V")
                              .help("version"))
                          .subcommand(SubCommand::with_name("get")
                                      .about("get value from the kv store")
                                      .version(version)
                                      .author(author)
                                      .arg(Arg::with_name("key")
                                          .short("k")
                                          .index(1)
                                          .required(true)
                                          .help("key")))
                          .subcommand(SubCommand::with_name("set")
                                      .about("set value from the kv store")
                                      .version(version)
                                      .author(author)
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
                                      .version(version)
                                      .author(author)
                                      .arg(Arg::with_name("key")
                                          .short("k")
                                          .index(1)
                                          .required(true)
                                          .help("key")))
                          .get_matches();

    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    if let Some(matches) = matches.subcommand_matches("get") {
        if let Some(key) = matches.value_of("key") {
            eprintln!("unimplemented");
            process::exit(1);
        }
    } else if let Some(matches) = matches.subcommand_matches("set") {
        if let Some(key) = matches.value_of("key") {
            if let Some(value) = matches.value_of("value") {
                eprintln!("unimplemented");
                process::exit(1);
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("rm") {
        if let Some(key) = matches.value_of("key") {
            eprintln!("unimplemented");
            process::exit(1);
        }
    } else {
        if matches.is_present("version") {
            println!("{}", version);
        } else {
            println!("{}", matches.usage());
            process::exit(1);
        }
    }

    // more program logic goes here...
}

