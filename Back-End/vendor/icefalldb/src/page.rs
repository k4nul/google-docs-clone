use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicUsize;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

use super::*;

trait PageCache {
    // data ops

    // NB don't let this grow beyond 4-8 (6 maybe ideal)
    fn delta(pid: PageID, delta: Delta) -> LogID;

    fn replace(pid: PageID, new_page: Page) -> LogID;

    /// return page. page table may only have disk ref, and will need to load it in
    fn read(pid: PageID) -> Data;

    // mgmt ops

    /// copies page into log I/O buf
    /// adds flush delta with caller annotation
    /// page table stores log addr
    /// may not yet be stable on disk
    fn flush(page_id: PageID, annotation: Vec<u8>) -> LogID;

    /// ensures all log data up until the provided address is stable
    fn make_stable(log_coords: LogID);

    /// returns current stable point in the log
    fn hi_stable() -> LogID;

    /// create new page, persist the table
    fn allocate() -> PageID;

    /// adds page to current epoch's pending freelist, persists table
    fn free(pid: PageID);

    // tx ops

    /// add a tx id (lsn) to tx table, maintained by CL
    fn tx_begin(id: TxID);

    /// tx removed from tx table
    /// tx is committed
    /// CAS page table
    /// tx flushed to LSS
    fn tx_commit(id: TxID);

    /// tx removed from tx table
    /// changed pages in cache are reset
    fn tx_abort(id: TxID);
}

#[derive(Debug, Clone)]
#[repr(C)]
struct Checkpoint {
    log_ids: HashMap<PageID, LogID>,
    free: Vec<PageID>,
    gc_pos: LogID,
    log_replay_idx: LogID,
}

#[derive(Debug, Clone)]
#[repr(C)]
struct PageLink {
    data: Data,
    pid: PageID,
    lid: Option<LogID>,
    child: Option<PageID>,
}

impl Encodable for Checkpoint {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("Checkpoint", 4, |s| {
            self.log_ids.encode(s)?;
            self.free.encode(s)?;
            self.gc_pos.encode(s)?;
            self.log_replay_idx.encode(s)
        })
    }
}

impl Decodable for Checkpoint {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("Checkpoint", 4, |d| {
            let log_ids = Decodable::decode(d)?;
            let free = Decodable::decode(d)?;
            let gc_pos = Decodable::decode(d)?;
            let log_replay_idx = Decodable::decode(d)?;
            Ok(Checkpoint {
                log_ids,
                free,
                gc_pos,
                log_replay_idx,
            })
        })
    }
}

impl Encodable for PageLink {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("PageLink", 4, |s| {
            self.data.encode(s)?;
            self.pid.encode(s)?;
            self.lid.encode(s)?;
            self.child.encode(s)
        })
    }
}

impl Decodable for PageLink {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("PageLink", 4, |d| {
            let data = Decodable::decode(d)?;
            let pid = Decodable::decode(d)?;
            let lid = Decodable::decode(d)?;
            let child = Decodable::decode(d)?;
            Ok(PageLink {
                data,
                pid,
                lid,
                child,
            })
        })
    }
}

struct Cache {
    highest_pid: PageID,
    heads: RwLock<BTreeMap<PageID, AtomicUsize>>,
    cached: RwLock<BTreeMap<LogID, Arc<Data>>>,
    log: Box<log::IOBufs>,
    // freelist managed as stack, biased to reuse low ID's
    free: Vec<PageID>,
}

struct ATT {

}
