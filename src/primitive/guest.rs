use std::collections::VecDeque;

use solana_sdk::pubkey::Pubkey;

use super::header::AccountHeader;
use super::tree::FilterEdge;

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
