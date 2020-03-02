use minicbor::{Decode, Decoder, Encoder, data::{Tag, Type}, encode::{ExactSizeIter, Iter}};
use std::{collections::BTreeMap, iter::FromIterator};

// Test vectors of RFC 7049
//
// See https://github.com/cbor/test-vectors and https://tools.ietf.org/html/rfc7049#appendix-A

// Decode a hex-string and compare the result against an expected value.
// If successful, encode the result and compare against the original hex-string.
macro_rules! roundtrip {
    ($method:ident, $s:expr, $expected:expr) => {{
        let x = hex::decode($s).unwrap();
        let mut d = Decoder::from_slice(&x);
        let v = d.$method().unwrap();
        assert_eq!($expected, v);

        let mut e = Encoder::new(Vec::new());
        e.$method(v).unwrap();
        let y = hex::encode(e.into_inner());
        assert_eq!($s, y);
    }}
}

#[test]
fn rfc_tv_uint() {
    roundtrip!(u64, "00", 0);
    roundtrip!(u64, "01", 1);
    roundtrip!(u64, "0a", 10);
    roundtrip!(u64, "17", 23);
    roundtrip!(u64, "1818", 24);
    roundtrip!(u64, "1903e8", 1000);
    roundtrip!(u64, "1a000f4240", 1000000);
    roundtrip!(u64, "1b000000e8d4a51000", 1000000000000);
    roundtrip!(u64, "1bffffffffffffffff", 18446744073709551615);
}

#[test]
fn rfc_tv_int() {
    roundtrip!(i64, "20", -1);
    roundtrip!(i64, "29", -10);
    roundtrip!(i64, "3863", -100);
    roundtrip!(i64, "3903e7", -1000);
}

#[test]
fn rfc_tv_float() {
    fn decode(s: &str, expected: f64) {
        let x = hex::decode(s).unwrap();
        let v = minicbor::from_slice(&x).unwrap();
        assert_eq!(expected, v)
    }

    decode("f90000", 0.0);
    decode("f98000", -0.0);
    decode("f93c00", 1.0);
    decode("f93e00", 1.5);
    decode("f97bff", 65504.0);
    decode("f90001", 5.960464477539063e-08);
    decode("f90400", 6.103515625e-05);
    decode("f9c400", -4.0);
    decode("f97c00", std::f64::INFINITY);
    decode("f9fc00", std::f64::NEG_INFINITY);
    decode("fa7f800000", std::f64::INFINITY);
    decode("faff800000", std::f64::NEG_INFINITY);
    decode("fb7ff0000000000000", std::f64::INFINITY);
    decode("fbfff0000000000000", std::f64::NEG_INFINITY);

    roundtrip!(f32, "fa47c35000", 100000.0);
    roundtrip!(f32, "fa7f7fffff", 3.4028234663852886e+38);
    roundtrip!(f64, "fb3ff199999999999a", 1.1);
    roundtrip!(f64, "fb7e37e43c8800759c", 1.0e+300);
    roundtrip!(f64, "fbc010666666666666", -4.1);

    for s in &["f97e00", "fa7fc00000", "fb7ff8000000000000"] {
        let x = hex::decode(s).unwrap();
        let mut d = Decoder::from_slice(&x);
        let v = d.f64().unwrap();
        assert!(v.is_nan())
    }
}

#[test]
fn rfc_tv_small() {
    roundtrip!(bool, "f4", false);
    roundtrip!(bool, "f5", true);
    roundtrip!(simple, "f0", 16);
    roundtrip!(simple, "f818", 24);
    roundtrip!(simple, "f8ff", 255);

    let x = hex::decode("f6").unwrap();
    let d = Decoder::from_slice(&x);
    assert_eq!(Type::Null, d.datatype().unwrap());

    let x = hex::decode("f7").unwrap();
    let d = Decoder::from_slice(&x);
    assert_eq!(Type::Undefined, d.datatype().unwrap())
}

