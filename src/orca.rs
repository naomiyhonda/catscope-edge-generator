use std::collections::VecDeque;

use solana_sdk::pubkey::Pubkey;

use crate::{
    primitive::{
        common::{match_discriminator, PUBKEY_LEN, U32_LEN, U64_LEN},
        filter::GuestFilter,
        header::AccountHeader,
        tree::{FilterEdge, WEIGHT_DIRECT, WEIGHT_IS_OUTGOING, WEIGHT_PROGRAM, WEIGHT_SYMLINK},
        wasmimport::HostImport,
    },
    DISCRIMINATOR_SIZE,
};

pub struct Orca {
    d_whirlpool: [u8; 8],
    d_whirlpoolconfig: [u8; 8],
    d_tickarray: [u8; 8],
    pub program_id: Pubkey,
}
impl GuestFilter for Orca {
    fn program_id_list(&self) -> Vec<Pubkey> {
        vec![self.program_id]
    }

    fn edge(&self, header: &AccountHeader, data: &[u8]) -> VecDeque<FilterEdge> {
        let mut list = VecDeque::new();
        let id = header.pubkey;
        // all discriminators are the same length
        let prefix = self.d_whirlpool.len();
        let mut i;
        let pubkey_len = std::mem::size_of::<Pubkey>();
        HostImport::log(format!(
            "orca_edge - 1 - pubkey {}; data len {}",
            id,
            data.len()
        ));
        if match_discriminator(&self.d_whirlpoolconfig, data) {
            HostImport::log(format!("orca_edge - 2 - whirlpoolconfig - pubkey {};", id));
            // fee authority
            {
                i = 8;
                let pubkey = Pubkey::try_from(&data[i..(i + pubkey_len)]).unwrap();
                list.push_back(FilterEdge {
                    slot: header.slot,
                    weight: WEIGHT_DIRECT,
                    from: id,
                    to: pubkey,
                });
            }
            // Collect Protocol Fees Authority
            {
                i = 40;
                let pubkey = Pubkey::try_from(&data[i..(i + pubkey_len)]).unwrap();
                list.push_back(FilterEdge {
                    slot: header.slot,
                    weight: WEIGHT_DIRECT,
                    from: id,
                    to: pubkey,
                });
            }
            // Reward Emissions Super Authority
            {
                i = 72;
                let pubkey = Pubkey::try_from(&data[i..(i + pubkey_len)]).unwrap();
                list.push_back(FilterEdge {
                    slot: header.slot,
                    weight: WEIGHT_DIRECT,
                    from: id,
                    to: pubkey,
                });
            }
        } else if match_discriminator(&self.d_whirlpool, data) {
            HostImport::log(format!("orca_edge - 2 - whirlpool - pubkey {};", id));
            // WhirlpoolsConfig
            {
                i = 8;
                let pubkey = Pubkey::try_from(&data[i..(i + pubkey_len)]).unwrap();
                list.push_back(FilterEdge {
                    slot: header.slot,
                    weight: WEIGHT_DIRECT,
                    from: pubkey,
                    to: id,
                });
            }
            // skip mint edges; they are useless as the mint information is available in the vault
            // accounts
            // token vault A
            {
                i = 133;
                let pubkey = Pubkey::try_from(&data[i..(i + pubkey_len)]).unwrap();
                list.push_back(FilterEdge {
                    slot: header.slot,
                    weight: WEIGHT_DIRECT,
                    from: id,
                    to: pubkey,
                });
            }
            // token vault B
            {
                i = 213;
                let pubkey = Pubkey::try_from(&data[i..(i + pubkey_len)]).unwrap();
                list.push_back(FilterEdge {
                    slot: header.slot,
                    weight: WEIGHT_DIRECT,
                    from: id,
                    to: pubkey,
                });
            }
            // rewards -> TBD
        } else if match_discriminator(&self.d_tickarray, data) {
            // whirlpool
            {
                i = prefix;
                let pubkey = Pubkey::try_from(&data[i..(i + pubkey_len)]).unwrap();
                list.push_back(FilterEdge {
                    slot: header.slot,
                    weight: WEIGHT_DIRECT,
                    from: pubkey,
                    to: id,
                });
            }
        }
        HostImport::log(format!("orca_edge - 4 - pubkey {};", id));
        list
    }
}

impl Orca {
    pub fn new(program_id: &Pubkey) -> Self {
        let d_whirlpool = whirlpool_discriminator();
        let d_whirlpoolconfig = whirlpoolconfig_discriminator();
        let d_tickarray = tickarray_discriminator();
        Self {
            program_id: *program_id,
            d_whirlpool,
            d_tickarray,
            d_whirlpoolconfig,
        }
    }
}
//  9d 14 31 e0 d9 57 c1 fe
pub fn whirlpoolconfig_discriminator() -> [u8; 8] {
    [157, 20, 49, 224, 217, 87, 193, 254]
}

// 3f 95 d1 0c e1 80 63 09
pub fn whirlpool_discriminator() -> [u8; 8] {
    [63, 149, 209, 12, 225, 128, 99, 9]
}
// 45 61 bd be 6e 07 42 bb
pub fn tickarray_discriminator() -> [u8; 8] {
    [69, 97, 189, 190, 110, 7, 66, 187]
}
