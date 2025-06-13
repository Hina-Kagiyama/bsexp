/// Variant-Length Integer
/// This type can be serialized to 1 to 8 bytes
/// encoding atmost 2^56 bit data
pub trait VLI {
    fn write_vli_bytes<F, E>(self, writer: F) -> Result<usize, E>
    where
        F: FnMut(u8) -> Result<(), E>;

    fn read_vli_bytes<F, E>(reader: F) -> Result<Self, E>
    where
        Self: Sized,
        F: FnMut() -> Result<u8, E>;
}

const MAX_ENCODING_LENGTH: usize = 9;

impl VLI for u64 {
    #[inline]
    fn write_vli_bytes<F, E>(self, mut writer: F) -> Result<usize, E>
    where
        F: FnMut(u8) -> Result<(), E>,
    {
        let tmp = self as u64;
        assert!((tmp & (0b1 << 63) == 0));

        // segments:
        // 7 bits + (7 bits + 7 bits + 7 bits + 7 bits + 7 bits + 7 bits + 7 bits)
        let segs = [
            (tmp & 0b0111_1111) as u8,
            ((tmp & (0b0111_1111 << (1 * 7))) >> (1 * 7)) as u8,
            ((tmp & (0b0111_1111 << (2 * 7))) >> (2 * 7)) as u8,
            ((tmp & (0b0111_1111 << (3 * 7))) >> (3 * 7)) as u8,
            ((tmp & (0b0111_1111 << (4 * 7))) >> (4 * 7)) as u8,
            ((tmp & (0b0111_1111 << (5 * 7))) >> (5 * 7)) as u8,
            ((tmp & (0b0111_1111 << (6 * 7))) >> (6 * 7)) as u8,
            ((tmp & (0b0111_1111 << (7 * 7))) >> (7 * 7)) as u8,
            ((tmp & (0b0111_1111 << (8 * 7))) >> (8 * 7)) as u8,
        ];

        let len = MAX_ENCODING_LENGTH - segs.iter().cloned().rev().take_while(|&x| x == 0).count();
        let mut it = segs.into_iter();
        (&mut it)
            .take(len - 1)
            .try_for_each(|x| writer(x | 0b1000_0000))?;
        it.take(1).try_for_each(|x| writer(x)).map(|()| len)
    }

    #[inline]
    fn read_vli_bytes<F, E>(mut reader: F) -> Result<Self, E>
    where
        F: FnMut() -> Result<u8, E>,
    {
        let mut ans = 0;
        for _ in 0..MAX_ENCODING_LENGTH {
            let x = reader()? as u64;
            ans <<= 7;
            ans |= x & (0b0111_1111);
            if (x & 0b1000_0000) == 0 {
                break;
            }
        }
        Ok(ans)
    }
}

#[test]
fn test_vli_encode_decode() {
    use core::convert::Infallible;
    use std::iter::repeat_with;
    type I = Result<(), Infallible>;
    use VLI;

    // test writing
    let mut buf = Vec::new();
    let v1 = [
        127,
        16383,
        2097151,
        268435455,
        34359738367,
        4398046511103,
        562949953421311,
        72057594037927935,
        9223372036854775807,
        // 18446744073709551615,
    ];
    v1.iter().for_each(|i| {
        i.write_vli_bytes(|x| I::Ok(buf.push(x))).unwrap();
    });

    // test reading
    let mut cursor = buf.into_iter();

    assert_eq!(
        repeat_with(|| {
            <u64 as VLI>::read_vli_bytes(|| match cursor.next() {
                Some(x) => Ok(x),
                None => Err(()),
            })
        })
        .map_while(|x| match x {
            Ok(x) => Some(x),
            Err(_) => None,
        })
        .zip(v1.into_iter())
        .fold(true, |acc, (l, r)| {
            println!("{l:064b} {r:064b}");
            acc && (l == r)
        }),
        true
    )
}
