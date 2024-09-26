//! endian create, includes some helpful type: UTF16, VLQ, Ascii etc and endiness/reuseable implmentations

#[cfg(all(feature = "big", feature = "little"))]
compile_error!("have both big or little feature");
#[cfg(all(feature = "len_16", feature = "len_vlq"))]
compile_error!("have both len_16 or len_vlq feature");

pub mod big;
pub mod impl_macro;
pub mod little;

use std::{ffi::CString, ops::Deref};

use crate::{impl_tuple, snd_ref};
use crate::{FromReader, IntoWriter, Receivable, Receiver, Sendable, Sender};

///UTF16 char
#[derive(PartialEq, Eq, Debug)]
pub struct UTF16Char(pub char);

//UTF16 String
#[derive(PartialEq, Eq, Debug)]
pub struct UTF16(pub String);

/// Variable-length quantity
/// https://en.wikipedia.org/wiki/Variable-length_quantity
pub struct VLQ(pub usize);

///Length for String, collections etc.
pub struct Length(pub usize);

impl Deref for UTF16Char {
    type Target = char;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for UTF16 {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Ascii string
#[derive(PartialEq, Eq, Debug)]
pub struct Ascii(pub String);

impl Deref for Ascii {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for VLQ {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Length {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Length {
    #[cfg(feature = "MAX_LEN_100M")]
    const MAX_LEN: usize = 104_857_600;
    #[cfg(feature = "MAX_LEN_500M")]
    const MAX_LEN: usize = 524_288_000;
    #[cfg(feature = "MAX_LEN_2G")]
    const MAX_LEN: usize = 2_147_483_648;
    ///Check if lenght is too big when
    /// The crates has one of the features: MAX_LEN_100M, MAX_LEN_500M, MAX_LEN_2G
    #[inline]
    pub fn check(&self) -> crate::Result<()> {
        #[cfg(any(
            feature = "MAX_LEN_100M",
            feature = "MAX_LEN_500M",
            feature = "MAX_LEN_2G"
        ))]
        if self.0 > Self::MAX_LEN {
            return Err(crate::error::Error::DataTooLarge(Self::MAX_LEN));
        }

        Ok(())
    }
}

impl Sendable for u8 {
    #[inline]
    fn snd_to<S>(&self, writer: &mut S) -> crate::Result<()>
    where
        S: Sender,
    {
        writer.snd_all(&[*self])
    }
}

snd_ref!(&u8);

impl Receivable for u8 {
    #[inline]
    fn rcv_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0];
        reader.rcv_all(&mut buf)?;
        Ok(buf[0])
    }
}

impl Sendable for i8 {
    #[inline]
    fn snd_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        writer.snd_all(&[*self as u8])
    }
}

snd_ref!(&i8);

impl Receivable for i8 {
    #[inline]
    fn rcv_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0];
        reader.rcv_all(&mut buf)?;
        Ok(buf[0] as i8)
    }
}

impl Sendable for bool {
    #[inline]
    fn snd_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        writer.snd_all(if *self { &[1] } else { &[0] })
    }
}

snd_ref!(&bool);

impl Receivable for bool {
    #[inline]
    fn rcv_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0];
        reader.rcv_all(&mut buf)?;
        Ok(buf[0] != 0)
    }
}

impl Sendable for Length {
    fn snd_to<S>(&self, writer: &mut S) -> crate::Result<()>
    where
        S: Sender {
        #[cfg(not(any(feature = "len_vlq", feature = "len_16")))]
        if cfg!(feature = "little") {
            writer.snd_all(&(self.0 as u32).to_le_bytes())?;
        } else if cfg!(feature = "big"){
            writer.snd_all(&(self.0 as u32).to_be_bytes())?;
        } else {
            writer.snd_all(&(self.0 as u32).to_ne_bytes())?;
        }
        
        #[cfg(feature = "len_16")]
        if cfg!(feature = "little") {
            writer.snd_all(&(self.0 as u16).to_le_bytes())?;
        } else {
            writer.snd_all(&(self.0 as u16).to_be_bytes())?;
        }
        #[cfg(feature = "len_vlq")]
        VLQ(self.0).snd_to(writer)?;

        Ok(())
    }
}

