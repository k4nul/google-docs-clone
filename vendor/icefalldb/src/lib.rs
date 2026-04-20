#![allow(dead_code)]

extern crate libc;
extern crate rustc_serialize;
extern crate bincode;

use std::collections::HashMap;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

mod log;
mod map;
mod page;
mod rsdb;
mod tree;
mod tx;
mod crc16;
pub mod ops;

pub use rsdb::RSDB;
pub use log::Log;

use crc16::crc16_arr;

type PageID = u64;
type LogID = u64; // LogID == position to simplify file mapping
type TxID = u64;
type Epoch = u64;

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct KV {
    k: Vec<u8>,
    v: Vec<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(C)]
pub enum LogDelta {
    KV(KV),
    Merge {
        left: PageID,
        right: PageID,
    },
    Split {
        left: PageID,
        right: PageID,
    },
    FailedFlush, // on-disk only
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(C)]
pub enum Delta {
    KV(KV),
    Merge {
        left: PageID,
        right: PageID,
    },
    Split {
        left: PageID,
        right: PageID,
    },
    TxBegin(TxID), // in-mem
    TxCommit(TxID), // in-mem
    TxAbort(TxID), // in-mem
    Flush {
        pid: PageID,
        annotation: Annotation,
    }, // in-mem
    PartialSwap(LogID), /* indicates part of page has been swapped out,
                         * shows where to find it */
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct Page;

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(C)]
pub enum Data {
    Page(Page),
    Delta(Delta),
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct PD {
    data: Data,
    lid: LogID,
    pid: PageID,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct LogPage;

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct Annotation;

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(C)]
pub enum LogData {
    Full(LogPage),
    Deltas(Vec<LogDelta>),
}

impl Encodable for KV {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("KV", 2, |s| {
            self.k.encode(s)?;
            self.v.encode(s)
        })
    }
}

impl Decodable for KV {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("KV", 2, |d| {
            let k = Decodable::decode(d)?;
            let v = Decodable::decode(d)?;
            Ok(KV { k, v })
        })
    }
}

impl Encodable for LogDelta {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_enum("LogDelta", |s| match *self {
            LogDelta::KV(ref kv) => s.emit_enum_variant("KV", 0, 1, |s| {
                s.emit_enum_variant_arg(0, |s| kv.encode(s))
            }),
            LogDelta::Merge { left, right } => s.emit_enum_variant("Merge", 1, 2, |s| {
                s.emit_enum_variant_arg(0, |s| left.encode(s))?;
                s.emit_enum_variant_arg(1, |s| right.encode(s))
            }),
            LogDelta::Split { left, right } => s.emit_enum_variant("Split", 2, 2, |s| {
                s.emit_enum_variant_arg(0, |s| left.encode(s))?;
                s.emit_enum_variant_arg(1, |s| right.encode(s))
            }),
            LogDelta::FailedFlush => s.emit_enum_variant("FailedFlush", 3, 0, |_| Ok(())),
        })
    }
}

impl Decodable for LogDelta {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_enum("LogDelta", |d| {
            d.read_enum_variant(&["KV", "Merge", "Split", "FailedFlush"], |d, idx| {
                match idx {
                    0 => d.read_enum_variant_arg(0, Decodable::decode).map(LogDelta::KV),
                    1 => {
                        let left = d.read_enum_variant_arg(0, Decodable::decode)?;
                        let right = d.read_enum_variant_arg(1, Decodable::decode)?;
                        Ok(LogDelta::Merge { left, right })
                    }
                    2 => {
                        let left = d.read_enum_variant_arg(0, Decodable::decode)?;
                        let right = d.read_enum_variant_arg(1, Decodable::decode)?;
                        Ok(LogDelta::Split { left, right })
                    }
                    3 => Ok(LogDelta::FailedFlush),
                    _ => unreachable!(),
                }
            })
        })
    }
}

impl Encodable for Delta {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_enum("Delta", |s| match *self {
            Delta::KV(ref kv) => s.emit_enum_variant("KV", 0, 1, |s| {
                s.emit_enum_variant_arg(0, |s| kv.encode(s))
            }),
            Delta::Merge { left, right } => s.emit_enum_variant("Merge", 1, 2, |s| {
                s.emit_enum_variant_arg(0, |s| left.encode(s))?;
                s.emit_enum_variant_arg(1, |s| right.encode(s))
            }),
            Delta::Split { left, right } => s.emit_enum_variant("Split", 2, 2, |s| {
                s.emit_enum_variant_arg(0, |s| left.encode(s))?;
                s.emit_enum_variant_arg(1, |s| right.encode(s))
            }),
            Delta::TxBegin(ref id) => s.emit_enum_variant("TxBegin", 3, 1, |s| {
                s.emit_enum_variant_arg(0, |s| id.encode(s))
            }),
            Delta::TxCommit(ref id) => s.emit_enum_variant("TxCommit", 4, 1, |s| {
                s.emit_enum_variant_arg(0, |s| id.encode(s))
            }),
            Delta::TxAbort(ref id) => s.emit_enum_variant("TxAbort", 5, 1, |s| {
                s.emit_enum_variant_arg(0, |s| id.encode(s))
            }),
            Delta::Flush { pid, ref annotation } => {
                s.emit_enum_variant("Flush", 6, 2, |s| {
                    s.emit_enum_variant_arg(0, |s| pid.encode(s))?;
                    s.emit_enum_variant_arg(1, |s| annotation.encode(s))
                })
            }
            Delta::PartialSwap(ref lid) => s.emit_enum_variant("PartialSwap", 7, 1, |s| {
                s.emit_enum_variant_arg(0, |s| lid.encode(s))
            }),
        })
    }
}

