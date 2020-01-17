/*
* @Author: BlahGeek
* @Date:   2017-08-09
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-08-19
*/

extern crate crypto;
extern crate byteorder;

use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;

use self::byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use std::collections::btree_map::BTreeMap;

use std::io;
use std::io::{Read, Write};
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::rc::Rc;

use crate::mcore::item::Item;
use crate::mcore::fuzzymatch::fuzzymatch;

/// 20 byte array representing SHA1 hash result
#[derive(PartialOrd, PartialEq, Eq, Ord, Debug)]
struct SHA1Result {
    bytes: [u8; 20],
}

impl SHA1Result {
    fn read_from(reader: &mut dyn Read) -> io::Result<SHA1Result> {
        let mut sha1bytes: [u8; 20] = [0; 20];
        reader.read_exact(&mut sha1bytes)?;
        Ok(SHA1Result{ bytes: sha1bytes })
    }

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(&self.bytes)?;
        Ok(())
    }
}

impl<'a> From<&'a str> for SHA1Result {
    fn from(text: &'a str) -> SHA1Result {
        let mut hash = Sha1::new();
        hash.input(text.as_bytes());

        let mut bytes: [u8; 20] = [0; 20];
        hash.result(&mut bytes);

        SHA1Result{ bytes: bytes }
    }
}

const FILE_MAGIC: i32 = 0x23333333;

// Sort items using filter text, via fuzzymatch algorithm
// Store select history in file to adjust future sorting
//
// for privacy reasons, original data should not be saved to file
// instead, SHA1 hash and hit count is saved
//
// for every hit: (filter_text, selected_item), two entries is inserted:
//   - SHA1(SALT + selected_item)
//   - SHA1(SALT + filter_text + selected_item)
//
// File format: MAGIC (SHA1, count) (SHA1, count) (SHA1, count) ...
// At startup, the file is loaded, compacted (merge same hashes using count) and dumped back to the file
// While running, new data would be appended only (would be compacted on next running)
pub struct Matcher {
    statistics: BTreeMap<SHA1Result, u32>,
    salt: String,
    file: io::BufWriter<File>,
}

fn dump(path: &Path, statistics: &BTreeMap<SHA1Result, u32>) -> io::Result<()> {
    let f = File::create(path)?;
    let mut f = io::BufWriter::new(f);

    f.write_i32::<LittleEndian>(FILE_MAGIC)?;
    for (sha1, count) in statistics.iter() {
        sha1.write_to(&mut f)?;
        f.write_u32::<LittleEndian>(*count)?;
    }
    Ok(())
}

fn load(path: &Path) -> io::Result<BTreeMap<SHA1Result, u32>> {
    let f = File::open(path)?;
    let mut f = io::BufReader::new(f);

    let magic = f.read_i32::<LittleEndian>()?;
    if magic != FILE_MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid file magic"));
    }

    let mut statistics = BTreeMap::new();
    loop {
        match SHA1Result::read_from(&mut f) {
            Ok(sha1) => {
                let count = f.read_u32::<LittleEndian>()?;
                let mut inserted = false;
                if let Some(val) = statistics.get_mut(&sha1) {
                    *val += count;
                    inserted = true;
                }
                if !inserted {
                    statistics.insert(sha1, count);
                }
            },
            Err(_) => { break; }
        }
    }

    Ok(statistics)
}

impl Matcher {

    fn hash_item(&self, item: &Item) -> SHA1Result {
        let s = format!("{}:{}", &self.salt, item.get_search_str());
        s.as_str().into()
    }

    fn hash_pattern_item(&self, pattern: &str, item: &Item) -> SHA1Result {
        let s = format!("{}:{}:{}", &self.salt, pattern, item.get_search_str());
        s.as_str().into()
    }

    fn inc(&mut self, sha1: SHA1Result) -> io::Result<u32> {
        trace!("Inc: {:?}", &sha1);

        sha1.write_to(&mut self.file)?;
        self.file.write_u32::<LittleEndian>(1)?;
        self.file.flush()?;

        if let Some(val) = self.statistics.get_mut(&sha1) {
            *val += 1;
            return Ok(*val);
        }
        self.statistics.insert(sha1, 1);
        Ok(1)
    }

    /// Record a hit of item, optionally with pattern
    /// This would update the statistics and append log to file
    pub fn record(&mut self, pattern: Option<&str>, item: &Item) -> io::Result<()> {
        trace!("Record history with pattern {:?} and item {}", pattern, item.get_search_str());
        let sha1 = self.hash_item(item);
        let count = self.inc(sha1)?;
        trace!("New count for item: {}", count);
        if let Some(pattern) = pattern {
            if pattern.len() > 0 {
                let sha1 = self.hash_pattern_item(pattern, item);
                let count = self.inc(sha1)?;
                trace!("New count for item with pattern: {}", count);
            }
        }
        Ok(())
    }

    pub fn new(path: &Path, salt: &str) -> io::Result<Matcher> {
        let statistics = if path.exists() {
            debug!("Opening {:?} for statistics", path);
            load(path)?
        } else {
            debug!("Using empty statistics");
            BTreeMap::new()
        };
        trace!("Loaded statistics: {}", statistics.len());

        debug!("Dump (compact) statistics to {:?}", path);
        dump(path, &statistics)?;

        Ok(Matcher{
            statistics: statistics,
            file: io::BufWriter::new(OpenOptions::new().append(true).open(path)?),
            salt: salt.into(),
        })
    }

    /// Filter and sort items using fuzzymatch
    /// return filtered items
    pub fn sort(&self, pattern: &str, items: &[Rc<Item>]) -> Vec<Rc<Item>> {
        trace!("filter: {:?}", pattern);
        let scores = items.iter().map(|item| {
            let p0 = fuzzymatch(item.get_search_str(), pattern, false);
            let p1 = ((self.statistics.get(&self.hash_item(&item)).unwrap_or(&0) + 1) as f32).log2() as i32;
            let p2 = 2 * ((self.statistics.get(&self.hash_pattern_item(pattern, &item)).unwrap_or(&0) + 1) as f32).log2() as i32;
            trace!("Score: {}: {} + {} + {}", &item.title, p0, p1, p2);
            (p0 + p1 + p2, p0)  // final score and the base score
        });
        let mut items_and_scores =
            items.iter().map(|x| x.clone())
            .zip(scores.into_iter())
            .collect::<Vec<(Rc<Item>, (i32, i32))>>();
        items_and_scores.sort_by_key(|item_and_score| -(item_and_score.1).0);
        items_and_scores.into_iter()
            .filter(|item_and_score| (item_and_score.1).1 > 0)
            .map(|item_and_score| item_and_score.0)
            .collect::<Vec<Rc<Item>>>()
    }

}