impl Receivable for Length {
    fn rcv_from<R>(reader: &mut R) -> crate::Result<Self>
    where
        R: Receiver {
            #[cfg(not(any(feature = "len_vlq", feature = "len_16")))]
            let len = {
                let mut v = [0; 4];
                reader.rcv_all(&mut v)?;
                if cfg!(feature = "little") {
                    u32::from_le_bytes(v) as usize
                } else if cfg!(feature = "big"){
                    u32::from_be_bytes(v) as usize
                } else {
                    u32::from_ne_bytes(v) as usize
                }
            };
            
            #[cfg(feature = "len_16")]
            let len = {
                let mut v = [0; 2];
                reader.rcv_all(&mut v)?;
                if cfg!(feature = "little") {
                    u16::from_le_bytes(v) as usize
                } else {
                    u16::from_be_bytes(v) as usize
                }
            };
            #[cfg(feature = "len_vlq")]
            let len = {
                let vlq: VLQ = VLQ::rcv_from(reader)?;
                vlq.0 as usize
            };
    
            Ok(Length(len))
        }
}

#[cfg(any(feature = "little", feature = "big"))]
impl Sendable for char {
    #[inline]
    fn snd_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        (*self as u32).snd_to(writer)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
snd_ref!(&char);

#[cfg(any(feature = "little", feature = "big"))]
impl Receivable for char {
    #[inline]
    fn rcv_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let v = u32::rcv_from(reader)?;
        char::from_u32(v).ok_or(crate::error::Error::InvalidChar(v))
    }
}

impl Sendable for VLQ {
    #[inline]
    fn snd_to<S>(&self, writer: &mut S) -> crate::Result<()>
    where
        S: Sender,
    {
        let mut b = (self.0 & 127) as u8;
        let mut v = self.0 >> 7;
        let mut vec: Vec<u8> = Vec::new();
        vec.push(b);

        loop {
            if v == 0 {
                break;
            }

            b = (v & 127) as u8;
            v >>= 7;

            vec.push(b | 128);
        }
        vec.reverse();
        writer.snd_all(&vec[..])
    }
}

snd_ref!(&VLQ);

impl Receivable for VLQ {
    #[inline]
    fn rcv_from<R>(reader: &mut R) -> crate::Result<Self>
    where
        R: Receiver,
    {
        let mut buf = [0];
        let mut v = 0;
        loop {
            reader.rcv_all(&mut buf)?;

            v = (v << 7) | (buf[0] & 127) as usize;

            let last = (buf[0] & 128) == 0;

            if last {
                break;
            }
        }
        Ok(VLQ(v))
    }
}

//There will confict if use Borrow<CString>
impl Sendable for CString {
    #[inline]
    fn snd_to<S>(&self, writer: &mut S) -> crate::Result<()>
    where
        S: Sender,
    {
        writer.snd_all(self.as_bytes_with_nul())
    }
}

snd_ref!(&CString);

impl Receivable for CString {
    #[inline]
    fn rcv_from<R>(reader: &mut R) -> crate::Result<Self>
    where
        R: Receiver,
    {
        let mut buf = [0];
        let mut vec = Vec::new();
        loop {
            reader.rcv_all(&mut buf)?;
            vec.push(buf[0]);
            if buf[0] == 0 {
                break;
            }
        }
        Ok(unsafe { CString::from_vec_with_nul_unchecked(vec) })
    }
}

impl<const N: usize> Sendable for [u8; N] {
    #[inline]
    fn snd_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        writer.snd_all(self)?;
        Ok(())
    }
}

