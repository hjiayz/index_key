
//! lexicographic sort order encoding.

use failure::Error;
use std::convert::TryInto;

pub trait IndexKey: Sized {
    fn to_key(&self) -> Vec<u8>;
    fn try_from_key(key: &[u8]) -> Result<Self, Error>;
}

impl IndexKey for String {
    fn to_key(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
    fn try_from_key(src: &[u8]) -> Result<String, Error> {
        Ok(std::str::from_utf8(src)?.to_string())
    }
}

impl IndexKey for Vec<u8> {
    fn to_key(&self) -> Vec<u8> {
        self.to_vec()
    }
    fn try_from_key(src: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(src.to_vec())
    }
}

macro_rules! impl_u {
    ($t:ident) => {
        impl IndexKey for $t {
            fn to_key(&self) -> Vec<u8> {
                self.to_be_bytes().to_vec()
            }
            fn try_from_key(src: &[u8]) -> Result<$t, Error> {
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
        impl IndexKey for $t {
            fn to_key(&self) -> Vec<u8> {
                use std::$t::MIN;
                (*self ^ MIN).to_be_bytes().to_vec()
            }
            fn try_from_key(src: &[u8]) -> Result<$t, Error> {
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
    ($f:ty,$fi:ident,$i:ident,$u:ident) => {
        impl IndexKey for $f {
            fn to_key(&self) -> Vec<u8> {
                use std::$i::MIN;
                let value = self.to_bits() as $i;
                if value < 0 { !value } else { value ^ MIN }
                    .to_be_bytes()
                    .to_vec()
            }
            fn try_from_key(src: &[u8]) -> Result<$f, Error> {
                use std::$i::MIN;
                let value = $u::from_be_bytes(src.try_into()?);
                let result = if (value as $i) < 0 {
                    <$f>::from_bits(value ^ (MIN as $u))
                } else {
                    <$f>::from_bits(!value)
                };
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

impl_f!(f32, f32, i32, u32);
impl_f!(f64, f64, i64, u64);
