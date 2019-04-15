use bytes::{BufMut, BytesMut};
use log::*;
use nom::*;
use std::io::{Error, ErrorKind};
use std::option::Option;
use std::str;
use std::string::String;
use tokio::codec::{Decoder, Encoder};

pub struct LspCodec;

impl Decoder for LspCodec {
    type Item = String;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        trace!(
            "Received message:\n{}",
            str::from_utf8(&src.to_vec()).unwrap()
        );

        match parse_message(&src) {
            Ok((remaining, content)) => {
                src.split_to(src.len() - remaining.len());
                Ok(Some(content))
            }
            Err(error) => {
                if error.is_incomplete() {
                    Ok(None)
                } else {
                    Err(Error::from(ErrorKind::InvalidData))
                }
            }
        }
    }
}

impl Encoder for LspCodec {
    type Item = String;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put(format!("Content-Length: {}\r\n", item.len()));
        dst.put("\r\n");
        dst.put(item);

        trace!("Sent message:\n{}", str::from_utf8(&dst.to_vec()).unwrap());
        Ok(())
    }
}

fn from_bytes(input: &[u8]) -> Result<usize, std::num::ParseIntError> {
    usize::from_str_radix(str::from_utf8(input).unwrap(), 10)
}

named!(content_type<&[u8], ()>,
    do_parse!(
        tag!("Content-Type: application/vscode-jsonrpc;charset=utf") >>
        opt!(tag!("-")) >>
        tag!("8") >>
        line_ending >>
        ()
    )
);

named!(content_length<&[u8], usize>,
    do_parse!(
        tag!("Content-Length: ") >>
        length: map_res!(take_while!(is_digit), from_bytes) >>
        line_ending >>
        (length)
    )
);

named!(
    parse_message<&[u8], String>,
    do_parse!(
        opt!(content_type) >>
        length: content_length >>
        opt!(content_type) >>
        line_ending >>
        content: map_res!(take!(length), str::from_utf8) >>
        (content.to_string())
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_content_type() {
        let result = content_type(b"Content-Type: application/vscode-jsonrpc;charset=utf-8\r\n");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_content_type_utf8() {
        let result = content_type(b"Content-Type: application/vscode-jsonrpc;charset=utf8\r\n");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_content_length() {
        let result = content_length(b"Content-Length: 42\r\n");
        assert_eq!(result.unwrap().1, 42usize);
    }

    #[test]
    fn test_parse_message_full() {
        let result = parse_message(
            b"Content-Length: 2\r\nContent-Type: application/vscode-jsonrpc;charset=utf8\r\n\r\n{}",
        );
        assert_eq!(result.unwrap().1, "{}");
    }

    #[test]
    fn test_parse_message_type_first() {
        let result = parse_message(
            b"Content-Type: application/vscode-jsonrpc;charset=utf8\r\nContent-Length: 2\r\n\r\n{}",
        );
        assert_eq!(result.unwrap().1, "{}");
    }

    #[test]
    fn test_parse_message_without_type() {
        let result = parse_message(b"Content-Length: 2\r\n\r\n{}");
        assert_eq!(result.unwrap().1, "{}");
    }

    #[test]
    fn test_parse_message_incomplete() {
        let result = parse_message(b"Content-Length:");
        assert!(result.unwrap_err().is_incomplete());
    }

    #[test]
    fn test_parse_message_invalid() {
        let error = parse_message(b"foo").unwrap_err();
        assert!(!error.is_incomplete());
    }

    #[test]
    fn test_parse_message_overflow() {
        let result = parse_message(b"Content-Length: 4\r\n\r\n{}");
        assert!(result.unwrap_err().is_incomplete());
    }
}
