//! lexicographic sort order encoding.

use std::io::Cursor;
use std::io::Error;
use std::io::Read;
use std::io::Write;

pub trait IndexKey: Sized {
    fn to_key<W: Write>(self, result: &mut W) -> Result<&mut W, Error>;
    fn from_key<R: Read>(key: &mut R) -> Result<Self, Error>;
}

impl IndexKey for String {
    #[inline]
    fn to_key<W: Write>(self, result: &mut W) -> Result<&mut W, Error> {
        self.into_bytes().to_key(result)
    }
    #[inline]
    fn from_key<R: Read>(key: &mut R) -> Result<Self, Error> {
        Ok(String::from_utf8_lossy(&Vec::<u8>::from_key(key)?).to_string())
    }
}

#[test]
fn test_string() {
    let s: String = "123".into();
    assert_eq!(from_key::<String>(to_key(s.clone())).unwrap(), s);

    for c in '\0' as u32..('😃' as u32) {
        let a = std::char::from_u32(c);
        let b = std::char::from_u32(c + 1);
        if a.is_none() {
            continue;
        }
        if b.is_none() {
            continue;
        }
        assert!(to_key(a.unwrap().to_string()) < to_key(b.unwrap().to_string()));
    }
}

impl IndexKey for Vec<u8> {
    fn to_key<W: Write>(self, result: &mut W) -> Result<&mut W, Error> {
        escape_encode(&mut Cursor::new(self), result)
    }
    fn from_key<R: Read>(key: &mut R) -> Result<Self, Error> {
        let mut result = vec![];
        escape_decode(key, &mut result)?;
        Ok(result)
    }
}

#[cfg(test)]
struct VecRange(Vec<u8>, usize);

#[cfg(test)]
impl Iterator for VecRange {
    type Item = (Vec<u8>, Vec<u8>);
    fn next(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        let mut state = true;
        let max_len = self.1;
        let old = self.0.clone();
        loop {
            if self.0.len() == 0 {
                if state {
                    self.0.push(0);
                    break;
                } else {
                    return None;
                }
            }
            if state && (self.0.len() < max_len) {
                if *self.0.last().unwrap() == 255 {
                    self.0.push(0);
                    break;
                }
                *self.0.last_mut().unwrap() += 1;
                break;
            }
            if *self.0.last().unwrap() == 255 {
                self.0.pop();
                state = false;
                continue;
            }
            *self.0.last_mut().unwrap() += 1;
            break;
        }
        Some((old, self.0.clone()))
    }
}

#[test]
fn test_vec_u8() {
    let v = vec![1u8, 2, 3, 4];
    assert_eq!(from_key::<Vec<u8>>(to_key(v.clone())).unwrap(), v);
    let it = VecRange(vec![], 6);
    for (old_v, new_v) in it {
        assert!(to_key(new_v.clone()) > to_key(old_v.clone()));
    }
}