#[test]
fn rfc_tv_tagged() {
    // Tag::DateTime
    {
        let s = "c074323031332d30332d32315432303a30343a30305a";
        let x = hex::decode(s).unwrap();
        let mut d = Decoder::from_slice(&x);
        assert_eq!(Tag::DateTime, d.tag().unwrap());
        assert_eq!("2013-03-21T20:04:00Z", d.str().unwrap());

        let mut e = Encoder::new(Vec::new());
        e.tag(Tag::DateTime).unwrap();
        e.str("2013-03-21T20:04:00Z").unwrap();
        let y = hex::encode(e.into_inner());
        assert_eq!(s, y);
    }

    // Tag::Timestamp (u32)
    {
        let s = "c11a514b67b0";
        let x = hex::decode(s).unwrap();
        let mut d = Decoder::from_slice(&x);
        assert_eq!(Tag::Timestamp, d.tag().unwrap());
        assert_eq!(Type::U32, d.datatype().unwrap());
        assert_eq!(1363896240, d.u32().unwrap());

        let mut e = Encoder::new(Vec::new());
        e.tag(Tag::Timestamp).unwrap();
        e.u32(1363896240).unwrap();
        let y = hex::encode(e.into_inner());
        assert_eq!(s, y);
    }

    // Tag::Timestamp (f64)
    {
        let s = "c1fb41d452d9ec200000";
        let x = hex::decode(s).unwrap();
        let mut d = Decoder::from_slice(&x);
        assert_eq!(Tag::Timestamp, d.tag().unwrap());
        assert_eq!(Type::F64, d.datatype().unwrap());
        assert_eq!(1363896240.5, d.f64().unwrap());

        let mut e = Encoder::new(Vec::new());
        e.tag(Tag::Timestamp).unwrap();
        e.f64(1363896240.5).unwrap();
        let y = hex::encode(e.into_inner());
        assert_eq!(s, y);
    }

    // Tag::ToBase16
    {
        let s = "d74401020304";
        let x = hex::decode(s).unwrap();
        let mut d = Decoder::from_slice(&x);
        assert_eq!(Tag::ToBase16, d.tag().unwrap());
        assert_eq!(Type::Bytes, d.datatype().unwrap());
        assert_eq!([1, 2, 3, 4], d.bytes().unwrap());

        let mut e = Encoder::new(Vec::new());
        e.tag(Tag::ToBase16).unwrap();
        e.bytes(&[1, 2, 3, 4][..]).unwrap();
        let y = hex::encode(e.into_inner());
        assert_eq!(s, y);
    }

    // Tag::Cbor
    {
        let s = "d818456449455446";
        let x = hex::decode(s).unwrap();
        let mut d = Decoder::from_slice(&x);
        assert_eq!(Tag::Cbor, d.tag().unwrap());
        assert_eq!(Type::Bytes, d.datatype().unwrap());
        let mut g = Decoder::from_slice(d.bytes().unwrap());
        assert_eq!(Type::String, g.datatype().unwrap());
        assert_eq!("IETF", g.str().unwrap());

        let mut e = Encoder::new(Vec::new());
        e.str("IETF").unwrap();
        let mut f = Encoder::new(Vec::new());
        f.tag(Tag::Cbor).unwrap();
        f.bytes(e.as_ref()).unwrap();
        let y = hex::encode(f.into_inner());
        assert_eq!(s, y);
    }

    // Tag::Uri
    {
        let s = "d82076687474703a2f2f7777772e6578616d706c652e636f6d";
        let x = hex::decode(s).unwrap();
        let mut d = Decoder::from_slice(&x);
        assert_eq!(Tag::Uri, d.tag().unwrap());
        assert_eq!(Type::String, d.datatype().unwrap());
        assert_eq!("http://www.example.com", d.str().unwrap());

        let mut e = Encoder::new(Vec::new());
        e.tag(Tag::Uri).unwrap();
        e.str("http://www.example.com").unwrap();
        let y = hex::encode(e.into_inner());
        assert_eq!(s, y);
    }
}

#[test]
fn rfc_tv_bytes() {
    roundtrip!(bytes, "40", b"");
    roundtrip!(bytes, "4401020304", [1, 2, 3, 4]);
}

#[test]
fn rfc_tv_string() {
    roundtrip!(str, "60", "");
    roundtrip!(str, "6161", "a");
    roundtrip!(str, "6449455446", "IETF");
    roundtrip!(str, "62225c", "\"\\");
    roundtrip!(str, "62c3bc", "ü");
    roundtrip!(str, "63e6b0b4", "水");
    roundtrip!(str, "64f0908591", "𐅑");
}

