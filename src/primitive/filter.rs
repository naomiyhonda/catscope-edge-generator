use super::{
    header::AccountHeader,
    tree::{FilterEdge, WEIGHT_IS_OUTGOING},
    wasmimport::HostImport,
    wasmstore::Store,
};
use solana_sdk::pubkey::Pubkey;
use std::collections::{BTreeMap, VecDeque};

// This trait has to be implemented by a guest wasm.
pub trait GuestFilter {
    /// Write a list of all program ids to be tracked.
    /// The index ( `usize` ) are used to mark Program IDs through Catscope code to save space.
    fn program_id_list(&self) -> Vec<Pubkey>;
    /// Return the Pubkey of the parent account.
    /// self is marked mutable to allow the writing of account data to a shared buffer.
    /// return -1 for failure, 0 for no parent, 1 for parent.
    fn edge(&self, header: &AccountHeader, data: &[u8]) -> VecDeque<FilterEdge>;
}

#[repr(C, align(8))]
pub struct CatscopeFilter {
    host_import: HostImport,
    l_program_id: Vec<Pubkey>,
    l_filter: Vec<Box<dyn GuestFilter + 'static>>,
    m_filter_by_pubkey: BTreeMap<Pubkey, usize>, // program id pubkey to filter index
    m_filter_by_listindex: BTreeMap<usize, usize>, // program id pubkey to filter index
}
impl CatscopeFilter {
    pub fn new(
        mut list: VecDeque<Box<dyn GuestFilter + 'static>>,
        host_import: HostImport,
    ) -> Self {
        let mut m_filter_by_pubkey = BTreeMap::new();
        let mut l_filter = Vec::new();
        let mut l_program_id = Vec::new();
        let mut m_filter_by_listindex = BTreeMap::new();
        let mut k = 0;
        let mut j = 0;
        while let Some(filter) = list.pop_front() {
            let list = filter.program_id_list();
            for pubkey in list.iter() {
                m_filter_by_pubkey.insert(*pubkey, k);
                m_filter_by_listindex.insert(j, k);
                l_program_id.push(*pubkey);
                j += 1;
            }
            l_filter.push(filter);
            k += 1;
        }
        Self {
            host_import,
            l_program_id,
            l_filter,
            m_filter_by_listindex,
            m_filter_by_pubkey,
        }
    }
    pub fn store(&self) -> &Store {
        self.host_import.store()
    }
    pub fn store_mut(&mut self) -> &mut Store {
        self.host_import.store_mut()
    }
}

impl GuestFilter for CatscopeFilter {
    fn program_id_list(&self) -> Vec<Pubkey> {
        self.l_program_id.clone()
    }

    fn edge(&self, header: &AccountHeader, data: &[u8]) -> VecDeque<FilterEdge> {
        let program_id = &header.owner;
        let o_k = self.m_filter_by_pubkey.get(program_id);
        if o_k.is_none() {
            return VecDeque::new();
        }
        let mut list = if let Some(filter) = self.l_filter.get(*o_k.unwrap()) {
            filter.edge(header, data)
        } else {
            VecDeque::new()
        };
        // make sure to set the weights to indicate the direction of the edge.
        // We do this because edges are sent independently of accounts.
        list.iter_mut().for_each(|a| {
            a.set_outgoing(&header.pubkey);
        });
        list
    }
}

pub fn weight_set_outgoing(edge: &mut FilterEdge) {
    edge.weight |= WEIGHT_IS_OUTGOING;
}

// convert a pointer to a filter.
pub fn ptr_to_filter<'a, T>(buffer_ptr: u64) -> Option<&'a mut T> {
    let cat_ptr = buffer_ptr as *mut T;
    if cat_ptr.is_null() {
        return None;
    }
    let s = unsafe { cat_ptr.as_mut().unwrap() };
    Some(s)
}