impl Decodable for Delta {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_enum("Delta", |d| {
            d.read_enum_variant(
                &[
                    "KV",
                    "Merge",
                    "Split",
                    "TxBegin",
                    "TxCommit",
                    "TxAbort",
                    "Flush",
                    "PartialSwap",
                ],
                |d, idx| match idx {
                    0 => d.read_enum_variant_arg(0, Decodable::decode).map(Delta::KV),
                    1 => {
                        let left = d.read_enum_variant_arg(0, Decodable::decode)?;
                        let right = d.read_enum_variant_arg(1, Decodable::decode)?;
                        Ok(Delta::Merge { left, right })
                    }
                    2 => {
                        let left = d.read_enum_variant_arg(0, Decodable::decode)?;
                        let right = d.read_enum_variant_arg(1, Decodable::decode)?;
                        Ok(Delta::Split { left, right })
                    }
                    3 => d.read_enum_variant_arg(0, Decodable::decode).map(Delta::TxBegin),
                    4 => d.read_enum_variant_arg(0, Decodable::decode).map(Delta::TxCommit),
                    5 => d.read_enum_variant_arg(0, Decodable::decode).map(Delta::TxAbort),
                    6 => {
                        let pid = d.read_enum_variant_arg(0, Decodable::decode)?;
                        let annotation = d.read_enum_variant_arg(1, Decodable::decode)?;
                        Ok(Delta::Flush { pid, annotation })
                    }
                    7 => d
                        .read_enum_variant_arg(0, Decodable::decode)
                        .map(Delta::PartialSwap),
                    _ => unreachable!(),
                },
            )
        })
    }
}

impl Encodable for Page {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("Page", 0, |_| Ok(()))
    }
}

impl Decodable for Page {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("Page", 0, |_| Ok(Page))
    }
}

impl Encodable for Data {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_enum("Data", |s| match *self {
            Data::Page(ref page) => s.emit_enum_variant("Page", 0, 1, |s| {
                s.emit_enum_variant_arg(0, |s| page.encode(s))
            }),
            Data::Delta(ref delta) => s.emit_enum_variant("Delta", 1, 1, |s| {
                s.emit_enum_variant_arg(0, |s| delta.encode(s))
            }),
        })
    }
}

impl Decodable for Data {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_enum("Data", |d| {
            d.read_enum_variant(&["Page", "Delta"], |d, idx| match idx {
                0 => d.read_enum_variant_arg(0, Decodable::decode).map(Data::Page),
                1 => d.read_enum_variant_arg(0, Decodable::decode).map(Data::Delta),
                _ => unreachable!(),
            })
        })
    }
}

impl Encodable for PD {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("PD", 3, |s| {
            self.data.encode(s)?;
            self.lid.encode(s)?;
            self.pid.encode(s)
        })
    }
}

impl Decodable for PD {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("PD", 3, |d| {
            let data = Decodable::decode(d)?;
            let lid = Decodable::decode(d)?;
            let pid = Decodable::decode(d)?;
            Ok(PD { data, lid, pid })
        })
    }
}

impl Encodable for LogPage {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("LogPage", 0, |_| Ok(()))
    }
}

impl Decodable for LogPage {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("LogPage", 0, |_| Ok(LogPage))
    }
}

impl Encodable for Annotation {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("Annotation", 0, |_| Ok(()))
    }
}

impl Decodable for Annotation {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("Annotation", 0, |_| Ok(Annotation))
    }
}

impl Encodable for LogData {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_enum("LogData", |s| match *self {
            LogData::Full(ref page) => s.emit_enum_variant("Full", 0, 1, |s| {
                s.emit_enum_variant_arg(0, |s| page.encode(s))
            }),
            LogData::Deltas(ref deltas) => s.emit_enum_variant("Deltas", 1, 1, |s| {
                s.emit_enum_variant_arg(0, |s| deltas.encode(s))
            }),
        })
    }
}

impl Decodable for LogData {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_enum("LogData", |d| {
            d.read_enum_variant(&["Full", "Deltas"], |d, idx| match idx {
                0 => d.read_enum_variant_arg(0, Decodable::decode).map(LogData::Full),
                1 => d
                    .read_enum_variant_arg(0, Decodable::decode)
                    .map(LogData::Deltas),
                _ => unreachable!(),
            })
        })
    }
}