#[test]
fn rfc_tv_bytes_indef() {
    let x = hex::decode("5f42010243030405ff").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq! {
        vec![&[1u8, 2][..], &[3, 4, 5][..]],
        d.bytes_iter().unwrap().map(Result::unwrap).collect::<Vec<_>>()
    }
    let mut e = Encoder::new(Vec::new());
    e.begin_bytes().unwrap();
    e.bytes(&[1, 2]).unwrap();
    e.bytes(&[3, 4, 5]).unwrap();
    e.end().unwrap();
    let y = hex::encode(e.into_inner());
    assert_eq!("5f42010243030405ff", y);
}

#[test]
fn rfc_tv_string_indef() {
    let x = hex::decode("7f657374726561646d696e67ff").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq! {
        vec!["strea", "ming"],
        d.str_iter().unwrap().map(Result::unwrap).collect::<Vec<_>>()
    }
    let mut e = Encoder::new(Vec::new());
    e.begin_str().unwrap();
    e.str("strea").unwrap();
    e.str("ming").unwrap();
    e.end().unwrap();
    let y = hex::encode(e.into_inner());
    assert_eq!("7f657374726561646d696e67ff", y);
}

#[test]
fn rfc_tv_array() {
    let x = hex::decode("80").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq!(Some(0), d.array().unwrap());
    let mut e = Encoder::new(Vec::new());
    e.array(0).unwrap();
    let y = hex::encode(e.into_inner());
    assert_eq!("80", y);

    let x = hex::decode("83010203").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq!(vec![1, 2, 3], (Decode::decode(&mut d) as Result<Vec<u8>, _>).unwrap());
    let y = hex::encode(minicbor::to_vec([1, 2, 3]).unwrap());
    assert_eq!("83010203", y);

    let x = hex::decode("8301820203820405").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq!(Some(3), d.array().unwrap());
    assert_eq!(1, d.u8().unwrap());
    assert_eq!(Some(2), d.array().unwrap());
    assert_eq!(2, d.u8().unwrap());
    assert_eq!(3, d.u8().unwrap());
    assert_eq!(Some(2), d.array().unwrap());
    assert_eq!(4, d.u8().unwrap());
    assert_eq!(5, d.u8().unwrap());
    let mut e = Encoder::new(Vec::new());
    e.array(3).unwrap();
    e.u8(1).unwrap();
    e.array(2).unwrap();
    e.u8(2).unwrap();
    e.u8(3).unwrap();
    e.array(2).unwrap();
    e.u8(4).unwrap();
    e.u8(5).unwrap();
    let y = hex::encode(e.into_inner());
    assert_eq!("8301820203820405", y);

    let x = hex::decode("98190102030405060708090a0b0c0d0e0f101112131415161718181819").unwrap();
    assert_eq! {
        (1 ..= 25).collect::<Vec<_>>(),
        minicbor::from_slice::<Vec<u8>>(&x).unwrap()
    }
    let y = hex::encode(minicbor::to_vec(ExactSizeIter::from(1u8 ..= 25)).unwrap());
    assert_eq!("98190102030405060708090a0b0c0d0e0f101112131415161718181819", y);

    let x = hex::decode("826161a161626163").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq!(Some(2), d.array().unwrap());
    assert_eq!("a", d.str().unwrap());
    assert_eq!(Some(1), d.map().unwrap());
    assert_eq!("b", d.str().unwrap());
    assert_eq!("c", d.str().unwrap());
    let mut e = Encoder::new(Vec::new());
    e.array(2).unwrap();
    e.str("a").unwrap();
    e.map(1).unwrap();
    e.str("b").unwrap();
    e.str("c").unwrap();
    let y = hex::encode(e.into_inner());
    assert_eq!("826161a161626163", y);
}

