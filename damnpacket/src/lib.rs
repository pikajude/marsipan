extern crate ansi_term;
#[macro_use] extern crate log;
#[macro_use] extern crate nom;
extern crate htmlescape;

use ansi_term::{ANSIByteStrings,Colour,Style};
use nom::*;
use std::collections::HashMap;
use std::io;

#[derive(Clone, Debug, Eq, PartialEq)]
struct AsciiBytes(Vec<u8>);

type Bytes = Vec<u8>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub name: Bytes,
    pub argument: Option<Bytes>,
    pub attrs: HashMap<Bytes, String>,
    pub body: Option<MessageBody>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageBody(AsciiBytes);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubMessage {
    pub name: Option<Bytes>,
    pub argument: Option<Bytes>,
    pub attrs: HashMap<Bytes, String>,
    pub body: Option<MessageBody>,
}

impl AsciiBytes {
    fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    fn decode(&self) -> String {
        let intermediate: String = self.trim().iter().map(|&c| c as char).collect();
        match htmlescape::decode_html(intermediate.as_str()) {
            Ok(s) => s,
            Err(e) => {
                warn!("HTML decoding error: {:?}", e);
                intermediate
            }
        }
    }

    fn trim(&self) -> &[u8] {
        let len = self.0.len();
        let target_len = if len > 1 && self.0[len - 2] == b'\n' {
            len - 2
        } else if self.0[len - 1] == b'\0' {
            len - 1
        } else {
            len
        };
        &self.0[0..target_len]
    }
}

impl MessageBody {
    pub fn submessage(&self) -> Result<SubMessage, nom::ErrorKind> {
        parse_submessage(self.0.as_slice()).to_result()
    }

    pub fn to_string(&self) -> String {
        self.0.decode()
    }
}

impl<'a> From<&'a [u8]> for Message {
    fn from(s: &'a [u8]) -> Self {
        parse(s).expect("no parse")
    }
}

impl From<&'static str> for Message {
    fn from(s: &'static str) -> Self {
        Self::from(s.as_bytes())
    }
}

pub trait MessageIsh {
    fn get_attr<V>(&self, key: V) -> Option<&str>
        where V: Into<Bytes>;

    fn has_attr<V>(&self, key: V) -> bool
        where V: Into<Bytes> {
        self.get_attr(key).is_some()
    }

    fn has_attr_of<V, S>(&self, key: V, value: S) -> bool
        where V: Into<Bytes>,
              S: AsRef<str> {
        self.get_attr(key).map(|q|q == value.as_ref()).unwrap_or(false)
    }

    fn body_(&self) -> &MessageBody;
}

impl Message {
    pub fn submessage(&self) -> Option<SubMessage> {
        match self.body {
            None => None,
            Some(ref m) => m.submessage().ok()
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(self.name.clone());
        if let Some(ref arg) = self.argument {
            bytes.extend(b" ");
            bytes.extend(arg);
        }
        bytes.extend(b"\n");
        for (k, v) in self.attrs.iter() {
            bytes.extend(k);
            bytes.extend(b"=");
            bytes.extend(v.as_bytes());
            bytes.extend(b"\n");
        }
        if let Some(MessageBody(AsciiBytes(ref body))) = self.body {
            bytes.extend(b"\n");
            bytes.extend(body);
        } else {
            bytes.extend(b"\0");
        }
        bytes
    }

    pub fn pretty<W>(&self, mut io: W) -> io::Result<()>
        where W: io::Write {
        let mut strings = vec![];
        let mut buf = vec![];
        strings.push(Colour::Green.paint(self.name.clone()));
        if let Some(ref arg) = self.argument {
            strings.push(Style::default().paint(&b" "[..]));
            strings.push(Colour::Yellow.paint(&arg[..]));
        }
        strings.push(Style::default().paint(&b"\n"[..]));
        for (k, v) in self.attrs.iter() {
            strings.push(Style::default().paint(&k[..]));
            strings.push(Colour::Fixed(11).paint(&b"="[..]));
            strings.push(Style::default().paint(v.as_bytes()));
            strings.push(Style::default().paint(&b"\n"[..]));
        }
        if let Some(MessageBody(ref m)) = self.body {
            strings.push(Style::default().paint(&b"\n"[..]));
            buf.extend(m.decode().as_bytes());
        }
        strings.push(Style::default().paint(buf));
        ANSIByteStrings(&strings[..]).write_to(&mut io)
    }
}

impl MessageIsh for Message {
    fn get_attr<V>(&self, key: V) -> Option<&str>
        where V: Into<Vec<u8>> {
        self.attrs.get(&key.into()).map(|x|x.as_str())
    }

    fn body_(&self) -> &MessageBody {
        match self.body {
            None => panic!("body_() but no body"),
            Some(ref b) => b
        }
    }
}

impl MessageIsh for SubMessage {
    fn get_attr<V>(&self, key: V) -> Option<&str>
        where V: Into<Vec<u8>> {
        self.attrs.get(&key.into()).map(|x|x.as_str())
    }

