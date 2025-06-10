use std::{collections::HashMap, iter::repeat_with};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BSExp {
    Atom(Vec<u8>),
    List(Vec<BSExp>),
}

impl BSExp {
    pub fn atom<T: Into<Vec<u8>>>(value: T) -> Self {
        BSExp::Atom(value.into())
    }

    pub fn list<T: Into<Vec<BSExp>>>(value: T) -> Self {
        BSExp::List(value.into())
    }
}

impl std::fmt::Display for BSExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BSExp::Atom(a) => write!(f, "{}", String::from_utf8_lossy(a)),
            BSExp::List(l) => {
                let mut iter = l.iter();
                write!(f, "(")?;
                if let Some(first) = iter.next() {
                    write!(f, "{}", first)?;
                    for item in iter {
                        write!(f, " {}", item)?;
                    }
                }
                write!(f, ")")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom() {
        let atom = BSExp::atom("hello");
        assert_eq!(format!("{}", atom), "hello");
    }

    #[test]
    fn test_list() {
        let list = BSExp::list([BSExp::atom("a"), BSExp::atom("b")]);
        assert_eq!(format!("{}", list), "(a b)");
    }

    #[test]
    fn test_nested_list() {
        let nested = BSExp::list([
            BSExp::atom("a"),
            BSExp::list([BSExp::atom("b"), BSExp::atom("c")]),
        ]);
        assert_eq!(format!("{}", nested), "(a (b c))");
    }
}

// Idx is used to represent indices in the constant and node pools.
// It is a signed 32-bit integer, allowing for a range of -2,147,483,648 to 2,147,483,647.
// For node pools we use positive indices for nodes,
//   and negative indices are for atoms.
// For constant pool, positive indices are for constants,
//   and negative indices for special constants.

type Idx = i32;
impl BSExp {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut symbol_table = HashMap::new();
        let mut constant_pool = Vec::<u8>::new();
        let mut node_pool = Vec::new();
        self.serialize(&mut symbol_table, &mut constant_pool, &mut node_pool);

        // The first is the length of the constant pool (Idx)
        // The second is the length of the node pool (Idx)
        // The rest is the constant pool (byte array) followed by the node pool (Idx array)
        let mut result = Vec::new();
        result.extend((constant_pool.len() as Idx).to_le_bytes());
        result.extend((node_pool.len() as Idx).to_le_bytes());
        result.extend(constant_pool);
        result.extend(node_pool.into_iter().map(|idx| idx.to_le_bytes()).flatten());
        result
    }

    fn serialize(
        &self,
        symbol_table: &mut HashMap<Vec<u8>, Idx>,
        constant_pool: &mut Vec<u8>,
        node_pool: &mut Vec<Idx>,
    ) {
        match self {
            BSExp::Atom(a) => {
                if let Some(&index) = symbol_table.get(a) {
                    // We have seen this atom before, use its index
                    // Negative indices are used for atoms in the node pool
                    node_pool.push(!index);
                } else {
                    // This is a new atom, add it to the constant pool
                    let index = constant_pool.len() as Idx;
                    constant_pool.extend_from_slice((a.len() as Idx).to_le_bytes().as_ref());
                    constant_pool.extend_from_slice(a);
                    symbol_table.insert(a.clone(), index);
                    node_pool.push(!index);
                }
            }
            BSExp::List(l) => {
                // list head: a length
                node_pool.push(l.len() as Idx);
                for item in l {
                    item.serialize(symbol_table, constant_pool, node_pool);
                }
            }
        }
    }
}

fn read_idx<'a, I: Iterator<Item = &'a u8>>(iter: &mut I) -> Result<Idx, String> {
    let mut bytes = [0u8; 4];
    for i in 0..4 {
        if let Some(&byte) = iter.next() {
            bytes[i] = byte;
        } else {
            return Err("Not enough bytes to read.".to_string());
        }
    }
    Ok(Idx::from_le_bytes(bytes))
}

impl BSExp {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut cursor = bytes.iter();
        let constant_pool_length = read_idx(&mut cursor)? as usize;
        let node_pool_length = read_idx(&mut cursor)? as usize;

