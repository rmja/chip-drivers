use atat::{
    nom::{branch, bytes, character, combinator, sequence},
    AtDigester, Digester,
};

use crate::commands::urc::Urc;

pub struct SimcomDigester(AtDigester<Urc>);

impl SimcomDigester {
    pub fn new() -> Self {
        let inner = AtDigester::new()
            .with_custom_success(|buf| {
                let (_reminder, (head, data, tail)) = branch::alt((sequence::tuple((
                    combinator::success(&b""[..]),
                    combinator::success(&b""[..]),
                    bytes::streaming::tag(b"\r\nSHUT OK\r\n"),
                )),
                sequence::tuple((
                    bytes::streaming::tag(b"\r\n"),
                    combinator::recognize(sequence::tuple((
                        character::streaming::u8,
                        bytes::streaming::tag(", CLOSE OK")

                    ))),
                    bytes::streaming::tag(b"\r\n"),
                ))
            
            ))(buf)?;

                Ok((data, head.len() + data.len() + tail.len()))
            })
            .with_custom_error(|buf| {
                let (_reminder, (head, data, tail)) = branch::alt((sequence::tuple((
                    bytes::streaming::tag(b"\r\n"),
                    combinator::recognize(sequence::tuple((
                        character::streaming::u8,
                        bytes::streaming::tag(", "),
                        bytes::streaming::tag("SEND FAIL"),
                    ))),
                    bytes::streaming::tag(b"\r\n"),
                )),))(buf)?;

                Ok((data, head.len() + data.len() + tail.len()))
            });

        Self(inner)
    }
}

impl Default for SimcomDigester {
    fn default() -> Self {
        Self::new()
    }
}

impl Digester for SimcomDigester {
    fn digest<'a>(&mut self, buf: &'a [u8]) -> (atat::DigestResult<'a>, usize) {
        self.0.digest(buf)
    }
}
