//! Little-endian type (LE) and implmentations
use std::{
    borrow::Cow,
    ops::Range,
    path::PathBuf,
    time::Duration,
};

use crate::{Receivable, Receiver, Sendable, Sender};

use super::{Length, UTF16Char, LE, UTF16};

pub trait SendableLE {
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()>;
}

pub trait ReceivableLE {
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized;
}



impl<T: SendableLE> Sendable for LE<T> {
    #[inline]
    fn snd_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        self.0.send_to(writer)
    }
}

impl<T: SendableLE> Sendable for &LE<T> {
    #[inline]
    fn snd_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        self.0.send_to(writer)
    }
}

#[cfg(feature = "little")]
impl<T: SendableLE> Sendable for T {
    #[inline]
    fn snd_to<W>(&self, writer: &mut W) -> crate::Result<()>
    where
        W: Sender,
    {
        self.send_to(writer)
    }
}

impl<'a, T: SendableLE> SendableLE for &'a T {
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        (*self).send_to(writer)
    }
}

impl SendableLE for u16 {
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        writer.snd_all(&self.to_le_bytes())
    }
}

impl SendableLE for i16 {
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        writer.snd_all(&self.to_le_bytes())
    }
}

impl SendableLE for u32 {
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        writer.snd_all(&self.to_le_bytes())
    }
}

impl SendableLE for i32 {
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        writer.snd_all(&self.to_le_bytes())
    }
}

impl SendableLE for f32 {
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        writer.snd_all(&self.to_le_bytes())
    }
}

impl SendableLE for u64 {
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        writer.snd_all(&self.to_le_bytes())
    }
}

impl SendableLE for i64 {
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        writer.snd_all(&self.to_le_bytes())
    }
}

impl SendableLE for f64 {
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        writer.snd_all(&self.to_le_bytes())
    }
}

impl SendableLE for u128 {
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        writer.snd_all(&self.to_le_bytes())
    }
}

impl SendableLE for i128 {
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        writer.snd_all(&self.to_le_bytes())
    }
}

//This is not possible for now
// impl<T: SendableLE, U: IntoIterator<Item = T>> SendableLE for U{
//     fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
//         todo!()
//     }
// }


impl SendableLE for UTF16Char {
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        let mut v = self.0 as u32;
        if v <= 0xFFFF {
            (v as u16).send_to(writer)
        } else {
            //4 bytes long
            v -= 0x10000;
            let high = (v >> 10) as u16 | 0xD800;
            high.send_to(writer)?;
            let low = (v as u16 & 0x3FF) | 0xDC00;
            low.send_to(writer)
        }
    }
}

impl SendableLE for UTF16 {
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        let len: usize = self.0.chars().map(|c| c.len_utf16() * 2).sum();
        Length(len).snd_to(writer)?;
        //don't reuse into_writer here (which is conditional compiled)
        for c in self.chars() {
            UTF16Char(c).send_to(writer)?;
        }
        Ok(())
    }
}

impl<T> Receivable for LE<T>
where
    T: ReceivableLE,
{
    #[inline]
    fn rcv_from<R>(reader: &mut R) -> crate::Result<Self>
    where
        R: Receiver,
    {
        Ok(LE(T::receive_from(reader)?))
    }
}

#[cfg(feature = "little")]
impl<T: ReceivableLE> Receivable for T {
    #[inline]
    fn rcv_from<R>(reader: &mut R) -> crate::Result<Self>
    where
        R: Receiver,
    {
        T::receive_from(reader)
    }
}

impl ReceivableLE for u16 {
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0; 2];
        reader.rcv_all(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }
}

impl ReceivableLE for i16 {
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0; 2];
        reader.rcv_all(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }
}

impl ReceivableLE for u32 {
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0; 4];
        reader.rcv_all(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
}

impl ReceivableLE for i32 {
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0; 4];
        reader.rcv_all(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }
}

impl ReceivableLE for f32 {
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0; 4];
        reader.rcv_all(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }
}

impl ReceivableLE for u64 {
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0; 8];
        reader.rcv_all(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }
}

impl ReceivableLE for i64 {
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0; 8];
        reader.rcv_all(&mut buf)?;
        Ok(i64::from_le_bytes(buf))
    }
}

