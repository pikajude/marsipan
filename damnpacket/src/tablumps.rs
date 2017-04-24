#![allow(non_camel_case_types)]

use htmlescape;

type ParsedText = Vec<Either<Vec<u8>, Tablump>>;

named!(pub tablumps<ParsedText>, map!(many0!(tok), collapse));

named!(tok<Either<Vec<u8>, Tablump>>, alt!(
    notamps => { |s: &[u8]| Either::A(s.to_vec()) }
  | lump => { |l| Either::B(l) }
  | tag!("&") => { |_| Either::A(b"&".to_vec()) }
));

named!(notamps, take_while1!(notamp));

fn notamp(c: u8) -> bool { c != b'&' }

named!(lump<Tablump>, alt!(
    do_parse!(
        tag!("&a\t") >>
        arg1: arg >>
        arg2: arg >>
        (arg1, arg2)
    ) => { |(x,y)| Tablump::A(x,y) }
  | tag!("&/a\t") => { |_| Tablump::C_A }

  | do_parse!(
        tag!("&abbr\t") >>
        arg1: arg >>
        (arg1)
    ) => { |x| Tablump::Abbr(x) }
  | tag!("&/abbr\t") => { |_| Tablump::C_Abbr }

  | do_parse!(
        tag!("&acro\t") >>
        arg1: arg >>
        (arg1)
    ) => { |x| Tablump::Acro(x) }
  | tag!("&/acro\t") => { |_| Tablump::C_Acro }

  | do_parse!(
        tag!("&avatar\t") >>
        arg1: arg >>
        arg2: arg >>
        (arg1, arg2)
    ) => { |(x,y)| Tablump::Avatar(x,y) }

  | tag!("&b\t") => { |_| Tablump::B }
  | tag!("&/b\t") => { |_| Tablump::C_B }
));

named!(arg<Arg>, do_parse!(
    arg: take_until!("\t") >>
    tag!("\t") >>
    (arg.to_vec())
));

type Arg = Vec<u8>;

#[derive(Debug)]
pub enum Tablump {
    A(Arg, Arg), C_A,
    Abbr(Arg), C_Abbr,
    Acro(Arg), C_Acro,
    Avatar(Arg, Arg),
    B, C_B,
}

impl Tablump {
    fn as_str(&self) -> &str {
        match *self {
            Tablump::B => "<b>",
            Tablump::C_B => "</b>",
            _ => panic!("{:?}", self)
        }
    }
}

#[derive(Debug)]
pub enum Either<A,B> {
    A(A),
    B(B),
}

fn collapse(i: ParsedText) -> ParsedText {
    let mut new = vec![];
    let mut left = vec![];

    for item in i.into_iter() {
        match item {
            Either::A(a) => left.extend(a),
            Either::B(b) => {
                if left.len() > 0 {
                    new.push(Either::A(left));
                    left = vec![];
                }
                new.push(Either::B(b));
            }
        }
    }

    if left.len() > 0 {
        new.push(Either::A(left))
    }

    new
}

pub fn render(t: ParsedText) -> String {
    let mut res = String::new();

    fn from_bytes(v: Vec<u8>) -> String {
        v.iter().map(|&c| c as char).collect()
    }

    for tok in t.into_iter() {
        match tok {
            Either::A(s) => {
                let str_ = from_bytes(s);
                match htmlescape::decode_html(str_.as_str()) {
                    Ok(s_) => res.push_str(s_.as_str()),
                    Err(e) => {
                        warn!("HTML decoding error: {:?}", e);
                        res.push_str(str_.as_str());
                    }
                }
            },
            Either::B(l) => {
                res.push_str(l.as_str());
            }
        }
    }

    res
}
