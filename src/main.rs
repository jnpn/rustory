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

#[derive(Serialize, Deserialize, Debug)]
struct Thing {
    // id|url|title|visit_count|typed_count|last_visit_time|hidden
    i:i64,
    s:String,
}

fn urls(conn:Connection) -> Result<Vec<Thing>, rusqlite::Error> {
    let mut stmt = try!(conn.prepare("SELECT * FROM urls LIMIT 32"));
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

impl serde::Serialize for ResponseWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let m:HashMap<String,String> = self.r.headers()
            .iter()
            .map(|(k,v)| (String::from(k.as_str()),String::from(v.to_str().unwrap())))
            .collect::<HashMap<String,String>>();

        let mut s = serializer.serialize_struct("Resp", 2)?;
        s.serialize_field("status_code", &self.r.status().as_u16())?;
        s.serialize_field("headers", &m)?;
        s.end()
    }
}

fn main() {
    let path = String::from("History");
    let client = Client::new();
    if let Ok(c) = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        if let Ok(all) = urls(c) {
            all.iter()
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