impl ReceivableLE for f64 {
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0; 8];
        reader.rcv_all(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }
}

impl ReceivableLE for u128 {
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0; 16];
        reader.rcv_all(&mut buf)?;
        Ok(u128::from_le_bytes(buf))
    }
}

impl ReceivableLE for i128 {
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0; 16];
        reader.rcv_all(&mut buf)?;
        Ok(i128::from_le_bytes(buf))
    }
}

impl ReceivableLE for UTF16Char {
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut high = u16::receive_from(reader)? as u32;
        if (0xD800..=0xDBFF).contains(&high) {
            //It's 4 byte long
            let low = u16::receive_from(reader)? as u32;
            // high = (((high - 0xD800) << 10) | (low - 0xDC00)) + 0x10000;
            high = (((high & 0x3FF) << 10) | (low & 0x3FF)) + 0x10000;
        };

        if let Some(v) = char::from_u32(high) {
            Ok(UTF16Char(v))
        } else {
            Err(crate::error::Error::InvalidChar(high))
        }
    }
}

impl ReceivableLE for UTF16 {
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut len = *Length::rcv_from(reader)?;
        let mut s = String::with_capacity(len / 2);
        while len > 0 {
            let c = UTF16Char::receive_from(reader)?;
            len -= c.0.len_utf16() * 2;
            //don't handle null value
            s.push(*c);
        }
        Ok(UTF16(s))
    }
}


impl<Idx> SendableLE for Range<Idx>
where
    Idx: SendableLE,
{
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        self.start.send_to(writer)?;
        self.end.send_to(writer)
    }
}

impl<Idx> ReceivableLE for Range<Idx>
where
    Idx: ReceivableLE,
{
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let start = Idx::receive_from(reader)?;
        let end = Idx::receive_from(reader)?;
        Ok(Range { start, end })
    }
}

impl SendableLE for Duration {
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        self.as_secs().send_to(writer)?;
        self.subsec_nanos().send_to(writer)
    }
}

impl ReceivableLE for Duration {
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let secs = u64::receive_from(reader)?;
        let nanos = u32::receive_from(reader)?;
        Ok(Duration::new(secs, nanos))
    }
}

impl<T> SendableLE for Box<T>
where
    T: SendableLE,
{
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        (**self).send_to(writer)
    }
}

impl<T> ReceivableLE for Box<T>
where
    T: ReceivableLE,
{
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let t = T::receive_from(reader)?;
        Ok(Box::new(t))
    }
}

impl<'a, T> SendableLE for Cow<'a, T>
where
    T: SendableLE + Clone,
{
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        match self {
            Cow::Borrowed(v) => v.send_to(writer),
            Cow::Owned(v) => v.send_to(writer),
        }
    }
}

impl<'a, T> ReceivableLE for Cow<'a, T>
where
    T: ReceivableLE + Clone,
{
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let t = T::receive_from(reader)?;
        Ok(Cow::Owned(t))
    }
}

impl SendableLE for PathBuf {
    #[inline]
    fn send_to<W: Sender>(&self, writer: &mut W) -> crate::Result<()> {
        match self.to_string_lossy() {
            Cow::Borrowed(v) => v.snd_to(writer),
            Cow::Owned(v) => v.snd_to(writer),
        }
    }
}

impl ReceivableLE for PathBuf {
    #[inline]
    fn receive_from<R: Receiver>(reader: &mut R) -> crate::Result<Self> {
        let s = String::rcv_from(reader)?;
        Ok(PathBuf::from(s))
    }
}


#[cfg(test)]
mod tests {
    use crate::{endian::little::LE, Snd};

    #[test]
    fn test_le_num() -> crate::Result<()> {
        let mut vec: Vec<u8> = Vec::new();
        let v = LE(&8_u16);
        vec.snd(&v)?;
        vec.snd(v)?;
        vec.snd(LE(32))?; //i32
        vec.snd(LE(32_u64))?;
        assert!(vec.len() == 16);
        assert!(vec[0] == 8);
        // assert!(vec[1] == 0);
        // assert!(vec[2] == 8);
        // assert!(vec[3] == 0);
        Ok(())
    }
}
