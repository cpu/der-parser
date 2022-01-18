use crate::ber::BitStringObject;
use crate::ber::{BerObject, BerObjectContent};
use alloc::string::String;
use alloc::vec::Vec;
use asn1_rs::Tag;
use core::fmt;
use core::iter::FromIterator;
use core::str;

use rusticata_macros::debug;

#[derive(Clone, Debug, PartialEq)]
pub enum PrettyPrinterFlag {
    ShowHeader,
}

pub struct PrettyBer<'a> {
    obj: &'a BerObject<'a>,
    indent: usize,
    inc: usize,

    flags: Vec<PrettyPrinterFlag>,
}

impl<'a> BerObject<'a> {
    pub fn as_pretty(&'a self, indent: usize, increment: usize) -> PrettyBer<'a> {
        PrettyBer {
            obj: self,
            indent,
            inc: increment,

            flags: Vec::new(),
        }
    }
}

impl<'a> PrettyBer<'a> {
    pub fn set_flag(&mut self, flag: PrettyPrinterFlag) {
        if !self.flags.contains(&flag) {
            self.flags.push(flag);
        }
    }

    pub fn next_indent<'b>(&self, obj: &'b BerObject) -> PrettyBer<'b> {
        PrettyBer {
            obj,
            indent: self.indent + self.inc,
            inc: self.inc,
            flags: self.flags.to_vec(),
        }
    }
}

impl<'a> fmt::Debug for PrettyBer<'a> {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.indent > 0 {
            write!(f, "{:1$}", " ", self.indent)?;
        };
        if self.flags.contains(&PrettyPrinterFlag::ShowHeader) {
            write!(f, "[c:{:?}, s:{}, t:{}] ", self.obj.header.class(), self.obj.header.constructed(), self.obj.header.tag())?;
        };
        fn print_utf32_string_with_type(f: &mut fmt::Formatter, s: &[u8], ty: &str) -> fmt::Result {
            let chars: Option<Vec<char>> = s
                .chunks_exact(4)
                .map(|a| core::char::from_u32(u32::from_be_bytes([a[0], a[1], a[2], a[3]])))
                .collect();

            match chars {
                Some(b)  => writeln!(f, "{}(\"{}\")", ty, String::from_iter(b)),
                None => writeln!(f, "{}({:?}) <error decoding utf32 string>", ty, s),
            }
        }
        match self.obj.content {
            BerObjectContent::EndOfContent           => writeln!(f, "EndOfContent"),
            BerObjectContent::Boolean(b)             => writeln!(f, "Boolean({:?})", b),
            BerObjectContent::Integer(i)             => writeln!(f, "Integer({:?})", debug::HexSlice(i)),
            BerObjectContent::Enum(i)                => writeln!(f, "Enum({})", i),
            BerObjectContent::OID(ref v)             => writeln!(f, "OID({:?})", v),
            BerObjectContent::RelativeOID(ref v)     => writeln!(f, "RelativeOID({:?})", v),
            BerObjectContent::Null                   => writeln!(f, "Null"),
            BerObjectContent::OctetString(v)         => writeln!(f, "OctetString({:?})", debug::HexSlice(v)),
            BerObjectContent::BitString(u,BitStringObject{data:v})
                                                     => writeln!(f, "BitString({},{:?})", u, debug::HexSlice(v)),
            BerObjectContent::GeneralizedTime(ref time)     => writeln!(f, "GeneralizedTime(\"{}\")", time),
            BerObjectContent::UTCTime(ref time)             => writeln!(f, "UTCTime(\"{}\")", time),
            BerObjectContent::VisibleString(s)       => writeln!(f, "VisibleString(\"{}\")", s),
            BerObjectContent::GeneralString(s)       => writeln!(f, "GeneralString(\"{}\")", s),
            BerObjectContent::GraphicString(s)       => writeln!(f, "GraphicString(\"{}\")", s),
            BerObjectContent::PrintableString(s)     => writeln!(f, "PrintableString(\"{}\")", s),
            BerObjectContent::NumericString(s)       => writeln!(f, "NumericString(\"{}\")", s),
            BerObjectContent::UTF8String(s)          => writeln!(f, "UTF8String(\"{}\")", s),
            BerObjectContent::IA5String(s)           => writeln!(f, "IA5String(\"{}\")", s),
            BerObjectContent::T61String(s)           => writeln!(f, "T61String({})", s),
            BerObjectContent::VideotexString(s)      => writeln!(f, "VideotexString({})", s),
            BerObjectContent::ObjectDescriptor(s)    => writeln!(f, "ObjectDescriptor(\"{}\")", s),
            BerObjectContent::BmpString(s)           => writeln!(f, "BmpString(\"{}\")", s),
            BerObjectContent::UniversalString(s)     => print_utf32_string_with_type(f, s, "UniversalString"),
            BerObjectContent::Optional(ref o) => {
                match o {
                    Some(obj) => writeln!(f, "OPTION {:?}", obj),
                    None => writeln!(f, "NONE"),
                }
            }
            BerObjectContent::Tagged(class, tag, ref obj) => {
                writeln!(f, "ContextSpecific [{} {}] {{", class, tag.0)?;
                write!(f, "{:?}", self.next_indent(obj))?;
                if self.indent > 0 {
                    write!(f, "{:1$}", " ", self.indent)?;
                };
                writeln!(f, "}}")?;
                Ok(())
            },
            BerObjectContent::Set(ref v) |
            BerObjectContent::Sequence(ref v)        => {
                let ty = if self.obj.header.tag() == Tag::Sequence { "Sequence" } else { "Set" };
                writeln!(f, "{}[", ty)?;
                for o in v {
                    write!(f, "{:?}", self.next_indent(o))?;
                };
                if self.indent > 0 {
                    write!(f, "{:1$}", " ", self.indent)?;
                };
                writeln!(f, "]")?;
                Ok(())
            },
            BerObjectContent::Unknown(ref any) => writeln!(f, "Unknown([{} {}] {:x?})", any.class(), any.tag().0, debug::HexSlice(any.data)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PrettyPrinterFlag;
    use crate::ber::*;

    #[test]
    fn test_pretty_print() {
        let d = BerObject::from_obj(BerObjectContent::Sequence(vec![
            BerObject::from_int_slice(b"\x01\x00\x01"),
            BerObject::from_int_slice(b"\x01\x00\x01"),
            BerObject::from_obj(BerObjectContent::Set(vec![
                BerObject::from_int_slice(b"\x01"),
                BerObject::from_int_slice(b"\x02"),
            ])),
        ]));

        println!("{:?}", d.as_pretty(0, 2));

        let mut pp = d.as_pretty(0, 4);
        pp.set_flag(PrettyPrinterFlag::ShowHeader);
        println!("{:?}", pp);
    }
}
