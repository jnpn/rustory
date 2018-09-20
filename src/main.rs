extern crate reqwest;
#[macro_use]
extern crate serde;
extern crate serde_json;
extern crate rusqlite;
#[macro_use]
extern crate serde_derive;

use serde::ser::{Serialize, SerializeStruct, Serializer};
use rusqlite::{Connection, OpenFlags};
use reqwest::{Client, Response};
use std::fmt;
use std::iter::Map;

#[derive(Serialize, Deserialize, Debug)]
struct Thing {
    // id|url|title|visit_count|typed_count|last_visit_time|hidden
    i:i64,
    s:String,
}

fn urls(conn:Connection) -> Result<Vec<Thing>, rusqlite::Error> {
    let mut stmt = try!(conn.prepare("SELECT * FROM urls LIMIT 10"));
    let it = stmt
        .query_map(&[], |row| Thing {
            i: row.get("id"),
            s: row.get("url"),
        })
        .unwrap();
    let mut urls = Vec::new();
    for t in it {
        urls.push(t?)
    }
    Ok(urls)
}

struct ResponseWrapper { r: Response }

impl serde::Serialize for ResponseWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let m = &self.r.headers()
            .iter()
            .map(|(k,v)| (String::from(k.as_str()),String::from(v.to_str().unwrap())))
            .collect::<Vec<(String,String)>>();
        //TODO make `m` a real map
        let mut s = serializer.serialize_struct("Resp", 2)?;
        s.serialize_field("status_code", &self.r.status().as_u16())?;
        s.serialize_field("headers", m);
        s.end()
    }
}

fn main() {
    let path = String::from("History");
    let client = Client::new();
    let c = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY);
    match c {
        Ok(conn) => {
            match urls(conn) {
                Ok(urls) => {
                    //let lim = 32;
                    let _rs = urls
                        .iter()
                        .enumerate()
                        .map(|(i,t)| {
                            //println!("{} HEAD {:?}", i, t);
                            client.head(t.s.as_str()).send() })
                        .for_each(|m| {
                            match m {
                                Ok(r) => {
                                    let w = ResponseWrapper { r };
                                    let j = serde_json::to_string(&w);
                                    match j {
                                        Ok(s) => println!("{}", s),
                                        _ => eprintln!("json encoding error for {:?}", w.r)
                                    }
                                },
                                _ => println!("[error] after head...")
                            }
                        });
                        //.collect::<Vec<Result<Response, reqwest::Error>>>();

                    // println!("heads {}", rs.len());
                    // let (a,_) = rs.as_slice().split_at(lim);
                    // println!("heads {:?}", a);

                    // println!("found: {:?} url(s)", urls.len());
                    // let (a,_) = urls.as_slice().split_at(lim);
                    // println!("first {}: {:?}", lim, a);

                }
                _ => panic!("sqlite query `urls` failed")
            }
        }
        _ => panic!("connection to file `{}` failed", path)
    }
    // println!("bye.");
}