#[test]
fn rfc_tv_array_indef() {
    let x = hex::decode("9fff").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq!(None, d.array().unwrap());
    assert_eq!(Type::Break, d.datatype().unwrap());
    let mut e = Encoder::new(Vec::new());
    e.begin_array().unwrap();
    e.end().unwrap();
    let y = hex::encode(e.into_inner());
    assert_eq!("9fff", y);

    let x = hex::decode("9f018202039f0405ffff").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq!(None, d.array().unwrap());
    assert_eq!(1, d.u8().unwrap());
    assert_eq!(Some(2), d.array().unwrap());
    assert_eq!(2, d.u8().unwrap());
    assert_eq!(3, d.u8().unwrap());
    assert_eq!(None, d.array().unwrap());
    assert_eq!(4, d.u8().unwrap());
    assert_eq!(5, d.u8().unwrap());
    assert_eq!(Type::Break, d.datatype().unwrap());
    d.skip().unwrap();
    assert_eq!(Type::Break, d.datatype().unwrap());
    d.skip().unwrap();
    let mut e = Encoder::new(Vec::new());
    e.begin_array().unwrap();
    e.u8(1).unwrap();
    e.array(2).unwrap();
    e.u8(2).unwrap();
    e.u8(3).unwrap();
    e.begin_array().unwrap();
    e.u8(4).unwrap();
    e.u8(5).unwrap();
    e.end().unwrap();
    e.end().unwrap();
    let y = hex::encode(e.into_inner());
    assert_eq!("9f018202039f0405ffff", y);

    let x = hex::decode("9f01820203820405ff").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq!(None, d.array().unwrap());
    assert_eq!(1, d.u8().unwrap());
    assert_eq!(Some(2), d.array().unwrap());
    assert_eq!(2, d.u8().unwrap());
    assert_eq!(3, d.u8().unwrap());
    assert_eq!(Some(2), d.array().unwrap());
    assert_eq!(4, d.u8().unwrap());
    assert_eq!(5, d.u8().unwrap());
    assert_eq!(Type::Break, d.datatype().unwrap());
    d.skip().unwrap();
    let mut e = Encoder::new(Vec::new());
    e.begin_array().unwrap();
    e.u8(1).unwrap();
    e.array(2).unwrap();
    e.u8(2).unwrap();
    e.u8(3).unwrap();
    e.array(2).unwrap();
    e.u8(4).unwrap();
    e.u8(5).unwrap();
    e.end().unwrap();
    let y = hex::encode(e.into_inner());
    assert_eq!("9f01820203820405ff", y);

    let x = hex::decode("83018202039f0405ff").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq!(Some(3), d.array().unwrap());
    assert_eq!(1, d.u8().unwrap());
    assert_eq!(Some(2), d.array().unwrap());
    assert_eq!(2, d.u8().unwrap());
    assert_eq!(3, d.u8().unwrap());
    assert_eq!(None, d.array().unwrap());
    assert_eq!(4, d.u8().unwrap());
    assert_eq!(5, d.u8().unwrap());
    assert_eq!(Type::Break, d.datatype().unwrap());
    d.skip().unwrap();
    let mut e = Encoder::new(Vec::new());
    e.array(3).unwrap();
    e.u8(1).unwrap();
    e.array(2).unwrap();
    e.u8(2).unwrap();
    e.u8(3).unwrap();
    e.begin_array().unwrap();
    e.u8(4).unwrap();
    e.u8(5).unwrap();
    e.end().unwrap();
    let y = hex::encode(e.into_inner());
    assert_eq!("83018202039f0405ff", y);

    let x = hex::decode("83019f0203ff820405").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq!(Some(3), d.array().unwrap());
    assert_eq!(1, d.u8().unwrap());
    assert_eq!(None, d.array().unwrap());
    assert_eq!(2, d.u8().unwrap());
    assert_eq!(3, d.u8().unwrap());
    assert_eq!(Type::Break, d.datatype().unwrap());
    d.skip().unwrap();
    assert_eq!(Some(2), d.array().unwrap());
    assert_eq!(4, d.u8().unwrap());
    assert_eq!(5, d.u8().unwrap());
    let mut e = Encoder::new(Vec::new());
    e.array(3).unwrap();
    e.u8(1).unwrap();
    e.begin_array().unwrap();
    e.u8(2).unwrap();
    e.u8(3).unwrap();
    e.end().unwrap();
    e.array(2).unwrap();
    e.u8(4).unwrap();
    e.u8(5).unwrap();
    let y = hex::encode(e.into_inner());
    assert_eq!("83019f0203ff820405", y);

    let x = hex::decode("9f0102030405060708090a0b0c0d0e0f101112131415161718181819ff").unwrap();
    assert_eq! {
        (1 ..= 25).collect::<Vec<_>>(),
        minicbor::from_slice::<Vec<u8>>(&x).unwrap()
    }
    let y = hex::encode(minicbor::to_vec(Iter::from(1u8 ..= 25)).unwrap());
    assert_eq!("9f0102030405060708090a0b0c0d0e0f101112131415161718181819ff", y);

    let x = hex::decode("826161bf61626163ff").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq!(Some(2), d.array().unwrap());
    assert_eq!("a", d.str().unwrap());
    assert_eq!(None, d.map().unwrap());
    assert_eq!("b", d.str().unwrap());
    assert_eq!("c", d.str().unwrap());
    assert_eq!(Type::Break, d.datatype().unwrap());
    d.skip().unwrap();
    let mut e = Encoder::new(Vec::new());
    e.array(2).unwrap();
    e.str("a").unwrap();
    e.begin_map().unwrap();
    e.str("b").unwrap();
    e.str("c").unwrap();
    e.end().unwrap();
    let y = hex::encode(e.into_inner());
    assert_eq!("826161bf61626163ff", y);
}


