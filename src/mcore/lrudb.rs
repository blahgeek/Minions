extern crate chrono;
extern crate rusqlite;

use self::chrono::TimeZone;

use std::result::Result;
use std::error::Error;
use std::path::Path;
use std::sync::Mutex;

pub struct LruResult {
    pub data: String,
    pub time: chrono::DateTime<chrono::Local>,
}

pub struct LruDB {
    conn: Mutex<rusqlite::Connection>,
    max_n: usize,
    table: String,
}


impl LruDB {

    pub fn add(&self, s: &str) -> Result<(), Box<Error + Sync + Send>> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().timestamp();
        conn.execute(&format!("INSERT OR REPLACE INTO {} (data, time) VALUES (?, ?)", self.table),
                     &[&s, &now])?;
        conn.execute(
            &format!("DELETE FROM {} WHERE id NOT IN
                      (SELECT id FROM {} ORDER BY time DESC, id DESC LIMIT {})",
                      self.table, self.table, self.max_n), &[])?;
        Ok(())
    }

    pub fn getall(&self) -> Result<Vec<LruResult>, Box<Error + Sync + Send>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            &format!("SELECT data, time FROM {} ORDER BY time DESC, id DESC", self.table))?;
        let data_iter =
            stmt.query_map(&[], |row| {
                LruResult {
                    data: row.get(0),
                    time: chrono::Local.timestamp(row.get(1), 0),
                }
            })?;
        let mut ret : Vec<LruResult> = Vec::new();
        for data in data_iter {
            ret.push(data.unwrap());
        }
        Ok(ret)
    }

    pub fn getall_textonly(&self) -> Result<Vec<String>, Box<Error + Sync + Send>> {
        Ok(self.getall()?.into_iter()
           .map(|x| x.data)
           .collect())
    }

    pub fn new(scope: &str, max_n: usize, dbpath: Option<&Path>) -> Result<LruDB, Box<Error + Sync + Send>> {
        let table = format!("lru_{}", scope);
        let conn =
            if let Some(dbpath) = dbpath {
                rusqlite::Connection::open(dbpath)?
            } else {
                rusqlite::Connection::open_in_memory()?
            };

        conn.execute(&format!("CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY,
                data TEXT UNIQUE,
                time INTEGER
            )", table), &[])?;
        conn.execute(&format!("CREATE INDEX IF NOT EXISTS time_id_idx ON {} (time, id)", table),
                     &[])?;

        Ok(LruDB {
            conn: Mutex::new(conn),
            max_n: max_n,
            table: table,
        })
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lrudb_test() {
        let lru = LruDB::new("test", 3, None).unwrap();
        lru.add("hello").unwrap();
        lru.add("world").unwrap();
        assert_eq!(lru.getall_textonly().unwrap(), vec!["world", "hello"]);

        lru.add("hello").unwrap();
        assert_eq!(lru.getall_textonly().unwrap(), vec!["hello", "world"]);

        lru.add("1").unwrap();
        lru.add("2").unwrap();
        assert_eq!(lru.getall_textonly().unwrap(), vec!["2", "1", "hello"]);
    }

}