impl<const N: usize> Sendable for &[u8; N] {
    #[inline]
    fn snd_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        #[allow(clippy::explicit_auto_deref)]
        writer.snd_all(*self)?;
        Ok(())
    }
}

impl<const N: usize> Receivable for [u8; N] {
    #[inline]
    fn rcv_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let v = reader.rcv_bytes(N)?;
        Self::try_from(v).map_err(|_| crate::error::Error::Other("convert vec to array error"))
    }
}

#[cfg(any(feature = "little", feature = "big"))]
impl Sendable for Vec<u8> {
    #[inline]
    fn snd_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        //length need to be little or big-endian
        Length(self.len()).snd_to(writer)?;
        writer.snd_all(self)?;
        Ok(())
    }
}

#[cfg(any(feature = "little", feature = "big"))]
snd_ref!(&Vec<u8>);

#[cfg(any(feature = "little", feature = "big"))]
impl Receivable for Vec<u8> {
    #[inline]
    fn rcv_from<R>(reader: &mut R) -> crate::Result<Self>
    where
        R: Receiver,
    {
        let len = *Length::rcv_from(reader)?;
        reader.rcv_bytes(len)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
impl Sendable for usize {
    #[inline]
    fn snd_to<S>(&self, writer: &mut S) -> crate::Result<()>
    where
        S: Sender,
    {
        (*self as u64).snd_to(writer)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
snd_ref!(&usize);

#[cfg(any(feature = "little", feature = "big"))]
impl Receivable for usize {
    #[inline]
    fn rcv_from<R>(reader: &mut R) -> crate::Result<Self>
    where
        R: Receiver,
    {
        Ok(u64::rcv_from(reader)? as usize)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
impl Sendable for isize {
    #[inline]
    fn snd_to<S: Sender>(&self, writer: &mut S) -> crate::Result<()> {
        (*self as i64).snd_to(writer)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
snd_ref!(&isize);

#[cfg(any(feature = "little", feature = "big"))]
impl Receivable for isize {
    #[inline]
    fn rcv_from<R>(reader: &mut R) -> crate::Result<Self>
    where
        R: Receiver,
    {
        let v = i64::rcv_from(reader)?;
        Ok(v as isize)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
impl Sendable for std::num::NonZeroU16 {
    #[inline]
    fn snd_to<S: Sender>(&self, writer: &mut S) -> crate::Result<()> {
        self.get().snd_to(writer)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
snd_ref!(&std::num::NonZeroU16);

#[cfg(any(feature = "little", feature = "big"))]
impl Receivable for std::num::NonZeroU16 {
    #[inline]
    fn rcv_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Self::new(u16::rcv_from(reader)?).ok_or(crate::error::Error::Zero)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
impl Sendable for std::num::NonZeroI16 {
    #[inline]
    fn snd_to<S: Sender>(&self, writer: &mut S) -> crate::Result<()> {
        self.get().snd_to(writer)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
snd_ref!(&std::num::NonZeroI16);

#[cfg(any(feature = "little", feature = "big"))]
impl Receivable for std::num::NonZeroI16 {
    #[inline]
    fn rcv_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Self::new(i16::rcv_from(reader)?).ok_or(crate::error::Error::Zero)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
impl Sendable for std::num::NonZeroU32 {
    #[inline]
    fn snd_to<S: Sender>(&self, writer: &mut S) -> crate::Result<()> {
        self.get().snd_to(writer)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
snd_ref!(&std::num::NonZeroU32);

#[cfg(any(feature = "little", feature = "big"))]
impl Receivable for std::num::NonZeroU32 {
    #[inline]
    fn rcv_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Self::new(u32::rcv_from(reader)?).ok_or(crate::error::Error::Zero)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
impl Sendable for std::num::NonZeroI32 {
    #[inline]
    fn snd_to<S: Sender>(&self, writer: &mut S) -> crate::Result<()> {
        self.get().snd_to(writer)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
snd_ref!(&std::num::NonZeroI32);

#[cfg(any(feature = "little", feature = "big"))]
impl Receivable for std::num::NonZeroI32 {
    #[inline]
    fn rcv_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Self::new(i32::rcv_from(reader)?).ok_or(crate::error::Error::Zero)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
impl Sendable for std::num::NonZeroU64 {
    #[inline]
    fn snd_to<S: Sender>(&self, writer: &mut S) -> crate::Result<()> {
        self.get().snd_to(writer)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
snd_ref!(&std::num::NonZeroU64);

#[cfg(any(feature = "little", feature = "big"))]
impl Receivable for std::num::NonZeroU64 {
    #[inline]
    fn rcv_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Self::new(u64::rcv_from(reader)?).ok_or(crate::error::Error::Zero)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
impl Sendable for std::num::NonZeroUsize {
    #[inline]
    fn snd_to<S: Sender>(&self, writer: &mut S) -> crate::Result<()> {
        self.get().snd_to(writer)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
snd_ref!(&std::num::NonZeroUsize);

#[cfg(any(feature = "little", feature = "big"))]
impl Receivable for std::num::NonZeroUsize {
    #[inline]
    fn rcv_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Self::new(u64::rcv_from(reader)? as usize).ok_or(crate::error::Error::Zero)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
impl Sendable for std::num::NonZeroU128 {
    #[inline]
    fn snd_to<S: Sender>(&self, writer: &mut S) -> crate::Result<()> {
        self.get().snd_to(writer)
    }
}

#[cfg(any(feature = "little", feature = "big"))]
snd_ref!(&std::num::NonZeroU128);

#[cfg(any(feature = "little", feature = "big"))]
impl Receivable for std::num::NonZeroU128 {
    #[inline]
    fn rcv_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Self::new(u128::rcv_from(reader)?).ok_or(crate::error::Error::Zero)
    }
}

impl_tuple!(A, B);
impl_tuple!(A, B, C);
impl_tuple!(A, B, C, D);
impl_tuple!(A, B, C, D, E);
impl_tuple!(A, B, C, D, E, F);
impl_tuple!(A, B, C, D, E, F, G);
impl_tuple!(A, B, C, D, E, F, G, H);
impl_tuple!(A, B, C, D, E, F, G, H, I);
impl_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

impl FromReader for String {
    #[inline]
    fn from_reader<R: Receiver>(reader: &mut R, len: usize) -> crate::Result<Self> {
        if len == 0 {
            return Ok("".to_string());
        }

        let b = reader.rcv_bytes(len)?;
        let mut s = std::str::from_utf8(&b)?.to_string();
        while s.ends_with('\0') {
            s.truncate(s.len() - 1); //String is UTF8
        }
        Ok(s)
    }
}

impl IntoWriter for String {
    #[inline]
    fn into_writer<S: Sender>(&self, writer: &mut S, len: usize) -> crate::Result<()> {
        let len_s = self.len();

        let (len_padding, b) = if len > len_s {
            (len - len_s, self.as_bytes())
        } else {
            (0, &self.as_bytes()[..len])
        };

        writer.snd_all(b)?;

        if len_padding > 0 {
            writer.snd_all(&vec![0; len_padding])?;
        }

        Ok(())
    }
}

impl FromReader for Ascii {
    #[inline]
    fn from_reader<R: Receiver>(reader: &mut R, len: usize) -> crate::Result<Self> {
        let s = String::from_reader(reader, len)?;
        Ok(Ascii(s))
    }
}

impl IntoWriter for Ascii {
    #[inline]
    fn into_writer<S: Sender>(&self, writer: &mut S, len: usize) -> crate::Result<()> {
        self.0.into_writer(writer, len)
    }
}

impl FromReader for Vec<u8> {
    #[inline]
    fn from_reader<R: Receiver>(reader: &mut R, len: usize) -> crate::Result<Self> {
        reader.rcv_bytes(len)
    }
}

impl IntoWriter for Vec<u8> {
    #[inline]
    fn into_writer<S: Sender>(&self, writer: &mut S, len: usize) -> crate::Result<()> {
        let len_s = self.len();
        let (l, left) = if len > len_s {
            (len_s, len - len_s)
        } else {
            (len, 0)
        };

        let b = &self[..l];

        writer.snd_all(b)?;

        if left > 0 {
            writer.snd_all(&vec![0; left])?;
        }

        Ok(())
    }
}

#[cfg(any(feature = "little", feature = "big"))]
impl FromReader for UTF16 {
    #[inline]
    fn from_reader<R: Receiver>(reader: &mut R, mut len: usize) -> crate::Result<Self> {
        if len == 0 {
            return Ok(UTF16("".to_string()));
        }

        let mut s = String::with_capacity(len / 2);
        while len > 0 {
            let c = UTF16Char::rcv_from(reader)?;
            len -= c.0.len_utf16() * 2;
            s.push(*c);
        }

        while s.ends_with('\0') {
            s.truncate(s.len() - 1); //String is UTF8
        }

        Ok(UTF16(s))
    }
}

#[cfg(any(feature = "little", feature = "big"))]
impl IntoWriter for UTF16 {
    #[inline]
    fn into_writer<S: Sender>(&self, writer: &mut S, mut len: usize) -> crate::Result<()> {
        if len > 0 {
            for c in self.chars() {
                UTF16Char(c).snd_to(writer)?;
                len -= c.len_utf16() * 2;
                if len == 0 {
                    break;
                }
            }
        }

        if len > 0 {
            writer.snd_all(&vec![0; len])?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        endian::{little::LE, UTF16Char, UTF16, VLQ},
        error::Error,
        Rcv, Snd,
    };
    use std::ffi::CString;

    #[cfg(any(feature = "big", feature = "little"))]
    #[test]
    fn test_default() -> crate::Result<()> {
        let mut vec = Vec::new();
        vec.snd(-8_i8)?;
        vec.snd(&22_u16)?;
        vec.snd(0xFFABCDEF as u32)?;
        vec.snd("Test")?;
        vec.snd(UTF16("utf16".to_string()))?;
        vec.snd(-32 as i32)?;

        let mut buf = &vec[..];

        let v: i8 = buf.rcv()?;
        assert_eq!(v, -8);

        let v: u16 = buf.rcv()?;
        assert_eq!(v, 22);

        let v: u32 = buf.rcv()?;
        assert_eq!(v, 0xFFABCDEF);

        let v: String = buf.rcv()?;
        assert_eq!(v, "Test");

        let v: UTF16 = buf.rcv()?;
        assert_eq!(*v, "utf16");

        let v: i32 = buf.rcv()?;
        assert_eq!(v, -32);

        Ok(())
    }

    #[test]
    fn test_u8() -> crate::Result<()> {
        let mut vec: Vec<u8> = Vec::new();
        vec.snd(8_u8)?;
        vec.snd(16_u8)?;
        assert!(vec.len() == 2);
        assert!(vec[0] == 8);
        assert!(vec[1] == 16);

        (&mut vec).snd(12_u8)?;

        let mut buf = &vec[..];

        let v: u8 = buf.rcv()?;
        assert_eq!(v, 8);

        let v: u8 = buf.rcv()?;
        assert_eq!(v, 16);

        let v: u8 = buf.rcv()?;
        assert_eq!(v, 12);

        let rst = buf.rcv::<u8>();
        assert!(rst.is_err());

        Ok(())
    }
    #[test]
    fn test_u16() -> crate::Result<()> {
        let mut vec: Vec<u8> = Vec::new();
        vec.snd(LE(22_u16))?;
        vec.snd(LE(0xF9FF_u16))?;
        assert_eq!(vec.len(), 4);
        assert_eq!(vec[0], 22);
        assert_eq!(vec[1], 0);
        assert_eq!(vec[2], 0xFF);
        assert_eq!(vec[3], 0xF9);

        let mut buf = &vec[..];

        let v: LE<u16> = buf.rcv()?;
        assert_eq!(*v, 22);

        let v: LE<u16> = buf.rcv()?;
        assert_eq!(*v, 0xF9FF);

        Ok(())
    }

    #[test]
    fn test_read_u32() -> crate::Result<()> {
        let mut b = "WEwe".as_bytes();
        let v: LE<u32> = b.rcv()?;
        assert_eq!(
            *v,
            'W' as u32 | ('E' as u32) << 8 | ('w' as u32) << 16 | ('e' as u32) << 24
        );
        let rst = b.rcv::<LE<u16>>();
        match rst {
            Ok(_) => assert!(false),
            Err(Error::Io(e)) => {
                println!("Err {:?}", e);
                assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            Err(_) => {
                panic!("Wrong error kind");
            }
        }

        Ok(())
    }

    #[test]
    fn test_read_u64() -> crate::Result<()> {
        let mut b = "WEwe1234".as_bytes();
        println!("before {:p}, {}", b, b[0]);
        let v1 = b.rcv::<LE<u32>>()?;
        println!("after u32 {:p}, {}", b, b[0]);
        let v2 = b.rcv::<LE<u32>>()?;
        println!("after another u32 {:p}", b);
        assert_eq!(
            *v1 as u64 | (*v2 as u64) << 32,
            'W' as u64
                | ('E' as u64) << 8
                | ('w' as u64) << 16
                | ('e' as u64) << 24
                | ('1' as u64) << 32
                | ('2' as u64) << 40
                | ('3' as u64) << 48
                | ('4' as u64) << 56
        );
        Ok(())
    }

    #[test]
    fn test_string() -> crate::Result<()> {
        let mut vec: Vec<u8> = Vec::new();
        vec.snd(LE("2欢迎2𝌆𠮷\0".to_string()))?;
        let mut buf = &vec[..];
        let s: LE<String> = buf.rcv()?;
        assert_eq!("2欢迎2𝌆𠮷\0", *s);
        Ok(())
    }

    #[test]
    fn test_utf16() -> crate::Result<()> {
        let u = LE(UTF16Char('W'));
        assert_eq!('W', **u);
        let mut vec: Vec<u8> = Vec::new();
        vec.snd(LE(UTF16Char('W')))?;
        assert_eq!('W' as u8, vec[0]);
        assert_eq!(0, vec[1]);

        let mut buf: &[u8] = vec.as_ref();
        assert_eq!('W', **(buf.rcv::<LE<UTF16Char>>()?));
        Ok(())
    }
    #[test]
    fn test_utf16_str() -> crate::Result<()> {
        let mut vec: Vec<u8> = Vec::new();

        vec.snd(LE(UTF16(("2欢迎2𝌆𠮷\0").to_string())))?;

        let s = String::from("Test String");
        vec.snd(LE(UTF16(s)))?;

        let mut buf: &[u8] = &vec;

        let s: LE<UTF16> = buf.rcv()?;
        assert_eq!("2欢迎2𝌆𠮷\0", **s);

        let s: LE<UTF16> = buf.rcv()?;
        assert_eq!("Test String", **s);

        Ok(())
    }

    #[test]
    fn test_vlq() -> crate::Result<()> {
        let mut vec: Vec<u8> = Vec::new();
        vec.snd(VLQ(0))?;
        vec.snd(VLQ(127))?;
        assert_eq!(2, vec.len());

        let mut buf: &[u8] = &vec;
        let v: VLQ = buf.rcv()?;
        assert_eq!(0, *v);
        let v: VLQ = buf.rcv()?;
        assert_eq!(127, *v);

        let mut vec: Vec<u8> = Vec::new();
        vec.snd(VLQ(128))?;
        assert_eq!(2, vec.len());

        let mut buf: &[u8] = &vec;
        let v: VLQ = buf.rcv()?;
        assert_eq!(128, *v);

        let mut vec: Vec<u8> = Vec::new();
        vec.snd(VLQ(16384))?;
        assert_eq!(3, vec.len());

        let mut buf: &[u8] = &vec;
        let v: VLQ = buf.rcv()?;
        assert_eq!(16384, *v);

        let mut vec: Vec<u8> = Vec::new();
        vec.snd(VLQ(0x0FFFFFFF))?;
        assert_eq!(4, vec.len());

        let mut buf: &[u8] = &vec;
        let v: VLQ = buf.rcv()?;
        assert_eq!(0x0FFFFFFF, *v);

        Ok(())
    }

    #[test]
    #[cfg(any(feature = "big", feature = "little"))]
    fn test_utf16_from() -> crate::Result<()> {
        use crate::endian::Length;
        use crate::{FromReader, IntoWriter};

        let mut vec: Vec<u8> = Vec::new();

        vec.snd(UTF16(("2欢迎2𝌆𠮷").to_string()))?;

        #[cfg(not(any(feature = "len_vlq", feature = "len_16")))]
        assert_eq!(16 + 4, vec.len());
        #[cfg(feature = "len_16")]
        assert_eq!(16 + 2, vec.len());
        #[cfg(feature = "len_vlq")]
        assert_eq!(16 + 1, vec.len());

        let u = UTF16(String::from("Test"));
        vec.snd(&u)?;

        vec.snd(8 as u32)?;
        u.into_writer(&mut vec, 8)?;

        let mut buf: &[u8] = &vec;

        let len: Length = buf.rcv()?;

        let s = UTF16::from_reader(&mut buf, *len)?;

        assert_eq!("2欢迎2𝌆𠮷", *s);

        let s: UTF16 = buf.rcv()?;
        assert_eq!("Test", *s);

        Ok(())
    }

    #[test]
    fn test_cstring() -> crate::Result<()> {
        let mut vec: Vec<u8> = Vec::new();
        let cs = CString::new("abc12 3").unwrap();
        vec.snd(&cs)?;

        let mut buf: &[u8] = &vec;
        let actual: CString = buf.rcv()?;
        assert_eq!(cs, actual);

        Ok(())
    }
    #[test]
    #[cfg(any(feature = "big", feature = "little"))]
    fn test_tuple() -> crate::Result<()> {
        let t = (1_u16, 2_u32);

        let mut vec: Vec<u8> = Vec::new();
        vec.snd(t)?;

        let mut buf: &[u8] = &vec;

        let actual: (u16, u32) = buf.rcv()?;

        assert_eq!(t, actual);

        Ok(())
    }

    #[test]
    #[cfg(any(feature = "big", feature = "little"))]
    fn test_range() -> crate::Result<()> {
        use std::ops::Range;

        let t = 1..30;

        let mut vec: Vec<u8> = Vec::new();
        vec.snd(&t)?;

        let mut buf: &[u8] = &vec;

        let actual: Range<i32> = buf.rcv()?;

        assert_eq!(t, actual);

        Ok(())
    }

    #[test]
    fn test_u8_array() -> crate::Result<()> {
        let b = [22_u8; 32];

        let mut vec = Vec::new();
        vec.snd(&b)?;

        let mut buf: &[u8] = &vec;
        let actual: [u8; 32] = buf.rcv()?;

        assert_eq!(&b, &actual);

        Ok(())
    }

    #[test]
    #[cfg(any(feature = "big", feature = "little"))]
    fn test_vec_u8() -> crate::Result<()> {
        let b = vec![1_u8, 2, 3];

        let mut vec = Vec::new();
        vec.snd(&b)?;

        let mut buf: &[u8] = &vec;
        let actual: Vec<u8> = buf.rcv()?;

        assert_eq!(&b, &actual);

        Ok(())
    }
}
