use atat::AtatLen;
use serde::{Serialize, Serializer};

use crate::commands::gsm::Facility;

impl AtatLen for Facility {
    const LEN: usize = 2;
}

impl Serialize for Facility {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Facility::AO => serializer.serialize_bytes(b"\"AO\""),
            Facility::OI => serializer.serialize_bytes(b"\"OI\""),
            Facility::OX => serializer.serialize_bytes(b"\"OX\""),
            Facility::AI => serializer.serialize_bytes(b"\"AI\""),
            Facility::IR => serializer.serialize_bytes(b"\"IR\""),
            Facility::FD => serializer.serialize_bytes(b"\"FD\""),
            Facility::SC => serializer.serialize_bytes(b"\"SC\""),
            Facility::PN => serializer.serialize_bytes(b"\"PN\""),
            Facility::PU => serializer.serialize_bytes(b"\"PU\""),
            Facility::PP => serializer.serialize_bytes(b"\"PP\""),
        }
    }
}

#[cfg(test)]
mod tests {
    use atat::serde_at::{to_slice, SerializeOptions};

    use crate::commands::gsm::Facility;

    #[test]
    fn can_serialize() {
        let options = SerializeOptions {
            value_sep: false,
            ..SerializeOptions::default()
        };
        let mut buf = [0; 32];
        let len = to_slice(&Facility::SC, "", &mut buf, options).unwrap();
        assert_eq!(b"\"SC\"", &buf[..len]);
    }
}
