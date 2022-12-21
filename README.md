# Resend 

Resend is a easy to use, performant, customizable and extendable Rust library for little-endian/big-endian serializing and deserializing.

# Example


Two functions only: 

snd() for any Write implementors (File, TcpStream etc)

rcv() for any Read implementors (File, TcpStream etc)

Cargo.toml:
```toml
[dependencies]
#with little-endian feature
resend = {version = "0.1", features = ["little"]}
```
Code:
```rust
use resend::{Snd, Rcv};

let mut vec = Vec::new();
vec.snd(-8_i8)?;
vec.snd(22_u16)?;
vec.snd(0xFFABCDEF as u32)?;
vec.snd("Test")?;

let mut buf = &vec[..];
let v: i8 = buf.rcv()?;
let v: u16 = buf.rcv()?;
let v: u32 = buf.rcv()?;
let v: String = buf.rcv()?;
```
## Derive
```toml
[dependencies]
resend = {version = "0.1", features = ["little"]}
resend_derive = "0.1"
```

```rust
use resend::{Snd, Rcv, endian::{Ascii, UTF16}};
use resend_derive::{Snd, Rcv};

#[repr(u32)]
#[derive(Snd, Rcv)]
pub enum DeviceType {
    PrinterType(IoPrinter) = 4,
    ScardType = 0x20,
}
#[derive(Snd, Rcv)]
struct Device{
    device_id: u32,
    #[len(8)]
    dos_name: Ascii,
}

#[derive(Snd, Rcv)]
pub struct IoPrinter{
    device: Device,
    length: u32,
    flags: u32,
    code_page: u32,
    pnp_name_len: u32,
    driver_name_len: u32,
    #[len(pnp_name_len)]
    pnp_name: UTF16,
    #[len(driver_name_len)]
    driver_name: UTF16,
}

...
let dt: DeviceType = stream.rcv()?;
stream.snd(&dt)?;

```

# Performant
Write/Read trait based, no intermediate variables.

# Format

- bool is serialized as 0_u8 (false) or 1_u8 (true).
- String, Vec, Array, Slice, Collections, Ascii, UTF16:
u32_length_header + data, no lengh header if "len" attribute is used.
- Option is serialized as bool_header + optional data, no bool_header if "when" attribute is used.
- Enum is serialized as tag value(int) + optional data. Use "repr" attribute for the size of tag value.

```rust
#[derive(Snd, Rcv)]
#[repr(u16)]
enum Color {
    Red,
    Blue = 32,
    Green =4,
}

```
Color::Red is serialized as 0_u16. Color::Blue is serialized as 32_u16.

```rust
#[repr(u32)]
#[derive(Snd, Rcv)]
pub enum DeviceType {
    PrinterType(IoPrinter) = 4,
    ScardType = 0x20,
}
```
DeviceType::PrinterType(printer) is serialized as 4_u32 + IoPrinter data.
DeviceType::ScardType is serialized as 0x20_u32.

Please be aware: [discriminants on non-unit variants are experimental for now (Rust 1.65)](https://github.com/rust-lang/rust/issues/60553), you have to use Rust nightly for this.

# Customizable (attributes)

1. Send both little-endian and big-endian at the same time with the resend::endian::little::LE and resend::endian::big::BE:

```rust
stream.snd(BE(100_u32))?;
```

2. No serialization with #[skip] attribute.

3. The length of String, Vector etc. can be from another field or constant with #[len(field_name_or_const)] attribute:
```rust
#[len(pnp_name_len)]
#[len(8)]
```
4. #[when(expr)] attribute is used on Option field. This field will be deserialized only if the expr is true. Warning: "expr" is not checked on serializing, no extra bool value in this case.

```rust
#[when(code_page > 0)]
#[when((flags & 2) != 0)]
```
5. Length can be u16 or [VLQ](https://en.wikipedia.org/wiki/Variable-length_quantity) with features (u32 by default)
```toml
resend = {version = "0.1", features = ["little", "len_16"]}
resend = {version = "0.1", features = ["big", "len_vlq"]}
```

6. Restricted length with features: MAX_LEN_100M, MAX_LEN_500M, MAX_LEN_2G
```toml
resend = {version = "0.1", features = ["little", "len_16", "MAX_LEN_100M"]}
```

# Extendable

For example, you want a string with [variable-length quantity](https://en.wikipedia.org/wiki/Variable-length_quantity)

```rust
pub struct VarLenString (pub String);

impl Sendable for VarLenString {
    fn snd_to<S>(&self, writer: &mut S) -> io::Result<()>
    where
        S: resend::Sender {
        writer.snd(resend::endian::VLQ(self.0.len()))?;
        writer.snd_all(self.0.as_bytes())
    }
}

impl Receivable for VarLenString {
    fn rcv_from<R>(reader: &mut R) -> io::Result<Self>
    where
        R: resend::Receiver {
        let len: VLQ = reader.rcv()?;
        let b = reader.rcv_bytes(*len)?;
        let s = std::str::from_utf8(&b)?;
        Ok(Self(s.to_string()))
    }
}
```
Resend includes the following types for your convenience:

```rust
use resend::endian::{Ascii, UTF16, UTF16Char, VLQ};
```

Implements resend::FromReader and resend::IntoWriter if you need the "len(field_name)" attribute working on your type. For example:
```rust
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
```

# Tips
1. String, Ascii and UTF16 with #[len(field_name_or_const)] attribute: if the specified length is bigger then the actual length: extra '\0' will be appended when it's serialized, and extra '\0' will be removed after it's deserialized; if the specified length is smaller, the string will be truncated to that length. This is useful if you need null terminated, fixed length string. 

2. Convert int to Enum
```rust
//Conver little-endian u16 to Blue ("little" feature)
//[u8] doesn't implement Read, convert it to &[u8] with as_ref()
let c: Color = [32_u8, 0].as_ref().rcv()?;
```

3. Use enumeration to serialize Object Oriented classes: 
```Rust
//type value (enum tag value) after the parent class
struct YourObject {
    parent: ParentClass,
    child: EnumOfChildClass,
}
//type value (enum tag value) before the parent class
struct Child {
    parent: ParentClass,
    child_field,
    ...
}
enum {
    child1,
    child2,
}

```
4. resend::endian:Length handles 3 types: u32, u16, VLQ. It's better to use this Length type directly in your object.

# License
MIT OR Apache-2.0

# Credits
This library is developed for [Remote Spark Corp's RDP (Remote Desktop Protocol) Project](https://www.remotespark.com/html5.html).