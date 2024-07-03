use atat::nom::{branch, bytes, character, combinator, error::ParseError, sequence, IResult};

/// Matches the equivalent of regex: \r\n+CGACT: [0-9],[0-9]
pub fn parse_pdp_state<'a, Error: ParseError<&'a [u8]>>(
    buf: &'a [u8],
) -> IResult<&'a [u8], (&'a [u8], usize), Error> {
    let (reminder, (_, frame)) = sequence::tuple((
        bytes::streaming::tag("\r\n"),
        combinator::recognize(sequence::tuple((
            bytes::streaming::tag("+CGACT: "),
            character::streaming::u8,
            bytes::streaming::tag(","),
            character::streaming::u8,
        ))),
    ))(buf)?;

    Ok((reminder, (frame, 2 + frame.len())))
}

/// Matches the equivalent of regex: \r\n[0-9], <tag>\r\n
pub fn parse_connection_status<'a, Error: ParseError<&'a [u8]>>(
    buf: &'a [u8],
) -> IResult<&'a [u8], (&'a [u8], usize), Error> {
    let (reminder, (_, frame, _)) = sequence::tuple((
        bytes::streaming::tag("\r\n"),
        combinator::recognize(sequence::tuple((
            character::streaming::u8,
            bytes::streaming::tag(", "),
            branch::alt((
                bytes::streaming::tag("CONNECT OK"),
                bytes::streaming::tag("CONNECT FAIL"),
                bytes::streaming::tag("ALREADY CONNECT"),
                bytes::streaming::tag("SEND OK"),
                bytes::streaming::tag("CLOSED"),
            )),
        ))),
        bytes::streaming::tag("\r\n"),
    ))(buf)?;

    Ok((reminder, (frame, 2 + frame.len() + 2)))
}

/// Matches the equivalent of regex: \r\nDATA ACCEPT: ?[0-9],[0-9]+\r\n
pub fn parse_data_accept<'a, Error: ParseError<&'a [u8]>>(
    buf: &'a [u8],
) -> IResult<&'a [u8], (&'a [u8], usize), Error> {
    let (reminder, (_, frame, _)) = sequence::tuple((
        bytes::streaming::tag("\r\n"),
        combinator::recognize(sequence::tuple((
            bytes::streaming::tag("DATA ACCEPT:"),
            combinator::opt(bytes::streaming::tag(b" ")),
            character::streaming::u8,
            bytes::streaming::tag(","),
            character::streaming::u16,
        ))),
        bytes::streaming::tag("\r\n"),
    ))(buf)?;

    Ok((reminder, (frame, 2 + frame.len() + 2)))
}

/// Matches the equivalent of regex: \r\n+CIPRXGET: ?1,[0-9]\r\n
pub fn parse_data_available<'a, Error: ParseError<&'a [u8]>>(
    buf: &'a [u8],
) -> IResult<&'a [u8], (&'a [u8], usize), Error> {
    let (reminder, (_, frame, _)) = sequence::tuple((
        bytes::streaming::tag("\r\n"),
        combinator::recognize(sequence::tuple((
            bytes::streaming::tag("+CIPRXGET:"),
            combinator::opt(bytes::streaming::tag(b" ")),
            bytes::streaming::tag("1,"),
            character::streaming::u8,
        ))),
        bytes::streaming::tag("\r\n"),
    ))(buf)?;

    Ok((reminder, (frame, 2 + frame.len() + 2)))
}

/// Matches the equivalent of regex: \r\n+CIPRXGET: ?2,[0-9],[0-9]+,[0-9]+\r\n
pub fn parse_read_data<'a, Error: ParseError<&'a [u8]>>(
    buf: &'a [u8],
) -> IResult<&'a [u8], (&'a [u8], usize), Error> {
    let (reminder, (_, frame)) = sequence::tuple((
        bytes::streaming::tag("\r\n"),
        combinator::recognize(sequence::tuple((
            bytes::streaming::tag(b"+CIPRXGET:"),
            combinator::opt(bytes::streaming::tag(b" ")),
            bytes::streaming::tag(b"2,"),
            character::streaming::u8,
            bytes::streaming::tag(","),
            combinator::flat_map(character::streaming::u16, |data_len| {
                combinator::recognize(sequence::tuple((
                    bytes::streaming::tag(","),
                    character::streaming::u16,
                    bytes::streaming::tag("\r\n"),
                    bytes::streaming::take(data_len),
                )))
            }),
        ))),
    ))(buf)?;

    Ok((reminder, (frame, 2 + frame.len())))
}

