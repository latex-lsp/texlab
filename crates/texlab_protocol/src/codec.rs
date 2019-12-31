use bytes::{BufMut, BytesMut};
use log::*;
use std::io::{Error, ErrorKind};
use std::option::Option;
use std::string::String;
use tokio_util::codec::{Decoder, Encoder};

pub struct LspCodec;

impl Decoder for LspCodec {
    type Item = String;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match parser::parse(src) {
            Ok((remaining, content)) => {
                trace!("Received message:\n{}\n", content);

                let offset = src.len() - remaining.len();
                let _ = src.split_to(offset);
                Ok(Some(content))
            }
            Err(error) => {
                if error.is_incomplete() {
                    Ok(None)
                } else {
                    Err(ErrorKind::InvalidData.into())
                }
            }
        }
    }
}

impl Encoder for LspCodec {
    type Item = String;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let message = format!("Content-Length: {}\r\n\r\n{}", item.len(), item);
        trace!("Sent message:\n{}\n", message);

        dst.reserve(message.len());
        dst.put(message.as_bytes());
        Ok(())
    }
}

mod parser {
    use nom::bytes::streaming::{tag, take, take_while};
    use nom::character::is_digit;
    use nom::character::streaming::line_ending;
    use nom::combinator::{map_res, opt};
    use nom::IResult;
    use std::str;

    pub fn parse(input: &[u8]) -> IResult<&[u8], String> {
        let (input, _) = opt(content_type)(input)?;
        let (input, length) = content_length(input)?;
        let (input, _) = opt(content_type)(input)?;
        let (input, _) = line_ending(input)?;
        let (input, content) = map_res(take(length), str::from_utf8)(input)?;
        Ok((input, content.to_owned()))
    }

    fn content_type(input: &[u8]) -> IResult<&[u8], &[u8]> {
        let (input, _) = tag("Content-Type: application/vscode-jsonrpc;charset=utf")(input)?;
        let (input, _) = opt(tag("-"))(input)?;
        let (input, _) = tag("8")(input)?;
        line_ending(input)
    }

    fn content_length(input: &[u8]) -> IResult<&[u8], usize> {
        let (input, _) = tag("Content-Length: ")(input)?;
        let (input, length) = map_res(take_while(is_digit), from_bytes)(input)?;
        let (input, _) = line_ending(input)?;
        Ok((input, length))
    }

    fn from_bytes(input: &[u8]) -> Result<usize, std::num::ParseIntError> {
        usize::from_str_radix(str::from_utf8(input).unwrap(), 10)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_content_type() {
            let result =
                content_type(b"Content-Type: application/vscode-jsonrpc;charset=utf-8\r\n");
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
            let result = parse(
                b"Content-Length: 2\r\nContent-Type: application/vscode-jsonrpc;charset=utf8\r\n\r\n{}",
            );
            assert_eq!(result.unwrap().1, "{}");
        }

        #[test]
        fn test_parse_message_type_first() {
            let result = parse(
                b"Content-Type: application/vscode-jsonrpc;charset=utf8\r\nContent-Length: 2\r\n\r\n{}",
            );
            assert_eq!(result.unwrap().1, "{}");
        }

        #[test]
        fn test_parse_message_without_type() {
            let result = parse(b"Content-Length: 2\r\n\r\n{}");
            assert_eq!(result.unwrap().1, "{}");
        }

        #[test]
        fn test_parse_message_incomplete() {
            let result = parse(b"Content-Length:");
            assert!(result.unwrap_err().is_incomplete());
        }

        #[test]
        fn test_parse_message_invalid() {
            let error = parse(b"foo").unwrap_err();
            assert!(!error.is_incomplete());
        }

        #[test]
        fn test_parse_message_overflow() {
            let result = parse(b"Content-Length: 4\r\n\r\n{}");
            assert!(result.unwrap_err().is_incomplete());
        }
    }
}
