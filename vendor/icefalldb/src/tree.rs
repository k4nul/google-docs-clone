#![allow(dead_code)]

use std::io;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

use super::*;

type RetVal = io::Result<Option<Vec<u8>>>;
type MultiRetVal = io::Result<Vec<Option<Vec<u8>>>>;

pub trait Tree {
    fn get(&self, k: Vec<u8>) -> RetVal;
    fn set(&self, k: Vec<u8>, v: Vec<u8>) -> RetVal;
    fn del(&self, k: Vec<u8>) -> RetVal;
    fn mget(&self, ks: Vec<Vec<u8>>) -> MultiRetVal;
}

#[derive(Debug, Clone)]
#[repr(C)]
struct TreePage {
    id: PageID,
    low_key: Vec<u8>,
    high_key: Vec<u8>,
    next: Option<PageID>,
    prev: Option<PageID>,
    children: Children,
}


#[derive(Debug, Clone)]
#[repr(C)]
enum Children {
    Data {
        kvs: Vec<KV>,
    },
    Index {
        seps: Vec<Vec<u8>>,
        ptrs: Vec<PageID>,
    },
}

impl Encodable for TreePage {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("TreePage", 6, |s| {
            self.id.encode(s)?;
            self.low_key.encode(s)?;
            self.high_key.encode(s)?;
            self.next.encode(s)?;
            self.prev.encode(s)?;
            self.children.encode(s)
        })
    }
}

impl Decodable for TreePage {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("TreePage", 6, |d| {
            let id = Decodable::decode(d)?;
            let low_key = Decodable::decode(d)?;
            let high_key = Decodable::decode(d)?;
            let next = Decodable::decode(d)?;
            let prev = Decodable::decode(d)?;
            let children = Decodable::decode(d)?;
            Ok(TreePage {
                id,
                low_key,
                high_key,
                next,
                prev,
                children,
            })
        })
    }
}

impl Encodable for Children {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_enum("Children", |s| match *self {
            Children::Data { ref kvs } => s.emit_enum_variant("Data", 0, 1, |s| {
                s.emit_enum_variant_arg(0, |s| kvs.encode(s))
            }),
            Children::Index {
                ref seps,
                ref ptrs,
            } => s.emit_enum_variant("Index", 1, 2, |s| {
                s.emit_enum_variant_arg(0, |s| seps.encode(s))?;
                s.emit_enum_variant_arg(1, |s| ptrs.encode(s))
            }),
        })
    }
}

impl Decodable for Children {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_enum("Children", |d| {
            d.read_enum_variant(&["Data", "Index"], |d, idx| match idx {
                0 => d.read_enum_variant_arg(0, Decodable::decode).map(|kvs| Children::Data {
                    kvs,
                }),
                1 => {
                    let seps = d.read_enum_variant_arg(0, Decodable::decode)?;
                    let ptrs = d.read_enum_variant_arg(1, Decodable::decode)?;
                    Ok(Children::Index { seps, ptrs })
                }
                _ => unreachable!(),
            })
        })
    }
}
