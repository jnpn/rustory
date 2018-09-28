extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate rusqlite;
#[macro_use]
extern crate serde_derive;

use serde::ser::{SerializeStruct, Serializer};
use rusqlite::{Connection, OpenFlags};
use reqwest::{Client, Response};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

#[derive(Serialize, Deserialize, Debug)]
struct Thing {
    // id|url|title|visit_count|typed_count|last_visit_time|hidden
    i:i64,
    s:String,
}

fn urls(conn:Connection) -> std::result::Result<Vec<Thing>, rusqlite::Error> {
    let mut stmt = try!(conn.prepare("SELECT * FROM urls"));
    let it = stmt
        .query_map(&[], |row| Thing {
            i: row.get("id"),
            s: row.get("url"),
        })
        .unwrap()
        .map(|t| t.unwrap())
        .collect::<Vec<Thing>>();
    Ok(it)
}

struct ResponseWrapper { r: Response }

impl Debug for ResponseWrapper {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        self.r.fmt(formatter)
    }
}

impl serde::Serialize for ResponseWrapper {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let m:HashMap<&str,&str> = self.r.headers()
            .iter()
            .map(|(k,v)| {
                match (k.as_str(),v.to_str()) {
                    (k,Ok(v)) => (k,v),
                    (k,Err(_)) => (k,"error @ v.to_str()")
                }
            })
            .collect::<>();

        let mut s = serializer.serialize_struct("Resp", 4)?;
        s.serialize_field("url", &self.r.url().as_str())?;
        s.serialize_field("version", &format!("{:?}",&self.r.version()))?;
        s.serialize_field("status_code", &self.r.status().as_u16())?;
        s.serialize_field("headers", &m)?;
        s.end()
    }
}

fn all () {
    let path = String::from("History");
    let client = Client::new();

    if let Ok(c) = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        info!("{} -> sqlite3: ok", path);
        if let Ok(all) = urls(c) {
            all.iter()
                .inspect(|t| eprintln!("[info] {}", t.s.as_str()))
                .map(|t| { client.head(t.s.as_str()).send() })
                .for_each(|m| {
                    if let Ok(r) = m {
                        if let Ok(j) = serde_json::to_string(&ResponseWrapper { r }) {
                            println!("{}", j)
                        }
                    }
                })
        }
    }
}

fn main() {

    all()

}
