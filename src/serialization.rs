use std::{collections::HashMap, convert::Infallible};

use crate::{BSExp, vli::VLI};

// An BSEFile is structured as following:
// 1. an VLI $N$ for the number of entries of constant pool.
// 2. an VLI $M$ for the byte-length of the whole constant pool.
// 3. the constant pool:
//   - for $N$ entries, each entry is of:
//     - an VLI $L$, the byte-length of the data
//     - $L$ bytes, the data
// 4. an VLI $P$ at byte-position $M$, number of root nodes.
// 5. $P$ VLIs, the indices of root nodes.
// 6. an VLI $Q$ for the number of entries of the node pool.
// 7. an VLI $R$ for the byte-length of the whole node pool.
// 8. the node pool:
//   - for $Q$ entries, each entry is of:
//     - an VLI $L$, the number of entries of a list
//     - $L$ VLIs, that for each VLI as $x$:
//       - $(x & 0b1) == 0$: a atom by index $(x >> 1)$.
//       - $(x & 0b1) != 0$: a node by index $(x >> 1)$.

pub trait BSEFile {
    fn to_bsefile(&self) -> Vec<u8>;
    fn from_bsefile(binary_file: &[u8]) -> Self;
}

fn traverse_helper(
    x: &BSExp,
    atom_map: &mut HashMap<Vec<u8>, u64>,
    node_map: &mut HashMap<Vec<u64>, u64>,
    atom_buf: &mut Vec<u8>,
    node_buf: &mut Vec<u8>,
) -> u64 {
    todo!()
}

impl BSEFile for &[BSExp] {
    fn to_bsefile(&self) -> Vec<u8> {
        let mut atom_map = HashMap::new();
        let mut node_map = HashMap::new();
        let mut atom_buf = Vec::new();
        let mut node_buf = Vec::new();
        let root_indices = self.iter().map(|x| {
            traverse_helper(
                x,
                &mut atom_map,
                &mut node_map,
                &mut atom_buf,
                &mut node_buf,
            )
        });

        let mut file_buf = Vec::new();
        let mut file_buf_writer = |x| Result::<(), Infallible>::Ok(file_buf.push(x));

        (atom_map.len() as u64)
            .write_vli_bytes(file_buf_writer)
            .unwrap();
        file_buf
    }

    fn from_bsefile(binary_file: &[u8]) -> Self {
        todo!()
    }
}
