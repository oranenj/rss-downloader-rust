extern crate config;
extern crate failure;
extern crate rss;
extern crate serde;
extern crate regex;
extern crate reqwest;
extern crate sha2;
use sha2::Digest;
use std::collections::HashMap;
use std::fs::{File};
use std::fs;
use std::path::Path;
use std::io::prelude::*;
use rss::Channel;
use failure::Error;

#[macro_use]
extern crate serde_derive;

#[derive(Debug, Deserialize)]
struct Source {
  url: String,
  matchers: Vec<String>
}

#[derive(Debug, Deserialize)]
struct Settings {
    file_suffix: String,
    cache_dir: String,
    download_dir: String,
    #[serde(flatten)]
    sources: HashMap<String, Source>
}

fn sha2_hash(input: &[u8]) -> String {
    format!("{:x}", sha2::Sha256::digest(&input))
}

fn process_source(s: &Source, root: &Path, cache_dir: &Path, suffix: &str) -> Result<(), Error> {
    println!("Downloading RSS from {}", s.url);
    let rss = Channel::read_from(std::io::BufReader::new(reqwest::get(&s.url)?))?;
    let set = regex::RegexSet::new(&s.matchers)?;

    let matches = rss.items().iter().filter(|&i| set.is_match(i.title().unwrap_or("NO_TITLE")));
    for m in matches {
        let maybe_guid = m.guid();

        let guid = match maybe_guid {
            Some(g) => g.value(),
            None  => { continue },
        };

        let t = m.title().unwrap_or("NO_TITLE");
        // This is the mark file to avoid duplicate downloads
        let mark = Path::join(cache_dir, sha2_hash(&guid.as_bytes()));
        if mark.exists() {
            println!("File already downloaded: {}", m.title().unwrap_or("NO_TITLE"));
            continue;
        }
        let mut buf = vec![];
        reqwest::get(m.link().unwrap_or(""))?.copy_to(&mut buf)?;
        let filename = format!("{}{}", sha2_hash(&buf), suffix);
        let dst = root.join(filename);
        println!("Downloading '{}', file: {}", t, dst.to_string_lossy());
        File::create(dst)?.write_all(&buf)?;
        File::create(mark)?;
    }
    return Ok(());
}

fn main() -> Result<(), Error> {
    let mut config = config::Config::default();

    config.merge(config::File::with_name("config")).unwrap();

    let settings : Settings = config.try_into().unwrap();

    let root = Path::new(&settings.download_dir);
    let cache_dir = Path::new(&settings.cache_dir);
    fs::create_dir_all(root).unwrap();
    fs::create_dir_all(cache_dir).unwrap();
    for s in settings.sources.values() {
       let res = process_source(&s, root, cache_dir, &settings.file_suffix);
       match res {
         Ok(_) => continue,
         Err(e) => println!("Error occurred while processing: {}", e),
       }
    }
    return Ok(())
}
