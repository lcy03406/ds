use std::fs::File;
use std::path::Path;
use std::ops::Deref;
use std::io::Read;

use toml;

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct ServiceConfig {
    pub name : String,
    pub listen : Vec<String>,
    pub connect : Vec<String>,
}

impl ServiceConfig {
    pub fn server<A,B>(name : A, addr : B) -> Self
        where A : Deref<Target=str>, B : Deref<Target=str>
    {
        ServiceConfig {
            name : name.to_string(),
            listen : vec![addr.to_string()],
            connect : vec![],
        }
    }
    pub fn client<A,B>(name : A, addr : B) -> Self
        where A : Deref<Target=str>, B : Deref<Target=str>
    {
        ServiceConfig {
            name : name.to_string(),
            listen : vec![],
            connect : vec![addr.to_string()],
        }
    }
    pub fn from_toml(value : toml::Value) -> Self {
        toml::decode(value).unwrap()
    }
    pub fn from_file<P, A>(path : P, name : A) -> Self
        where P : AsRef<Path>, A : Deref<Target=str>
    {
        let mut file = File::open(path).unwrap();
        let mut st = String::new();
        file.read_to_string(&mut st).unwrap();
        let mut map = toml::Parser::new(&st).parse().unwrap();
        let value = map.remove(&*name).unwrap();
        Self::from_toml(value)
    }
}
