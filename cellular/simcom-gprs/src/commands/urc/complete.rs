use super::{Data, ReadResult, Urc};
use atat::nom::{branch, bytes, character, combinator, sequence};

pub(super) fn parse_connection_status(resp: &[u8]) -> Option<Urc> {
    if let Ok((reminder, (id, _, status))) = sequence::tuple::<_, _, (), _>((
        character::complete::u8,
        bytes::complete::tag(", "),
        branch::alt((
            bytes::complete::tag("CONNECT OK"),
            bytes::complete::tag("CONNECT FAIL"),
            bytes::complete::tag("ALREADY CONNECT"),
            bytes::complete::tag("SEND OK"),
            bytes::complete::tag("CLOSED"),
        )),
    ))(resp) && reminder.is_empty() {
        let id = id as usize;
        Some(match status {
            b"CONNECT OK" => Urc::ConnectOk(id),
            b"CONNECT FAIL" => Urc::ConnectFail(id),
            b"ALREADY CONNECT" => Urc::AlreadyConnect(id),
            b"SEND OK" => Urc::SendOk(id),
            b"CLOSED" => Urc::Closed(id),
            _ => return None,
        })
    }
    else {
        None
    }
}

pub(super) fn parse_data_available(resp: &[u8]) -> Option<Urc> {
    if let Ok((reminder, (_, id))) = sequence::tuple::<_, _, (), _>((
        combinator::recognize(sequence::tuple((bytes::complete::tag("+CIPRXGET:"), combinator::opt(bytes::complete::tag(b" ")), bytes::complete::tag("1,")))),
            character::complete::u8,
    ))(resp) && reminder.is_empty() {
        Some(Urc::DataAvailable(id as usize))
    }
    else {
        None
    }
}

pub(super) fn parse_read_data(resp: &[u8]) -> Option<Urc> {
    if let Ok((reminder, (_, id, _, (_, pending_len, _, data)))) = sequence::tuple::<_, _, (), _>((
        combinator::recognize(sequence::tuple((bytes::complete::tag("+CIPRXGET:"), combinator::opt(bytes::complete::tag(b" ")), bytes::complete::tag("2,")))),
            character::complete::u8,
            bytes::complete::tag(","),
            combinator::flat_map(character::complete::u16, |data_len| {
                sequence::tuple((
                    bytes::complete::tag(","),
                    character::complete::u16,
                    bytes::complete::tag("\r\n"),
                    bytes::complete::take(data_len),
                ))
            }),
    ))(resp) && reminder.is_empty() {
        Some(Urc::ReadData(ReadResult { id: id as usize, data_len: data.len(), pending_len: pending_len as usize, data: Data::new(data) }))
    }
    else {
        None
    }
}

pub(super) fn parse_dns_error(resp: &[u8]) -> Option<Urc> {
    if let Ok((reminder, (_, error_code))) = sequence::tuple::<_, _, (), _>((
        bytes::complete::tag("+CDNSGIP: 0,"),
            character::complete::u8,
    ))(resp) && reminder.is_empty() {
        Some(Urc::DnsResult(Err(error_code as usize)))
    }
    else {
        None
    }
}