#[test]
fn rfc_tv_map() {
    let x = hex::decode("a0").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq!(Some(0), d.map().unwrap());
    let mut e = Encoder::new(Vec::new());
    e.map(0).unwrap();
    let y = hex::encode(e.into_inner());
    assert_eq!("a0", y);


    let x = hex::decode("a201020304").unwrap();
    let m = BTreeMap::from_iter(vec![(1u8, 2u8), (3, 4)]);
    assert_eq!(m, minicbor::from_slice(&x).unwrap());
    let y = hex::encode(minicbor::to_vec(&m).unwrap());
    assert_eq!("a201020304", y);

    let x = hex::decode("a26161016162820203").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq!(Some(2), d.map().unwrap());
    assert_eq!("a", d.str().unwrap());
    assert_eq!(1, d.u8().unwrap());
    assert_eq!("b", d.str().unwrap());
    assert_eq!(Some(2), d.array().unwrap());
    assert_eq!(2, d.u8().unwrap());
    assert_eq!(3, d.u8().unwrap());
    let mut e = Encoder::new(Vec::new());
    e.map(2).unwrap();
    e.str("a").unwrap();
    e.u8(1).unwrap();
    e.str("b").unwrap();
    e.array(2).unwrap();
    e.u8(2).unwrap();
    e.u8(3).unwrap();
    let y = hex::encode(e.into_inner());
    assert_eq!("a26161016162820203", y);


    let x = hex::decode("a56161614161626142616361436164614461656145").unwrap();
    let m = BTreeMap::from_iter(vec![("a", "A"), ("b", "B"), ("c", "C"), ("d", "D"), ("e", "E")]);
    assert_eq!(m, minicbor::from_slice(&x).unwrap());
    let y = hex::encode(minicbor::to_vec(&m).unwrap());
    assert_eq!("a56161614161626142616361436164614461656145", y);


    let x = hex::decode("bf61610161629f0203ffff").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq!(None, d.map().unwrap());
    assert_eq!("a", d.str().unwrap());
    assert_eq!(1, d.u8().unwrap());
    assert_eq!("b", d.str().unwrap());
    assert_eq!(None, d.array().unwrap());
    assert_eq!(2, d.u8().unwrap());
    assert_eq!(3, d.u8().unwrap());
    assert_eq!(Type::Break, d.datatype().unwrap());
    d.skip().unwrap();
    assert_eq!(Type::Break, d.datatype().unwrap());
    d.skip().unwrap();
    let mut e = Encoder::new(Vec::new());
    e.begin_map().unwrap();
    e.str("a").unwrap();
    e.u8(1).unwrap();
    e.str("b").unwrap();
    e.begin_array().unwrap();
    e.u8(2).unwrap();
    e.u8(3).unwrap();
    e.end().unwrap();
    e.end().unwrap();
    let y = hex::encode(e.into_inner());
    assert_eq!("bf61610161629f0203ffff", y);

    let x = hex::decode("bf6346756ef563416d7421ff").unwrap();
    let mut d = Decoder::from_slice(&x);
    assert_eq!(None, d.map().unwrap());
    assert_eq!("Fun", d.str().unwrap());
    assert_eq!(true, d.bool().unwrap());
    assert_eq!("Amt", d.str().unwrap());
    assert_eq!(-2, d.i8().unwrap());
    assert_eq!(Type::Break, d.datatype().unwrap());
    d.skip().unwrap();
    let mut e = Encoder::new(Vec::new());
    e.begin_map().unwrap();
    e.str("Fun").unwrap();
    e.bool(true).unwrap();
    e.str("Amt").unwrap();
    e.i8(-2).unwrap();
    e.end().unwrap();
    let y = hex::encode(e.into_inner());
    assert_eq!("bf6346756ef563416d7421ff", y);
}