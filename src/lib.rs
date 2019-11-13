//! lexicographic sort order encoding.

use anyhow::Error;
use std::convert::TryInto;

pub trait IndexKey<'a>: Sized {
    fn to_key(&self) -> Vec<u8>;
    fn try_from_key(key: &'a [u8]) -> Result<Self, Error>;
}

impl<'a> IndexKey<'a> for String {
    fn to_key(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
    fn try_from_key(src: &'a [u8]) -> Result<String, Error> {
        Ok(std::str::from_utf8(src)?.to_string())
    }
}

#[test]
fn test_string() {
    let s: String = "123".into();
    assert_eq!(String::try_from_key(&s.to_key()).unwrap(), s)
}

impl<'a> IndexKey<'a> for &'a str {
    fn to_key(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
    fn try_from_key(src: &'a [u8]) -> Result<&'a str, Error> {
        Ok(std::str::from_utf8(src)?)
    }
}

#[test]
fn test_str() {
    let s = "123";
    assert_eq!(<&str>::try_from_key(&s.to_key()).unwrap(), s)
}

impl<'a> IndexKey<'a> for Vec<u8> {
    fn to_key(&self) -> Vec<u8> {
        self.to_vec()
    }
    fn try_from_key(src: &'a [u8]) -> Result<Vec<u8>, Error> {
        Ok(src.to_vec())
    }
}

macro_rules! impl_u {
    ($t:ident) => {
        impl<'a> IndexKey<'a> for $t {
            fn to_key(&self) -> Vec<u8> {
                self.to_be_bytes().to_vec()
            }
            fn try_from_key(src: &'a [u8]) -> Result<$t, Error> {
                Ok(<$t>::from_be_bytes(src.try_into()?))
            }
        }
        #[test]
        fn $t() {
            use std::$t::*;
            let mut list = vec![MAX, 1, 2, 0];
            list.sort_by_key(|value| {
                assert_eq!(<$t>::try_from_key(&value.to_key()).unwrap(), *value);
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
        impl<'a> IndexKey<'a> for $t {
            fn to_key(&self) -> Vec<u8> {
                use std::$t::MIN;
                (*self ^ MIN).to_be_bytes().to_vec()
            }
            fn try_from_key(src: &'a [u8]) -> Result<$t, Error> {
                use std::$t::MIN;
                Ok(<$t>::from_be_bytes(src.try_into()?) ^ MIN)
            }
        }
        #[test]
        fn $t() {
            use std::$t::*;
            let mut list = vec![MAX, MIN, 1, 2, -1, -2, 0];
            list.sort_by_key(|value| {
                assert_eq!(<$t>::try_from_key(&value.to_key()).unwrap(), *value);
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
        impl<'a> IndexKey<'a> for $f {
            fn to_key(&self) -> Vec<u8> {
                use std::mem::size_of;
                use std::$i::MIN;
                let value = self.to_bits() as $i;
                (((value >> (size_of::<$i>() * 8 - 1)) | MIN) ^ value)
                    .to_be_bytes()
                    .to_vec()
            }
            fn try_from_key(src: &'a [u8]) -> Result<$f, Error> {
                use std::mem::size_of;
                use std::$i::MIN;
                let value = $i::from_be_bytes(src.try_into()?);
                let result =
                    <$f>::from_bits(((!value >> (size_of::<$i>() * 8 - 1) | MIN) ^ value) as $u);
                Ok(result)
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
                assert_eq!(<$f>::try_from_key(&value.to_key()).unwrap(), *value);
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
