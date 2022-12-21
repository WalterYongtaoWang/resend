#![cfg(any(feature = "little", feature = "big"))]
use std::assert_eq;

use resend::{
    endian::{UTF16, VLQ},
    Rcv, Receivable, Sendable, Snd,
};
use resend_derive::{Rcv, Snd};

#[derive(Copy, Clone, Debug, Snd, Rcv, PartialEq)]
#[repr(u16)]
enum Color {
    Red = 2,
    Blue = 32,
    Green = 4,
}

#[derive(Snd, Rcv, PartialEq, Debug)]
pub struct Point {
    x: u16,
    y: u16,
    #[len(x)]
    s: String,
    #[len(y)]
    u: UTF16,
}

#[derive(Snd, Rcv, PartialEq, Debug)]
struct Person {
    name: String,
    age: u16,
    qty: u32,
    #[when(age > 30)]
    senior: Option<bool>,
    desc: UTF16,
    #[skip]
    ignore: u32,
    color: Color,
    point: Point,
}

//discriminants on non-unit variants are experimental
//https://github.com/rust-lang/rust/issues/60553
#[repr(u32)]
#[derive(Snd, Rcv, Debug, PartialEq)]
#[cfg(feature = "unstable")]
enum DeviceType {
    A,
    B = 2,
    Pt(Point) = 22,
}

pub struct VarLenString(pub String);

impl Sendable for VarLenString {
    fn snd_to<W>(&self, writer: &mut W) -> resend::Result<()>
    where
        W: resend::Sender,
    {
        writer.snd(resend::endian::VLQ(self.0.len()))?;
        writer.snd_all(self.0.as_bytes())
    }
}

impl Receivable for VarLenString {
    fn rcv_from<R>(reader: &mut R) -> resend::Result<Self>
    where
        R: resend::Receiver,
    {
        let len: VLQ = reader.rcv()?;
        let b = reader.rcv_bytes(*len)?;
        let s = std::str::from_utf8(&b)?;
        Ok(Self(s.to_string()))
    }
}

#[test]
fn test_point() -> resend::Result<()> {
    let mut vec = Vec::new();
    let p = Point {
        x: 5,
        y: 8,
        s: "1234".to_string(),
        u: UTF16("123".to_string()),
    };
    vec.snd(&p)?;

    let mut buf = &vec[..];
    let p1: Point = buf.rcv()?;
    assert_eq!(p, p1);
    Ok(())
}

#[test]
fn test_person() -> resend::Result<()> {
    let mut vec = Vec::new();
    let p = Person {
        name: "Great".to_string(),
        age: 32,
        qty: 0xF8342122,
        senior: Some(true),
        desc: UTF16("people".to_string()),
        ignore: 33,
        color: Color::Green,
        point: Point {
            x: 5,
            y: 12,
            s: "great".to_string(),
            u: UTF16("12345".to_string()),
        },
    };

    vec.snd(&p)?;

    vec.snd(2_u16)?;

    vec.snd(&234)?;

    let mut buf = &vec[..];
    let mut p1: Person = buf.rcv()?;
    assert_eq!(p1.ignore, 0);
    p1.ignore = 33;
    assert_eq!(p, p1);

    let v: u8 = buf.rcv()?;

    if v == 0 {
        println!("*** big endian");
    } else if v == 2 {
        println!("*** little endian");
    } else {
        panic!("Wrong");
    }

    Ok(())
}

#[cfg(feature = "unstable")]
#[test]
fn test_enum() -> resend::Result<()> {
    assert_eq!(32, Color::Blue as u32);
    let mut vec = Vec::new();
    vec.snd(&Color::Blue)?;

    vec.snd(&DeviceType::B)?;

    let p = Point {
        x: 5,
        y: 8,
        s: "1234".to_string(),
        u: UTF16("123".to_string()),
    };
    vec.snd(&DeviceType::Pt(p))?;

    let mut buf = &vec[..];
    let c: Color = buf.rcv()?;
    assert_eq!(c, Color::Blue);

    let dt: DeviceType = buf.rcv()?;
    assert_eq!(dt, DeviceType::B);

    let dt: DeviceType = buf.rcv()?;
    assert_eq!(
        dt,
        DeviceType::Pt(Point {
            x: 5,
            y: 8,
            s: "1234".to_string(),
            u: UTF16("123".to_string())
        })
    );

    Ok(())
}

#[test]
fn test_unit_enum() -> resend::Result<()> {
    assert_eq!(32, Color::Blue as u32);
    let mut vec = Vec::new();
    vec.snd(&Color::Blue)?;

    let mut buf = &vec[..];
    let c: Color = buf.rcv()?;
    assert_eq!(c, Color::Blue);

    #[cfg(feature = "little")]
    {
        let c: Color = [32_u8, 0].as_ref().rcv()?;
        assert_eq!(c, Color::Blue);
    }

    #[cfg(feature = "big")]
    {
        let c: Color = [0, 32_u8].as_ref().rcv()?;
        assert_eq!(c, Color::Blue);
    }

    Ok(())
}