macro_rules! impl_u {
    ($t:ident) => {
        impl IndexKey for $t {
            fn to_key<W: Write>(self, result: &mut W) -> Result<&mut W, Error> {
                result.write_all(&self.to_be_bytes())?;
                Ok(result)
            }
            fn from_key<R: Read>(key: &mut R) -> Result<$t, Error> {
                let mut slice = [0u8; std::mem::size_of::<$t>()];
                key.read_exact(&mut slice)?;
                Ok(<$t>::from_be_bytes(slice))
            }
        }
        #[test]
        fn $t() {
            use std::$t::*;
            let mut list = vec![MAX, 1, 2, 0];
            list.sort_by_key(|value| {
                assert_eq!(from_key::<$t>(to_key(*value)).unwrap(), *value);
                to_key(*value)
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
            fn to_key<W: Write>(self, result: &mut W) -> Result<&mut W, Error> {
                use std::$t::MIN;
                let slice = (self ^ MIN).to_be_bytes();
                result.write_all(&slice)?;
                Ok(result)
            }
            fn from_key<R: Read>(key: &mut R) -> Result<$t, Error> {
                use std::$t::MIN;
                let mut slice = [0u8; std::mem::size_of::<$t>()];
                key.read_exact(&mut slice)?;
                Ok(<$t>::from_be_bytes(slice) ^ MIN)
            }
        }
        #[test]
        fn $t() {
            use std::$t::*;
            let mut list = vec![MAX, MIN, 1, 2, -1, -2, 0];
            list.sort_by_key(|value| {
                assert_eq!(from_key::<$t>(to_key(*value)).unwrap(), *value);
                to_key(*value)
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
            fn to_key<W: Write>(self, result: &mut W) -> Result<&mut W, Error> {
                use std::mem::size_of;
                use std::$i::MIN;
                let value = self.to_bits() as $i;
                let slice = (((value >> (size_of::<$i>() * 8 - 1)) | MIN) ^ value).to_be_bytes();
                result.write_all(&slice)?;
                Ok(result)
            }
            fn from_key<R: Read>(key: &mut R) -> Result<$f, Error> {
                use std::mem::size_of;
                use std::$i::MIN;
                let mut slice = [0u8; std::mem::size_of::<$f>()];
                key.read_exact(&mut slice)?;
                let value = $i::from_be_bytes(slice);
                Ok(<$f>::from_bits(
                    ((!value >> (size_of::<$i>() * 8 - 1) | MIN) ^ value) as $u,
                ))
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
                assert_eq!(from_key::<$f>(to_key(*value)).unwrap(), *value);
                to_key(*value)
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
            assert!(to_key(NAN) > to_key(INFINITY))
        }
    };
}

impl_f!(f32, f32, i32, u32, 31);
impl_f!(f64, f64, i64, u64, 63);

impl IndexKey for bool {
    fn to_key<W: Write>(self, result: &mut W) -> Result<&mut W, Error> {
        if self {
            result.write_all(&[1])
        } else {
            result.write_all(&[0])
        }
        .map(|_| result)
    }
    fn from_key<R: Read>(key: &mut R) -> Result<bool, Error> {
        let mut slice = [0];
        key.read_exact(&mut slice)?;
        Ok(slice[0] != 0)
    }
}

#[test]
fn test_bool() {
    assert_eq!(from_key::<bool>(to_key(true)).unwrap(), true);
    assert_eq!(to_key(true), vec![1]);
}

macro_rules! impl_tuple {
    ( $( $v:ident ),+ ) => {
        impl< $( $v ),+ > IndexKey for ( $($v),+ )
        where
            $( $v : IndexKey ,)+
        {
            #[inline]
            #[allow(non_snake_case)]
            fn to_key<W: Write>(self, result: &mut W) -> Result<&mut W, Error> {
                let ($( $v,)+) = self;
                $(
                    $v.to_key(result)?;
                )+
                Ok(result)
            }
            #[inline]
            fn from_key<R: Read>(key: &mut R) -> Result<( $($v),+ ), Error> {
                Ok(( $(
                    $v::from_key(key)?,
                )+ ))
            }
        }
    }
}

impl_tuple!(T1, T2);
impl_tuple!(T1, T2, T3);
impl_tuple!(T1, T2, T3, T4);
impl_tuple!(T1, T2, T3, T4, T5);
impl_tuple!(T1, T2, T3, T4, T5, T6);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);

#[test]
fn test_tuple() {
    let list1: Vec<u8> = vec![1, 2, 1, 2, 0];
    let list2: Vec<u8> = vec![1, 2, 1, 2, 2];
    let string: String = "123".to_owned();
    let key = to_key((
        list1.clone(),
        list2.clone(),
        string.clone(),
        true,
        1.0f32,
        1i64,
    ));

    let (l1, l2, s, b, f, i): (Vec<u8>, Vec<u8>, String, bool, f32, i64) = from_key(key).unwrap();
    assert_eq!(list1, l1);
    assert_eq!(list2, l2);
    assert_eq!(string, s);
    assert_eq!(true, b);
    assert_eq!(1.0f32, f);
    assert_eq!(1i64, i);
}

#[test]
fn test_tuple2() {
    let it = VecRange(vec![], 2);
    for (a1, _) in it {
        let it2 = VecRange(vec![], 2);
        for (a2, _) in it2 {
            for b1 in [0u16, 1, u16::max_value()].into_iter() {
                for b2 in [0u16, 1, u16::max_value()].into_iter() {
                    let (b1, b2) = (*b1, *b2);
                    if a1 < a2 {
                        assert!(to_key((a1.clone(), b1)) < to_key((a2.clone(), b2)));
                        continue;
                    }
                    if a1 > a2 {
                        assert!(to_key((a1.clone(), b1)) > to_key((a2.clone(), b2)));
                        continue;
                    }
                    assert!(b1.cmp(&b2) == to_key((a1.clone(), b1)).cmp(&to_key((a2.clone(), b2))))
                }
            }
        }
    }
}

pub fn escape_encode<'a, R: Read, W: Write>(
    src: &mut R,
    result: &'a mut W,
) -> Result<&'a mut W, Error> {
    let mut buf = [0u8];
    while src.read_exact(&mut buf).is_ok() {
        let item = buf[0];
        match item {
            0 | 1 => result.write_all(&[1])?,
            _ => (),
        };
        result.write_all(&buf)?;
    }
    result.write_all(&[0])?;
    Ok(result)
}

pub fn escape_decode<'a, R: Read, W: Write>(
    src: &mut R,
    result: &'a mut W,
) -> Result<&'a mut W, Error> {
    let mut state = true;
    let mut buf = [0u8];
    while src.read_exact(&mut buf).is_ok() {
        let item = buf[0];
        if state {
            match item {
                0 => return Ok(result),
                1 => {
                    state = false;
                    continue;
                }
                _ => (),
            }
        }
        result.write_all(&buf)?;
        state = true;
    }
    Ok(result)
}

pub fn to_key<I: IndexKey>(i: I) -> Vec<u8> {
    let mut result = vec![];
    let _ = i.to_key(&mut result);
    result
}

pub fn from_key<I: IndexKey>(src: Vec<u8>) -> Result<I, Error> {
    let mut cur = Cursor::new(src);
    I::from_key(&mut cur)
}