        // Read the constant pool
        let constant_pool = cursor
            .by_ref()
            .take(constant_pool_length)
            .cloned()
            .collect::<Vec<u8>>();

        let mut idx_cursor = repeat_with(|| read_idx(&mut cursor)).take(node_pool_length as usize);
        Self::deserialize(&mut idx_cursor, &constant_pool)
    }

    fn deserialize<'a, I: Iterator<Item = Result<Idx, String>>>(
        idx_cursor: &mut I,
        constant_pool: &Vec<u8>,
    ) -> Result<Self, String> {
        match idx_cursor.next() {
            // This is an empty list
            Some(Ok(0)) => Ok(BSExp::List(Vec::new())),
            // This is an atom
            Some(Ok(idx)) if idx < 0 => {
                let idx = !idx as usize;
                let len: &[u8; 4] = &constant_pool[idx..idx + 4]
                    .try_into()
                    .map_err(|_| "Invalid constant pool index.".to_string())?;
                let idx = idx + 4;
                let len = Idx::from_le_bytes(*len);
                Ok(BSExp::Atom(constant_pool[idx..idx + len as usize].to_vec()))
            }
            // This is a non-empty list
            Some(Ok(idx)) => Ok(BSExp::List(
                repeat_with(|| Self::deserialize(idx_cursor, constant_pool))
                    .take(idx as usize)
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            _ => Err("Invalid index encountered during deserialization.".to_string()),
        }
    }
}

#[cfg(test)]
mod serialization_tests {
    use super::*;

    #[test]
    fn test_empty_list() {
        let exp = BSExp::list([]);
        let bytes = exp.to_bytes();
        println!("Serialized bytes: {:?}", bytes);
        let deserialized = BSExp::from_bytes(&bytes).unwrap();
        assert_eq!(format!("{}", deserialized), "()");
    }

    #[test]
    fn test_single_atom() {
        let exp = BSExp::atom("test");
        let bytes = exp.to_bytes();
        println!("Serialized bytes: {:?}", bytes);
        let deserialized = BSExp::from_bytes(&bytes).unwrap();
        assert_eq!(format!("{}", deserialized), "test");
    }

    #[test]
    fn test_nested_list() {
        let exp = BSExp::list([
            BSExp::atom("a"),
            BSExp::list([BSExp::atom("b"), BSExp::atom("c")]),
        ]);
        let bytes = exp.to_bytes();
        println!("Serialized bytes: {:?}", bytes);
        let deserialized = BSExp::from_bytes(&bytes).unwrap();
        assert_eq!(format!("{}", deserialized), "(a (b c))");
    }

    #[test]
    fn test_complex_structure() {
        let exp = BSExp::list([
            BSExp::atom("x"),
            BSExp::list([BSExp::atom("y"), BSExp::atom("z")]),
            BSExp::atom("w"),
        ]);
        let bytes = exp.to_bytes();
        println!("Serialized bytes: {:?}", bytes);
        let deserialized = BSExp::from_bytes(&bytes).unwrap();
        assert_eq!(format!("{}", deserialized), "(x (y z) w)");
    }

    #[test]
    fn test_atom_reuse() {
        let exp1 = BSExp::atom("shared");
        let exp2 = BSExp::list([exp1.clone(), exp1]);
        let bytes = exp2.to_bytes();
        println!("Serialized bytes: {:?}", bytes);
        let deserialized = BSExp::from_bytes(&bytes).unwrap();
        assert_eq!(format!("{}", deserialized), "(shared shared)");
    }

    #[test]
    fn test_complex_reuse() {
        let exp1 = BSExp::atom("a");
        let exp2 = BSExp::atom("b");
        let exp3 = BSExp::list([exp1.clone(), exp2.clone()]);
        let exp4 = BSExp::list([exp3.clone(), exp1, exp2]);
        let bytes = exp4.to_bytes();
        println!("Serialized bytes: {:?}", bytes);
        let deserialized = BSExp::from_bytes(&bytes).unwrap();
        assert_eq!(format!("{}", deserialized), "((a b) a b)");
    }
}
