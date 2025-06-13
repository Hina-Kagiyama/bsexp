use std::iter::repeat_with;

/// Variant-Length Integer
/// This type can be serialized to 1 to 8 bytes
/// encoding atmost 2^56 bit data
pub trait VLI {
    fn to_vli_bytes(self) -> ([u8; 9], usize);

    fn read_vli_bytes<F, E>(reader: F) -> Result<Self, E>
    where
        Self: Sized,
        F: FnMut() -> Result<u8, E>;
}

impl VLI for u64 {
    #[inline]
    fn to_vli_bytes(self) -> ([u8; 9], usize) {
        let len = match self {
            _ if self < (0b1 << (1 * 7)) => 1,
            _ if self < (0b1 << (2 * 7)) => 2,
            _ if self < (0b1 << (3 * 7)) => 3,
            _ if self < (0b1 << (4 * 7)) => 4,
            _ if self < (0b1 << (5 * 7)) => 5,
            _ if self < (0b1 << (6 * 7)) => 6,
            _ if self < (0b1 << (7 * 7)) => 7,
            _ if self < (0b1 << (8 * 7)) => 8,
            _ => 9,
        };
        let msk = |n| if len > n { 0b1000_0000 } else { 0 };
        let sec = |m: u64, n: usize| ((self & (m << (n * 7))) >> (n * 7)) as u8;
        (
            [
                sec(0b0111_1111, 0) | msk(1), // - seg0: 7 bits, must exist
                sec(0b0111_1111, 1) | msk(2), // - seg1: 7 bits
                sec(0b0111_1111, 2) | msk(3), // - seg2: 7 bits
                sec(0b0111_1111, 3) | msk(4), // - seg3: 7 bits
                sec(0b0111_1111, 4) | msk(5), // - seg4: 7 bits
                sec(0b0111_1111, 5) | msk(6), // - seg5: 7 bits
                sec(0b0111_1111, 6) | msk(7), // - seg6: 7 bits
                sec(0b0111_1111, 7) | msk(8), // - seg7: 7 bits
                sec(0b1111_1111, 8),          // - seg8: 8 bits
            ],
            len,
        )
    }

    #[inline]
    fn read_vli_bytes<F, E>(mut reader: F) -> Result<Self, E>
    where
        F: FnMut() -> Result<u8, E>,
    {
        use std::ops::ControlFlow::{Break, Continue};
        match repeat_with(&mut reader)
            .take(8)
            .enumerate()
            .try_fold(0, |acc, (i, x)| match x {
                Ok(x) => {
                    if (x & 0b1000_0000) != 0 {
                        Continue(acc | (((x & 0b0111_1111) as u64) << (i * 7)))
                    } else {
                        Break(Ok(acc | ((x as u64) << (i * 7))))
                    }
                }
                Err(e) => Break(Err(e)),
            }) {
            Continue(x) => reader().map(|y| x | ((y & 0b1111_1111) as u64) << 56),
            Break(r) => r,
        }
    }
}

#[test]
fn test_vli_encode_decode() {
    // use core::convert::Infallible;
    use std::iter::repeat_with;
    // type I = Result<(), Infallible>;
    use VLI;

    // test writing
    let mut buf = Vec::new();
    let mut seed = 0x1234_5678_9ABC_DEF0u64;
    let v1 = repeat_with(|| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        seed
    })
    .take(1000)
    .collect::<Vec<_>>();
    v1.iter().map(|i| i.to_vli_bytes()).for_each(|(v, l)| {
        buf.extend_from_slice(&v[0..l]);
    });

    // test reading
    let mut cursor = buf.into_iter();

    assert!(
        repeat_with(|| <u64 as VLI>::read_vli_bytes(|| { cursor.next().ok_or(()) }))
            .map_while(|x| x.ok())
            .zip(v1.into_iter())
            .fold(true, |acc, (l, r)| { acc && (l == r) })
    )
}