/// Matches the equivalent of regex: \r\n+RECEIVE,[0-9],[0-9]+\r\n
pub fn parse_receive<'a, Error: ParseError<&'a [u8]>>(
    buf: &'a [u8],
) -> IResult<&'a [u8], (&'a [u8], usize), Error> {
    let (reminder, (_, frame, _)) = sequence::tuple((
        bytes::streaming::tag("\r\n"),
        combinator::recognize(sequence::tuple((
            bytes::streaming::tag("+RECEIVE,"),
            character::streaming::u8,
            bytes::streaming::tag(","),
            character::streaming::u16,
            bytes::streaming::tag(":"),
        ))),
        bytes::streaming::tag("\r\n"),
    ))(buf)?;

    Ok((reminder, (frame, 2 + frame.len() + 2)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_pdp_state() {
        let (reminder, result) = parse_pdp_state::<()>(b"\r\n+CGACT: 1,0TAIL").unwrap();
        assert_eq!(b"TAIL", reminder);
        assert_eq!(b"+CGACT: 1,0", result.0);
        assert_eq!(13, result.1);
    }

    #[test]
    fn can_parse_connection_status() {
        let (reminder, result) =
            parse_connection_status::<()>(b"\r\n2, CONNECT OK\r\nTAIL").unwrap();
        assert_eq!(b"TAIL", reminder);
        assert_eq!(b"2, CONNECT OK", result.0);
        assert_eq!(17, result.1);
    }

    #[test]
    fn can_parse_data_accept_sim800() {
        let (reminder, result) = parse_data_accept::<()>(b"\r\nDATA ACCEPT: 1,2\r\nTAIL").unwrap();
        assert_eq!(b"TAIL", reminder);
        assert_eq!(b"DATA ACCEPT: 1,2", result.0);
        assert_eq!(20, result.1);
    }

    #[test]
    fn can_parse_data_accept_sim900() {
        let (reminder, result) = parse_data_accept::<()>(b"\r\nDATA ACCEPT:1,2\r\nTAIL").unwrap();
        assert_eq!(b"TAIL", reminder);
        assert_eq!(b"DATA ACCEPT:1,2", result.0);
        assert_eq!(19, result.1);
    }

    #[test]
    fn can_parse_data_available_sim800() {
        let (reminder, result) = parse_data_available::<()>(b"\r\n+CIPRXGET: 1,2\r\nTAIL").unwrap();
        assert_eq!(b"TAIL", reminder);
        assert_eq!(b"+CIPRXGET: 1,2", result.0);
        assert_eq!(18, result.1);
    }

    #[test]
    fn can_parse_data_available_sim900() {
        let (reminder, result) = parse_data_available::<()>(b"\r\n+CIPRXGET:1,2\r\nTAIL").unwrap();
        assert_eq!(b"TAIL", reminder);
        assert_eq!(b"+CIPRXGET:1,2", result.0);
        assert_eq!(17, result.1);
    }

    #[test]
    fn can_parse_read_data_sim800() {
        let (reminder, result) =
            parse_read_data::<()>(b"\r\n+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\nTAIL").unwrap();
        assert_eq!(b"TAIL", reminder);
        assert_eq!(b"+CIPRXGET: 2,5,8,0\r\nHTTP\r\n\r\n", result.0);
        assert_eq!(30, result.1);
    }

    #[test]
    fn can_parse_read_data_sim900() {
        let (reminder, result) =
            parse_read_data::<()>(b"\r\n+CIPRXGET:2,5,8,0\r\nHTTP\r\n\r\nTAIL").unwrap();
        assert_eq!(b"TAIL", reminder);
        assert_eq!(b"+CIPRXGET:2,5,8,0\r\nHTTP\r\n\r\n", result.0);
        assert_eq!(29, result.1);
    }

    #[test]
    fn can_parse_receive() {
        let (reminder, result) = parse_receive::<()>(b"\r\n+RECEIVE,2,1234:\r\nTAIL").unwrap();
        assert_eq!(b"TAIL", reminder);
        assert_eq!(b"+RECEIVE,2,1234:", result.0);
        assert_eq!(20, result.1);
    }
}
