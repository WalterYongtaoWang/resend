//! Sender, Receiver, Snd, Rcv traits.
pub mod endian;
pub mod error;

pub type Result<T> = std::result::Result<T, crate::error::Error>;

///Abstract layer for Write since it's not avaialbe in no_std
pub trait Sender {
    fn snd_all(&mut self, buf: &[u8]) -> Result<()>;
}

///Abstract layer for Read since it's not avaialbe in no_std
pub trait Receiver {
    fn rcv_all(&mut self, buf: &mut [u8]) -> Result<()>;
}

///Impl Sendable if the data need to be serialized.
pub trait Sendable {
    fn snd_to<S>(&self, writer: &mut S) -> Result<()>
    where
        S: Sender;
}

///Impl Receivable if the data need to be deserialized.
pub trait Receivable: Sized {
    fn rcv_from<R>(reader: &mut R) -> Result<Self>
    where
        R: Receiver;
}

///Send Trait for Sender
pub trait Snd {
    fn snd<T>(&mut self, v: T) -> Result<()>
    where
        T: Sendable;
}

///Receive Trait for Receiver
pub trait Rcv {
    fn rcv<T>(&mut self) -> Result<T>
    where
        T: Receivable;
    fn rcv_bytes(&mut self, len: usize) -> Result<Vec<u8>>;
}

///Receive Trait for the #[len] attribute
/// The length can be from another field value or const for the #[len] attribute
/// For example: #[len(field_name)], #[len(8)]
pub trait FromReader: Sized {
    fn from_reader<R: Receiver>(reader: &mut R, len: usize) -> Result<Self>;
}

///Send Trait for the #[len] attribute
/// The length can be from another field value or const for the #[len] attribute
/// For example: #[len(field_name)], #[len(8)]
pub trait IntoWriter {
    #[allow(clippy::wrong_self_convention)]
    fn into_writer<S: Sender>(&self, writer: &mut S, len: usize) -> Result<()>;
}

impl<S: Sender> Snd for S {
    #[inline]
    fn snd<T: Sendable>(&mut self, v: T) -> Result<()> {
        v.snd_to(self)
    }
}

impl<R> Rcv for R
where
    R: Receiver,
{
    #[inline]
    fn rcv<T>(&mut self) -> Result<T>
    where
        T: Receivable,
    {
        T::rcv_from(self)
    }

    #[inline]
    #[allow(clippy::uninit_vec)]
    fn rcv_bytes(&mut self, len: usize) -> Result<Vec<u8>> {
        // let mut vec = vec![0; len];
        let mut vec = Vec::with_capacity(len);
        unsafe {
            vec.set_len(len);
        }

        self.rcv_all(&mut vec)?;
        Ok(vec)
    }
}

//impl Sender for all Write implementors
impl<W: std::io::Write> Sender for W {
    #[inline]
    fn snd_all(&mut self, buf: &[u8]) -> Result<()> {
        self.write_all(buf)?;
        Ok(())
    }
}

//impl Receiver for all Read implmentors
impl<R: std::io::Read> Receiver for R {
    #[inline]
    fn rcv_all(&mut self, buf: &mut [u8]) -> Result<()> {
        self.read_exact(buf)?;
        Ok(())
    }
}
