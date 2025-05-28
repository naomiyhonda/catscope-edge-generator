use std::collections::HashMap;

use once_cell::sync::Lazy;
use solana_sdk::{clock::Slot, pubkey::Pubkey};

use super::err::CatscopeWasmError;

pub type Weight = u32;
pub const WEIGHT_IS_OUTGOING: Weight = 1 << 0;
pub const WEIGHT_SLOT: Weight = 1 << 1;
// this is for the host to map accounts to a program for when clients sync.
pub const WEIGHT_CLIENT: Weight = 1 << 2;
/// upload the destination node to a client.
pub const WEIGHT_UPLOAD: Weight = 1 << 3;

pub const WEIGHT_NON_ACCOUNT: Weight =
    WEIGHT_SLOT | WEIGHT_CLIENT | WEIGHT_UPLOAD | WEIGHT_IS_OUTGOING;
pub const WEIGHT_ACCOUNT: Weight = !WEIGHT_NON_ACCOUNT;

pub const MAX_WEIGHT_NONACCOUNT_EXPONENT: u8 = 3;

pub const WEIGHT_PROGRAM: Weight = 1 << 4;
pub const WEIGHT_SPLTOKEN_OWNER: Weight = 1 << 5;
pub const WEIGHT_SPLTOKEN_MINT: Weight = 1 << 6;
pub const WEIGHT_DIRECT: Weight = 1 << 7;
pub const WEIGHT_SYMLINK: Weight = 1 << 8;

// the memory requirements grow exponentionally (doubles) for every integer increment of this
// variable.
pub const MAX_WEIGHT_ACCOUNT_EXPONENT: u8 = 9;
pub const MAX_WEIGHT: Weight = 1 << MAX_WEIGHT_ACCOUNT_EXPONENT;

/// Index weights for use when updating subscriptions.
static WEIGHT_HASH_MAP: Lazy<HashMap<Weight, Vec<Weight>>> = Lazy::new(|| {
    let mut lookup: HashMap<Weight, Vec<Weight>> = HashMap::new();
    // 0..32
    for mask in 0..(1 << (MAX_WEIGHT_ACCOUNT_EXPONENT - MAX_WEIGHT_NONACCOUNT_EXPONENT)) {
        let mut subset = mask;
        // Generate all subsets (masks) of 'b'
        loop {
            lookup
                .entry(mask)
                .or_default()
                .push(subset << MAX_WEIGHT_NONACCOUNT_EXPONENT);
            if subset == 0 {
                break;
            }
            subset = (subset - 1) & mask; // Generate next subset
        }
    }
    lookup
});

#[repr(C, align(8))]
#[derive(Debug, Clone, Default)]
pub struct ProgramList {
    pub count: u16,
    pub list: [Pubkey; 32], // have a max length
}

/// The edge goes in the graph determined by the `from` `program_id`.
/// `from` is the account_id.
#[repr(C, align(8))]
#[derive(Debug, Clone, Default)]
pub struct FilterEdge {
    pub slot: Slot,
    pub to: Pubkey,
    pub from: Pubkey,
    pub weight: Weight, // weight of zero is not allowed
}
impl FilterEdge {
    pub fn from_raw_parts<'a, 'b: 'a>(data: &'b [u8]) -> Result<&'a Self, CatscopeWasmError> {
        if data.len() < std::mem::size_of::<Self>() {
            return Err(CatscopeWasmError::InsufficientBuffer);
        }
        let filter: &Self = unsafe { &*(data.as_ptr() as *const Self) };
        Ok(filter)
    }
    pub fn set_outgoing(&mut self, id: &Pubkey) {
        if self.from.eq(id) {
            self.weight |= WEIGHT_IS_OUTGOING;
        }
    }
}

#[inline(always)]
pub fn edge_is_outgoing(weight: &Weight) -> bool {
    0 < *weight & WEIGHT_IS_OUTGOING
}

#[inline]
fn zero_out_bits_above_n(value: u32, n: u32) -> u32 {
    // Create a mask with 1s for bits <= n and 0s for bits > n.
    // We use wrapping_shl to handle potential overflow if n is close to 32.
    let mask = if n >= 31 {
        u32::MAX // If n is 31 or more, keep all bits.
    } else {
        (1u32 << (n + 1)).wrapping_sub(1) // Create the mask (e.g., n=2 -> 0b111).
    };

    value & mask // Apply the mask to zero out the unwanted bits.
}

// Return all bit map combinations covered by this weight.
pub fn weight_list(weight: &Weight) -> &'static [Weight] {
    let reduced = zero_out_bits_above_n(*weight, MAX_WEIGHT_ACCOUNT_EXPONENT as u32 - 1);
    let k = reduced >> MAX_WEIGHT_NONACCOUNT_EXPONENT;
    let result = match WEIGHT_HASH_MAP.get(&k) {
        Some(x) => x,
        None => panic!(
            "failed to get k {}; reduced {}; weight {};",
            k, reduced, weight
        ),
    };
    result
}
pub fn parse_program_list(input: &[u8]) -> Result<Vec<Pubkey>, CatscopeWasmError> {
    let input_str = match std::str::from_utf8(input) {
        Ok(x) => x,
        Err(e) => return Err(CatscopeWasmError::Unknown(e.to_string())),
    };
    let pre_list: Vec<&str> = input_str.split(',').collect();
    let mut list = Vec::with_capacity(pre_list.len());
    for i in 0..pre_list.len() {
        let y = pre_list[i].trim();
        let x: Pubkey = match y.try_into() {
            Ok(z) => z,
            Err(e) => return Err(CatscopeWasmError::Unknown(e.to_string())),
        };
        list.push(x);
    }
    Ok(list)
}
