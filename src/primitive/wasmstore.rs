use core::slice;
use std::collections::{HashMap, VecDeque};

use solana_sdk::pubkey::Pubkey;

use super::{
    common::{find_k, slice_to_struct, DATA_LIST},
    err::CatscopeWasmError,
    header::{AccountHeader, AccountId},
    tree::FilterEdge,
};

/// Store a pool of byte arrays with fixed size.
#[derive(Debug)]
struct SingleStore {
    layer_index: usize, // if this file store is part of an array, set the index of that array here
    unit_size: usize,
    // what is the latest unit that had been created
    q_empty: VecDeque<usize>, // do not clone
    m_slice: HashMap<usize, Vec<u8>>,
}
impl SingleStore {
    pub fn new(layer_index: usize, unit_size: usize) -> Self {
        Self {
            layer_index,
            unit_size,
            q_empty: VecDeque::new(),
            m_slice: HashMap::new(),
        }
    }
    pub fn allocate(&mut self, target_size: usize) -> GuestBlob {
        let ptr = if let Some(p) = self.q_empty.pop_front() {
            p
        } else {
            let d = vec![0u8; self.unit_size];
            let p = d.as_ptr() as usize;
            self.m_slice.insert(p, d);
            p
        };
        GuestBlob {
            ptr,
            len: target_size,
            capacity: self.unit_size,
        }
    }
    pub fn deallocate(&mut self, ptr: usize) {
        let mut erase = false;
        if self.m_slice.contains_key(&ptr) {
            if self.q_empty.len() < 5 {
                self.q_empty.push_back(ptr);
            } else {
                // deallocate
                erase = true;
            }
        }
        if erase {
            self.m_slice.remove(&ptr);
        }
    }
}

#[derive(Debug, Clone)]
pub struct GuestBlob {
    ptr: usize,
    len: usize,
    capacity: usize,
}

impl GuestBlob {
    pub fn pointer(&self) -> u64 {
        self.ptr as u64
    }
    pub fn slice<'a, 'b: 'a>(&self) -> &[u8] {
        let ptr = self.ptr as *const u8;
        unsafe { slice::from_raw_parts(ptr, self.len) }
    }
    pub fn slice_mut<'a, 'b: 'a>(&mut self) -> &mut [u8] {
        let ptr = self.ptr as *mut u8;
        unsafe { slice::from_raw_parts_mut(ptr, self.len) }
    }
    // TODO: make this return an error if the slice size is wrong.
    pub fn payload<'a, 'b: 'a, T: Default>(&self) -> &T {
        let slice = self.slice();
        unsafe { &*(slice.as_ptr() as *const T) }
    }
    pub fn payload_mut<'a, 'b: 'a, T: Default>(&mut self) -> &mut T {
        let slice = self.slice_mut();
        unsafe { &mut *(slice.as_mut_ptr() as *mut T) }
    }
}

pub struct Store {
    data: Vec<SingleStore>,
    m_general: HashMap<usize, Vec<u8>>,
    m_ptr_to_size: HashMap<usize, usize>,
}
impl Default for Store {
    fn default() -> Self {
        let mut data = Vec::new();
        for (k, unit_size) in DATA_LIST.into_iter().enumerate() {
            let single = SingleStore::new(k, unit_size);
            data.push(single);
        }
        Self {
            data,
            m_ptr_to_size: HashMap::new(),
            m_general: HashMap::new(),
        }
    }
}
impl Store {
    pub fn ptr_len_to_blob(&self, ptr: u64, size: u32) -> Option<GuestBlob> {
        let k = find_k(&DATA_LIST, size as usize)?;
        let capacity = self.data[k].unit_size;
        Some(GuestBlob {
            ptr: ptr as usize,
            len: size as usize,
            capacity,
        })
    }
    pub fn allocate(&mut self, data_size: usize) -> Option<GuestBlob> {
        let mut vec = vec![0u8; data_size];
        let ptr = vec.as_mut_ptr();
        let gb = GuestBlob {
            ptr: ptr as usize,
            len: vec.len(),
            capacity: vec.len(),
        };
        self.m_general.insert(ptr as usize, vec);
        Some(gb)
    }
    pub fn allocate2(&mut self, data_size: usize) -> Option<GuestBlob> {
        let k = find_k(&DATA_LIST, data_size)?;
        if 0 < data_size {
            let mut gb = self.data[k].allocate(data_size);
            let slice = gb.slice_mut();
            for i in 0..slice.len() {
                slice[i] = 0;
            }
            self.m_ptr_to_size.insert(gb.ptr, gb.capacity);
            Some(gb)
        } else {
            None
        }
    }

    pub fn allocate_struct<T: Default>(&mut self) -> Option<GuestBlob> {
        self.allocate(std::mem::size_of::<T>())
    }

    pub fn recover_blob(&self, ptr: usize, expected_size: usize) -> Option<GuestBlob> {
        if let Some(vec) = self.m_general.get(&ptr) {
            let gb = GuestBlob {
                ptr,
                len: vec.len(),
                capacity: vec.len(),
            };
            return Some(gb);
        }
        if let Some(size) = self.m_ptr_to_size.get(&ptr) {
            if size.le(&expected_size) {
                panic!("unexpected size: {} vs {}", size, expected_size);
            }
            if let Some(k) = find_k(&DATA_LIST, *size) {
                let capacity = self.data[k].unit_size;
                return Some(GuestBlob {
                    ptr,
                    len: expected_size,
                    capacity,
                });
            }
        }
        None
    }
    pub fn recover_struct<T: Default>(&self, ptr: usize) -> Option<GuestBlob> {
        self.recover_blob(ptr, std::mem::size_of::<T>())
    }
    pub fn deallocate(&mut self, ptr: u64) {
        let p = ptr as usize;
        if self.m_general.remove(&p).is_some() {
            return;
        }
        let p = ptr as usize;
        if let Some(size) = self.m_ptr_to_size.get(&p) {
            if let Some(k) = find_k(&DATA_LIST, *size) {
                self.data[k].deallocate(ptr as usize);
            }
        }
    }
}

/// This struct is the WASM guest equivalent of Account.
pub struct AccountOnGuest {
    blob: GuestBlob,
}
impl AccountOnGuest {
    pub fn header_size() -> usize {
        std::mem::size_of::<AccountHeader>()
    }
    pub fn header(&self) -> &AccountHeader {
        let slice = self.blob.slice();
        //        HostImport::log(format!("header {:?}", slice));
        slice_to_struct(slice, 0).unwrap()
        //  let headerbuf = &slice[0..Self::header_size()];
        // let header: &AccountHeader = unsafe { &*(headerbuf.as_ptr() as *const AccountHeader) };
        //        header
    }
    pub fn data(&self) -> &[u8] {
        let slice = self.blob.slice();
        &slice[Self::header_size()..]
    }
}
impl TryFrom<GuestBlob> for AccountOnGuest {
    type Error = CatscopeWasmError;

    fn try_from(blob: GuestBlob) -> Result<Self, Self::Error> {
        let slice = blob.slice();
        if slice.len() < Self::header_size() {
            return Err(CatscopeWasmError::InsufficientBuffer);
        }
        Ok(Self { blob })
    }
}

#[repr(C, align(8))]
#[derive(Debug, Clone, Default)]
pub struct FilterEdgeWithNextPointer {
    pub is_empty: bool, // return true if there are no edges
    pub edge: FilterEdge,
    pub next_pointer: u64,
}

#[repr(C, align(8))]
#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
pub struct PublicKeyWithNode {
    pub node_id: AccountId,
    pub pubkey: Pubkey,
}
