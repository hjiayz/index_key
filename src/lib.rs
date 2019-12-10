//! lexicographic sort order encoding.

pub trait IndexKey: Sized {
    fn to_key(&self) -> Vec<u8>;
    fn from_key(key: &[u8]) -> Self;
}

impl IndexKey for String {
    #[inline]
    fn to_key(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
    #[inline]
    fn from_key(src: &[u8]) -> String {
        String::from_utf8_lossy(src).to_string()
    }
}

#[test]
fn test_string() {
    let s: String = "123".into();
    assert_eq!(String::from_key(&s.to_key()), s)
}

impl IndexKey for Vec<u8> {
    fn to_key(&self) -> Vec<u8> {
        self.to_vec()
    }
    fn from_key(src: &[u8]) -> Vec<u8> {
        src.to_vec()
    }
}

macro_rules! from_slice {
    ($t:ty,$src:expr) => {{
        let mut val = [0u8; std::mem::size_of::<$t>()];
        for (v, s) in val.iter_mut().zip($src) {
            *v = *s;
        }
        val
    }};
}

macro_rules! impl_u {
    ($t:ident) => {
        impl IndexKey for $t {
            fn to_key(&self) -> Vec<u8> {
                self.to_be_bytes().to_vec()
            }
            fn from_key(src: &[u8]) -> $t {
                <$t>::from_be_bytes(from_slice!($t, src))
            }
        }
        #[test]
        fn $t() {
            use std::$t::*;
            let mut list = vec![MAX, 1, 2, 0];
            list.sort_by_key(|value| {
                assert_eq!(<$t>::from_key(&value.to_key()), *value);
                value.to_key()
            });
            assert_eq!(list, vec![0, 1, 2, MAX]);
        }
    };
}

impl_u!(u8);
impl_u!(u16);
impl_u!(u32);
impl_u!(u64);
impl_u!(u128);

macro_rules! impl_i {
    ($t:ident) => {
        impl IndexKey for $t {
            fn to_key(&self) -> Vec<u8> {
                use std::$t::MIN;
                (*self ^ MIN).to_be_bytes().to_vec()
            }
            fn from_key(src: &[u8]) -> $t {
                use std::$t::MIN;
                <$t>::from_be_bytes(from_slice!($t, src)) ^ MIN
            }
        }
        #[test]
        fn $t() {
            use std::$t::*;
            let mut list = vec![MAX, MIN, 1, 2, -1, -2, 0];
            list.sort_by_key(|value| {
                assert_eq!(<$t>::from_key(&value.to_key()), *value);
                value.to_key()
            });
            assert_eq!(list, vec![MIN, -2, -1, 0, 1, 2, MAX]);
        }
    };
}

impl_i!(i8);
impl_i!(i16);
impl_i!(i32);
impl_i!(i64);
impl_i!(i128);

macro_rules! impl_f {
    ($f:ty,$fi:ident,$i:ident,$u:ident,$n:expr) => {
        impl IndexKey for $f {
            fn to_key(&self) -> Vec<u8> {
                use std::mem::size_of;
                use std::$i::MIN;
                let value = self.to_bits() as $i;
                (((value >> (size_of::<$i>() * 8 - 1)) | MIN) ^ value)
                    .to_be_bytes()
                    .to_vec()
            }
            fn from_key(src: &[u8]) -> $f {
                use std::mem::size_of;
                use std::$i::MIN;
                let value = $i::from_be_bytes(from_slice!($f, src));
                <$f>::from_bits(((!value >> (size_of::<$i>() * 8 - 1) | MIN) ^ value) as $u)
            }
        }
        #[test]
        fn $fi() {
            use std::$fi::*;
            let mut list: Vec<$f> = vec![
                0.0,
                -0.0,
                1.0,
                -1.0,
                1.1,
                -1.1,
                0.001,
                -0.001,
                INFINITY,
                MAX,
                MIN,
                NEG_INFINITY,
            ];
            list.sort_by_key(|value| {
                assert_eq!(<$f>::from_key(&value.to_key()), *value);
                value.to_key()
            });
            assert_eq!(
                list,
                vec![
                    NEG_INFINITY,
                    MIN,
                    -1.1,
                    -1.0,
                    -0.001,
                    -0.0,
                    0.0,
                    0.001,
                    1.0,
                    1.1,
                    MAX,
                    INFINITY,
                ]
            );
            assert!(NAN.to_key() > INFINITY.to_key())
        }
    };
}

impl_f!(f32, f32, i32, u32, 31);
impl_f!(f64, f64, i64, u64, 63);
