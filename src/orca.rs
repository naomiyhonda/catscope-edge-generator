use std::collections::VecDeque;

use solana_sdk::pubkey::Pubkey;

#[cfg(target_os = "wasi")]
use crate::primitive::wasmimport::HostImport;
use crate::{
    primitive::{
        common::{match_discriminator, PUBKEY_LEN, U32_LEN, U64_LEN},
        guest::GuestFilter,
        header::AccountHeader,
        tree::{FilterEdge, WEIGHT_DIRECT, WEIGHT_IS_OUTGOING, WEIGHT_PROGRAM, WEIGHT_SYMLINK},
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
        #[cfg(target_os = "wasi")]
        HostImport::log(format!(
            "orca_edge - 1 - pubkey {}; data len {}",
            id,
            data.len()
        ));
        if match_discriminator(&self.d_whirlpoolconfig, data) {
            #[cfg(target_os = "wasi")]
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
            #[cfg(target_os = "wasi")]
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
        #[cfg(target_os = "wasi")]
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
    [184, 79, 171, 0, 183, 43, 113, 110]
}

// 3f 95 d1 0c e1 80 63 09
pub fn whirlpool_discriminator() -> [u8; 8] {
    [184, 79, 171, 0, 183, 43, 113, 110]
}
// 45 61 bd be 6e 07 42 bb
pub fn tickarray_discriminator() -> [u8; 8] {
    [184, 79, 171, 0, 183, 43, 113, 110]
}

// unit tests
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::primitive::tree::Weight;

    use super::*; // import functions from the parent module
    use hex;
    // dump with:  solana account --output-file=./whirlpool1.bin DtYKbQELgMZ3ihFUrCcCs9gy4djcUuhwgR7UpxVpP2Tg; xxd -p -c 99999999 ./whirlpool1.bin
    #[test]
    fn test_whirlpool() {
        let program_id = Pubkey::try_from("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc").unwrap();
        let data =
            hex::decode(std::fs::read_to_string("tests/data/whirlpool1.hex").unwrap()).unwrap();
        let header = AccountHeader {
            pubkey: Pubkey::try_from("DtYKbQELgMZ3ihFUrCcCs9gy4djcUuhwgR7UpxVpP2Tg").unwrap(),
            lamports: 488832,
            data_size: data.len() as u32,
            node_id: 100432,
            owner: Pubkey::try_from("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc").unwrap(),
            rent_epoch: 1,
            slot: 1,
            executable: false,
        };
        let filter = Orca::new(&program_id);
        let mut list = filter.edge(&header, &data);
        assert_eq!(list.len(), 3, "whirlpool must have 3 edges");
        let mut m_edge: HashMap<Pubkey, Weight> = HashMap::default();
        while let Some(edge) = list.pop_front() {
            if edge.from == header.pubkey {
                // from
                m_edge.insert(edge.to, edge.weight);
            } else {
                // to
                m_edge.insert(edge.from, edge.weight);
            }
        }
        let token_vault_a =
            Pubkey::try_from("3AfqjdDWMof5p2gEH4MRPZQyhDC36spx8GJk5LZJQRnP").unwrap();
        assert_eq!(*m_edge.get(&token_vault_a).unwrap(), WEIGHT_DIRECT);
        let token_vault_b =
            Pubkey::try_from("CdRr1RX5uFdJes33NiFiiG5TZNd6gvXWts9u9xjiCVRq").unwrap();
        assert_eq!(*m_edge.get(&token_vault_b).unwrap(), WEIGHT_DIRECT);
    }
}
