# BSExp: A binary S-Expression serialization / deserialization library for Rust

## Representation

BSExp is a binary serialization format for S-expressions, designed to be efficient and compact. It uses a simple structure to represent atoms, lists, and other data types in a binary format.

A serialized BSExp, have the following structure:

- The first _size number_ is the length of the constant pool.
- The next _size number_ is the length of the data section.
- The next bytes are the constant pool, which contains all the unique atoms used in the S-expression.
- The next bytes are the data section, which contains the serialized S-expression.

Where the _size number_ is a 64-bit unsigned integer (8 bytes) in little-endian format.
This might be changed in the future to allow a variable size number, but for now it is fixed at 8 bytes.

For each atom in the constant pool, it is serialized as a _size number_ followed by the encoded bytes of the atom. The _size number_ indicates the length of the atom in bytes. The encoding of the atom is not specified, and it can be any binary format, such as UTF-8 for strings or a custom binary format for other types like integers.

For the data section, the first _size number_ $s$ indicates the type of the data:

- $s < 0$: The data is an atom, and $bitwise_not s$ is the index of the atom in the constant pool.
- $s = 0$: The data is an empty list.
- $s > 0$: The data is a list, and $s$ is the number of elements in the list. Then followed by the serialized elements of the list, which can be atoms or other lists.

## Example

```rust
use bsexp::BSExp;

fn main() -> Result<(), String> {
    // Example S-expression: (a b (c a))
    let s_expr = BSExp::list(vec![
        BSExp::atom("a"),
        BSExp::atom("b"),
        BSExp::list(vec![
            BSExp::atom("c"),
            BSExp::atom("a"),
        ]),
    ]);

    // Serialize the S-expression to a byte vector
    let mut buffer = s_expr.to_bytes(&mut buffer);

    // Deserialize the byte vector back to an S-expression
    let deserialized_s_expr = BSExp::from_bytes(&buffer)?;

    // Print the deserialized S-expression
    println!("{deserialized_s_expr}");

    Ok(())
}
```
