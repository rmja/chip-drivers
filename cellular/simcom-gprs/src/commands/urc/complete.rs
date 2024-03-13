use crate::{commands::gprs::PdpState, ContextId};

use super::{Data, ReadResult, Urc};
use atat::nom::{branch, bytes, character, combinator, sequence};

pub(super) fn parse_pdp_state(resp: &[u8]) -> Option<Urc> {
    if let Ok((reminder, (_, id, _, state))) = sequence::tuple::<_, _, (), _>((
        bytes::complete::tag("+CGACT: "),
        character::complete::u8,
        bytes::complete::tag(","),
        character::complete::u8,
    ))(resp)
    {
        if reminder.is_empty() {
            if reminder.is_empty() {
                return Some(Urc::PdbState(super::PdpContextState {
                    cid: ContextId(id),
                    state: PdpState::try_from(state).unwrap(),
                }));
            }
        }
    }

    None
}

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
    ))(resp)
    {
        if reminder.is_empty() {
            let id = id as usize;
            return Some(match status {
                b"CONNECT OK" => Urc::ConnectOk(id),
                b"CONNECT FAIL" => Urc::ConnectFail(id),
                b"ALREADY CONNECT" => Urc::AlreadyConnect(id),
                b"SEND OK" => Urc::SendOk(id),
                b"CLOSED" => Urc::Closed(id),
                _ => return None,
            });
        }
    }

    None
}

pub(super) fn parse_data_available(resp: &[u8]) -> Option<Urc> {
    if let Ok((reminder, (_, id))) = sequence::tuple::<_, _, (), _>((
        combinator::recognize(sequence::tuple((
            bytes::complete::tag("+CIPRXGET:"),
            combinator::opt(bytes::complete::tag(b" ")),
            bytes::complete::tag("1,"),
        ))),
        character::complete::u8,
    ))(resp)
    {
        if reminder.is_empty() {
            return Some(Urc::DataAvailable(id as usize));
        }
    }

    None
}

pub(super) fn parse_read_data(resp: &[u8]) -> Option<Urc> {
    if let Ok((reminder, (_, id, _, (_, pending_len, _, data)))) = sequence::tuple::<_, _, (), _>((
        combinator::recognize(sequence::tuple((
            bytes::complete::tag("+CIPRXGET:"),
            combinator::opt(bytes::complete::tag(b" ")),
            bytes::complete::tag("2,"),
        ))),
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
    ))(resp)
    {
        if reminder.is_empty() {
            return Some(Urc::ReadData(ReadResult {
                id: id as usize,
                data_len: data.len(),
                pending_len: pending_len as usize,
                data: Data::new(data),
            }));
        }
    }

    None
}

pub(super) fn parse_dns_error(resp: &[u8]) -> Option<Urc> {
    if let Ok((reminder, (_, error_code))) = sequence::tuple::<_, _, (), _>((
        bytes::complete::tag("+CDNSGIP: 0,"),
        character::complete::u8,
    ))(resp)
    {
        if reminder.is_empty() {
            return Some(Urc::DnsResult(Err(error_code as usize)));
        }
    }

    None
}
