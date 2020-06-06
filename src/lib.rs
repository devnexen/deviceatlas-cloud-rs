use json;
pub use serde::{Deserialize, Serialize};
use crypto::digest::Digest;
use curl::easy::{Easy, List};
use lru_cache::LruCache;
use std::collections::HashMap;
use std::process;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    pub b: bool,
    pub i: i32,
    pub s: String,
}

impl Value {
    pub fn new() -> Value {
        Value {
            b: false,
            i: 0,
            s: String::new(),
        }
    }

    pub fn to_string(&self) -> String {
        self.s.to_string()
    }
}

impl PartialEq for Value {
    fn eq(&self, v: &Self) -> bool {
        self.s == v.s
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub licence_key: String,
    cc: LruCache<String, Properties>,
}

impl Config {
    pub fn new(host: String, licence_key: String, centries: usize) -> Config {
        Config {
            host: host,
            licence_key: licence_key,
            cc: LruCache::new(centries),
        }
    }
}

type Properties = HashMap<String, Value>;

#[derive(Clone)]
pub struct Dacloud {
    cfg: Config,
    hash: crypto::sha1::Sha1,
    pub headers: HashMap<String, String>,
    pub resp: Vec<u8>,
}

impl Dacloud {
    pub fn new(cfg: Config) -> Dacloud {
        Dacloud {
            cfg: cfg,
            hash: crypto::sha1::Sha1::new(),
            headers: HashMap::new(),
            resp: Vec::new(),
        }
    }

    pub fn req(&mut self) -> Properties {
        let mut jobj = json::JsonValue::new_object();
        let mut props = Properties::new();
        let mut hdl = Easy::new();
        let mut hdrs = List::new();

        self.hash.reset();
        self.resp.clear();

        if !self.headers.contains_key("user-agent") {
            return props;
        }

        let ua = self.headers.get("user-agent").unwrap().clone();
        let full_url = format!(
            "http://{}/v1/detect/properties?licencekey={}&useragent={}",
            self.cfg.host, self.cfg.licence_key, ua
        );

        for (k, v) in self.headers.clone() {
            self.hash.input_str(&k);
            self.hash.input_str(&v);
            hdrs.append(&format!("X-DA-{}: {}", k, v)).unwrap();
        }

        let hashstr = self.hash.result_str().clone();

        if self.cfg.cc.contains_key(&hashstr) {
            return self.cfg.cc.get_mut(&hashstr).unwrap().clone();
        }

        hdrs.append(&format!("User-Agent: rust/{}", env!("CARGO_PKG_VERSION")))
            .unwrap();
        hdrs.append("Accept: application/json").unwrap();

        hdl.http_headers(hdrs).unwrap();
        hdl.url(&full_url[..]).unwrap();
        {
            let mut t = hdl.transfer();
            t.write_function(|b| {
                self.resp.extend_from_slice(b);
                Ok(b.len())
            })
            .unwrap();
            match t.perform() {
                Ok(_) => (),
                Err(s) => {
                            println!("error request: {} from {}", s, full_url);
                            process::exit(-1)
                          },
            };
        }
        match String::from_utf8(self.resp.clone()) {
            Ok(s) => match json::parse(&s[..]) {
                Ok(ob) => jobj = ob.clone(),
                Err(_) => println!("error decoding: {:?}", String::from_utf8(self.resp.clone())),
            },
            Err(_) => (),
        };
        let properties = jobj["properties"].clone();
        for (k, v) in properties.entries() {
            let mut val = Value::new();
            if v == "true" || v == "false" {
                val.b = v.to_string().parse().unwrap();
            } else {
                let mut i: i32 = -1;
                if let Ok(s) = v.to_string().parse::<i32>() {
                    i = s;
                }
                val.i = i;
            }
            val.s = v.to_string().clone();
            props.insert(String::from(k), val.clone());
        }
        props
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dc_integration_tests() {
        let mut host = String::from("region0.deviceatlascloud.com");
        let mut licence_key = String::from("12345");
        let cfg = Config::new(host, licence_key, 0 as usize);
        let mut dc = Dacloud::new(cfg);
        assert!(dc.headers.len() == 0);
        dc.headers
            .insert(String::from("user-agent"), String::from("iPhone"));
        assert!(dc.headers.len() == 1);
        let mut ret = dc.req();
        assert!(ret.len() == 0);
        host = String::from("region2.deviceatlascloud.com");
        licence_key = String::from("dummy");
        let cfg2 = Config::new(host, licence_key, 32 as usize);
        let mut dc2 = Dacloud::new(cfg2);
        dc2.headers
            .insert(String::from("user-agent"), String::from("iPhone"));
        assert!(dc.headers == dc2.headers);
        ret = dc.req();
        assert!(ret.len() == 0);
    }
}
