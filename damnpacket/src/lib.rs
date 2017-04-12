#[macro_use]
extern crate nom;
extern crate ansi_term;

use ansi_term::{ANSIByteStrings,Colour,Style};
use nom::*;
use std::collections::HashMap;
use std::io;

type Bytes = Vec<u8>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub name: Bytes,
    pub argument: Option<Bytes>,
    pub attrs: HashMap<Bytes, String>,
    pub body: Option<MessageBody>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageBody(Bytes);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubMessage {
    pub name: Option<Bytes>,
    pub argument: Option<Bytes>,
    pub attrs: HashMap<Bytes, String>,
    pub body: Option<MessageBody>,
}

impl MessageBody {
    pub fn submessage(&self) -> Result<SubMessage, nom::ErrorKind> {
        parse_submessage(self.0.as_slice()).to_result()
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

impl Message {
    pub fn get_attr<V>(&self, key: V) -> Option<&str>
        where V: Into<Vec<u8>> {
        self.attrs.get(&key.into()).map(|x|x.as_ref())
    }

    pub fn has_attr<V, S>(&self, key: V, value: S) -> bool
        where V: Into<Vec<u8>>,
              S: Into<String> {
        self.get_attr(key) == Some(value.into().as_ref())
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
        if let Some(MessageBody(ref body)) = self.body {
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
        ANSIByteStrings(&strings[..]).write_to(&mut io)
    }
}

fn attr(input: &[u8]) -> IResult<&[u8], (Bytes, String)> {
    let (i1, key) = try_parse!(input, alphanumeric);
    let (i2, _) = try_parse!(i1, tag!("="));
    let (i3, val) = try_parse!(i2, take_until!("\n"));
    let (i4, _) = try_parse!(i3, tag!("\n"));
    IResult::Done(i4,
                  (key.to_vec(), String::from_utf8(val.to_vec()).expect("not UTF8")))
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
            return IResult::Done(i3, Some(MessageBody(bvec)));
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
    parse_message(bs).to_result()
}

#[test]
fn parse_basic() {
    assert_eq!(parse(b"foo bar\nbaz=qux\n\nthis is the body\0"),
               Ok(Message {
                      name: b"foo".to_vec(),
                      argument: Some(b"bar".to_vec()),
                      attrs: vec![(b"baz".to_vec(), String::from("qux"))],
                      body: Some(MessageBody(b"this is the body\0".to_vec())),
                  }));
}

#[test]
fn parse_no_body() {
    assert_eq!(parse(b"foo bar\nbaz=qux\n\0"),
               Ok(Message {
                      name: b"foo".to_vec(),
                      argument: Some(b"bar".to_vec()),
                      attrs: vec![(b"baz".to_vec(), String::from("qux"))],
                      body: None,
                  }));
}

#[test]
fn parse_no_attrs() {
    assert_eq!(parse(b"foo bar\n\0"),
               Ok(Message {
                      name: b"foo".to_vec(),
                      argument: Some(b"bar".to_vec()),
                      attrs: vec![],
                      body: None,
                  }));
}

#[test]
fn parse_no_arg() {
    assert_eq!(parse(b"foo\n\0"),
               Ok(Message {
                      name: b"foo".to_vec(),
                      argument: None,
                      attrs: vec![],
                      body: None,
                  }));
}

#[test]
fn parse_sub() {
    let msg = parse(b"foo\n\na=b\nc=d\n\0")
        .expect("oh no")
        .body
        .expect("no body");
    assert_eq!(msg.submessage(),
               Ok(SubMessage {
                      name: None,
                      argument: None,
                      attrs: vec![(b"a".to_vec(), String::from("b")),
                                  (b"c".to_vec(), String::from("d"))],
                      body: None,
                  }))
}