    fn body_(&self) -> &MessageBody {
        match self.body {
            None => panic!("invariant: body_ called with no body"),
            Some(ref b) => b
        }
    }
}

fn attr(input: &[u8]) -> IResult<&[u8], (Bytes, String)> {
    let (i1, key) = try_parse!(input, alphanumeric);
    let (i2, _) = try_parse!(i1, tag!("="));
    let (i3, val) = try_parse!(i2, take_until!("\n"));
    let (i4, _) = try_parse!(i3, tag!("\n"));
    IResult::Done(i4, (key.to_vec(), AsciiBytes(val.to_vec()).decode()))
}

fn pbody(input: &[u8]) -> IResult<&[u8], Option<MessageBody>> {
    let (i1, next) = try_parse!(input, be_u8);
    fn is0(x: u8) -> bool {
        x == 0
    }
    match next {
        b'\n' => {
            let (i2, body) = try_parse!(i1, take_till!(is0));
            let (i3, b0) = try_parse!(i2, tag!("\0"));
            let mut bvec = body.to_vec();
            bvec.extend(b0);
            return IResult::Done(i3, Some(MessageBody(AsciiBytes(bvec))));
        }
        _ => IResult::Done(i1, None),
    }
}

fn parse_message(input: &[u8]) -> IResult<&[u8], Message> {
    let (i1, nm) = try_parse!(input, alpha);
    let (i2, arg) = match i1[0] {
        b' ' => {
            let (a, b) = try_parse!(i1, take_until1!("\n"));
            (a, Some(&b[1..]))
        }
        _ => (i1, None),
    };
    let (i3, _) = try_parse!(i2, tag!("\n"));
    let (i4, attrs) = try_parse!(i3, many0!(attr));
    let (i5, body) = try_parse!(i4, pbody);
    IResult::Done(i5,
                  Message {
                      name: nm.to_vec(),
                      argument: arg.map(|x| x.to_vec()),
                      attrs: attrs.into_iter().collect(),
                      body: body,
                  })
}

fn parse_submessage(input: &[u8]) -> IResult<&[u8], SubMessage> {
    let (i1, ar) = try_parse!(input, opt!(attr));
    match ar {
        Some(pair) => {
            let (i2, more) = try_parse!(i1, many0!(attr));
            let (i3, body) = try_parse!(i2, pbody);
            let mut my_attrs = vec![];
            my_attrs.push(pair);
            my_attrs.extend(more);
            IResult::Done(i3,
                          SubMessage {
                              name: None,
                              argument: None,
                              attrs: my_attrs.into_iter().collect(),
                              body: body,
                          })
        }
        None => {
            let (i2, msg) = try_parse!(i1, parse_message);
            IResult::Done(i2,
                          SubMessage {
                              name: Some(msg.name),
                              argument: msg.argument,
                              attrs: msg.attrs,
                              body: msg.body,
                          })
        }
    }
}

pub fn parse<'a>(bs: &[u8]) -> Result<Message, nom::ErrorKind> {
    let res = parse_message(bs);
    match res {
        IResult::Done(x, out) => if x.len() == 0 {
            Ok(out)
        } else {
            Err(nom::ErrorKind::Custom(0xdeadbeef))
        },
        IResult::Error(e) => Err(e),
        IResult::Incomplete(_) => panic!("incomplete input passed to parser")
    }
}

#[test]
fn parse_basic() {
    assert_eq!(parse(b"foo bar\nbaz=qux\n\nthis is the body\0"),
               Ok(Message {
                      name: b"foo".to_vec(),
                      argument: Some(b"bar".to_vec()),
                      attrs: vec![(b"baz".to_vec(), String::from("qux"))].into_iter().collect(),
                      body: Some(MessageBody(AsciiBytes(b"this is the body\0".to_vec()))),
                  }));
}

#[test]
fn parse_no_body() {
    assert_eq!(parse(b"foo bar\nbaz=qux\n\0"),
               Ok(Message {
                      name: b"foo".to_vec(),
                      argument: Some(b"bar".to_vec()),
                      attrs: vec![(b"baz".to_vec(), String::from("qux"))].into_iter().collect(),
                      body: None,
                  }));
}

#[test]
fn parse_empty_attr() {
    assert_eq!(parse(b"foo\nbaz=\n\0"),
        Ok(Message {
            name: b"foo".to_vec(),
            argument: None,
            attrs: vec![(b"baz".to_vec(), String::from(""))].into_iter().collect(),
            body: None,
        }))
}

#[test]
fn parse_no_attrs() {
    assert_eq!(parse(b"foo bar\n\0"),
               Ok(Message {
                      name: b"foo".to_vec(),
                      argument: Some(b"bar".to_vec()),
                      attrs: HashMap::new(),
                      body: None,
                  }));
}

#[test]
fn parse_no_arg() {
    assert_eq!(parse(b"foo\n\0"),
               Ok(Message {
                      name: b"foo".to_vec(),
                      argument: None,
                      attrs: HashMap::new(),
                      body: None,
                  }));
}

#[test]
fn parse_sub() {
    let msg = Message::from("foo\n\na=b\nc=d\n\0").body.expect("no body");
    assert_eq!(msg.submessage(),
               Ok(SubMessage {
                      name: None,
                      argument: None,
                      attrs: vec![(b"a".to_vec(), String::from("b")),
                                  (b"c".to_vec(), String::from("d"))].into_iter().collect(),
                      body: None,
                  }))
}
