extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate rusqlite;

use rusqlite::{Connection, OpenFlags};
use reqwest::{Client, Response};
use std::iter::Map;

struct Thing { s:String }

fn urls(conn:Connection) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = try!(conn.prepare("SELECT * FROM urls LIMIT 1000"));
    let it = stmt.query_map(&[], |row| Thing { s: row.get("url") }).unwrap();
    let mut urls = Vec::new();
    for t in it {
        urls.push(t?.s)
    }
    Ok(urls)
}

fn _dump(s:Map<String,String>) {
    let j = serde_json::to_string("foo");
    println!("{:?}", j);
    println!("{:?}", s);
}

fn main() {
    let lim = 32;
    let path = String::from("History");
    let client = Client::new();
    let c = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY);
    match c {
        Ok(conn) => {
            match urls(conn) {
                Ok(urls) => {
                    let rs = urls
                        .iter()
                        .enumerate()
                        .map(|(i,u)| {
                            println!("{} HEAD {}", i, u);
                            client.head(u.as_str()).send() })
                        .collect::<Vec<Result<Response, reqwest::Error>>>();

                    println!("heads {}", rs.len());
                    let (a,_) = rs.as_slice().split_at(lim);
                    println!("heads {:?}", a);

                    println!("found: {:?} url(s)", urls.len());
                    let (a,_) = urls.as_slice().split_at(lim);
                    println!("first {}: {:?}", lim, a);
                }
                _ => panic!("sqlite query `urls` failed")
            }
        }
        _ => panic!("connection to file `{}` failed", path)
    }
    println!("bye.");
}
