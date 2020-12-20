use minicbor::{Encode, Decode};
use std::borrow::Cow;

#[derive(Encode, Decode)]
struct ByteSlice<'a> {
    #[n(0)]
    #[cbor(with = "minicbor::bytes")]
    field: &'a [u8]
}

#[derive(Encode, Decode)]
struct OptByteSlice<'a> {
    #[n(0)]
    #[cbor(with = "minicbor::bytes")]
    field: Option<&'a [u8]>
}

#[derive(Encode, Decode)]
struct ByteVec {
    #[n(0)]
    #[cbor(with = "minicbor::bytes")]
    field: Vec<u8>
}
