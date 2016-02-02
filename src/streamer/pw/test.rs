use std::fmt::Debug;
use serde::{Serialize, Deserialize};

use super::{Serializer, Deserializer, Error};

fn test_ser<T>(data : &T, expect_buf : &Vec<u8>) where T : Serialize + Deserialize + Eq + Debug{
    let mut buf = Vec::new();
    {
        let mut ser = Serializer::new(&mut buf);
        data.serialize(&mut ser).unwrap();
    }
    assert_eq!(buf, *expect_buf);
}

fn test_de<T>(expect_data : &T, buf: &Vec<u8>) where T : Serialize + Deserialize + Eq + Debug {
    let data : T = T::deserialize(&mut Deserializer::new(&buf[..])).unwrap();
    assert_eq!(data, *expect_data);
}

fn test_serde<T>(data : &T, buf: &Vec<u8>) where T : Serialize + Deserialize + Eq + Debug {
    test_ser(data, buf);
    test_de(data, buf);
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Struct {
    a : i32,
    b : String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum Enum {
    One,
    Two,
    Three(i32),
    Four(Struct),
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum MoreEnum {
    Five = 5,
    Six = 6,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum ProtocolFrom8000 {
    Seven(i32),
    Eight{i : i32},
    Nine(Struct),
}

#[test]
fn test1() {
    test_serde( 
        &Struct {
            a : 42,
            b : "fish".to_string(),
        },
        &vec![0, 0, 0, 42, 4, b'f', b'i', b's', b'h']
    );
}
#[test]
fn test2() {
    test_serde(
        &(42, "fish".to_string()),
        &vec![0, 0, 0, 42, 4, b'f', b'i', b's', b'h']
    );
}
#[test]
fn test3() {
    let data : (Option<u16>, Option<i32>, Result<i32, i16>) = (Some(42), None, Ok(42));
    test_serde(
        &data,
        &vec![1, 0, 42, 0, 0, 0, 0, 0, 42]
    );
}
#[test]
fn test4() {
    test_serde(
        &Enum::Three(666),
        &vec![2, 0, 0, 2, 154]
    );
}
#[test]
fn test5() {
    test_serde(
        &(Enum::Four(Struct { a : 1024, b : "good man".to_string() })),
        &vec![3, 0, 0, 4, 0, 8, b'g', b'o', b'o', b'd', b' ', b'm', b'a', b'n']
    );
}
#[test]
fn test6() {
    test_serde(
        &vec![1, 233, 666],
        &vec![3, 0, 0, 0, 1, 0, 0, 0, 233, 0, 0, 2, 154]
    );
}
#[test]
fn test7() {
    test_serde(
        b"binary",
        &vec![6, b'b', b'i', b'n', b'a', b'r', b'y']
    );
}
#[test] //serde does not respect the value of the enum, only the index
fn test8() {
    test_serde(
        &MoreEnum::Five,
        &vec![0]
    );
}
#[test]
fn test9() {
    test_serde(
        &ProtocolFrom8000::Seven(880),
        &vec![159, 64, 4, 0, 0, 3, 112]
    );
}
#[test]
fn test10() {
    test_serde(
        &ProtocolFrom8000::Eight{i:880},
        &vec![159, 65, 4, 0, 0, 3, 112]
    );
}
#[test]
fn test11() {
    test_serde(
        &ProtocolFrom8000::Nine(Struct{a:880,b:String::new()}),
        &vec![159, 66, 5, 0, 0, 3, 112, 0]
    );
}

