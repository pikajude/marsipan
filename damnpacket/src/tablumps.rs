#![allow(non_camel_case_types)]

use htmlescape;

type Arg = Vec<u8>;

#[derive(Debug)]
pub enum Tablump {
    A(Arg, Arg), C_A,
    Abbr(Arg), C_Abbr,
    Acro(Arg), C_Acro,
    Avatar(Arg, Arg),
    B, C_B,
    Bcode, C_Bcode,
    Br,
    Code, C_Code,
    Dev(Arg, Arg),
    Embed(Arg, Arg, Arg), C_Embed,
    Emote(Arg, Arg, Arg, Arg, Arg),
    I, C_I,
    Iframe(Arg, Arg, Arg), C_Iframe,
    Img(Arg, Arg, Arg),
    Li, C_Li,
    Link(Arg, Option<Arg>),
    Ol, C_Ol,
    P, C_P,
    S, C_S,
    Sub, C_Sub,
    Sup, C_Sup,
    Thumb(Arg, Arg, Arg, Arg, Arg, Arg),
    U, C_U,
    Ul, C_Ul,
}

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

  | tag!("&bcode\t") => { |_| Tablump::Bcode }
  | tag!("&/bcode\t") => { |_| Tablump::C_Bcode }

  | tag!("&br\t") => { |_| Tablump::Br }

  | tag!("&code\t") => { |_| Tablump::Code }
  | tag!("&/code\t") => { |_| Tablump::C_Code }

  | do_parse!(
        tag!("&dev\t") >>
        arg1: arg >>
        arg2: arg >>
        (arg1, arg2)
    ) => { |(a,b)| Tablump::Dev(a,b) }

  | do_parse!(
        tag!("&embed\t") >>
        arg1: arg >>
        arg2: arg >>
        arg3: arg >>
        (arg1, arg2, arg3)
    ) => { |(a,b,c)| Tablump::Embed(a,b,c) }
  | tag!("&/embed\t") => { |_| Tablump::C_Embed }

  | do_parse!(
        tag!("&emote\t") >>
        arg1: arg >>
        arg2: arg >>
        arg3: arg >>
        arg4: arg >>
        arg5: arg >>
        (arg1, arg2, arg3, arg4, arg5)
    ) => { |(a,b,c,d,e)| Tablump::Emote(a,b,c,d,e) }

  | tag!("&i\t") => { |_| Tablump::I }
  | tag!("&/i\t") => { |_| Tablump::C_I }

  | do_parse!(
        tag!("&iframe\t") >>
        arg1: arg >>
        arg2: arg >>
        arg3: arg >>
        (arg1, arg2, arg3)
    ) => { |(a,b,c)| Tablump::Iframe(a,b,c) }
  | tag!("&/iframe\t") => { |_| Tablump::C_Iframe }

  | do_parse!(
        tag!("&img\t") >>
        arg1: arg >>
        arg2: arg >>
        arg3: arg >>
        (arg1, arg2, arg3)
    ) => { |(a,b,c)| Tablump::Img(a,b,c) }

  | tag!("&li\t") => { |_| Tablump::Li }
  | tag!("&/li\t") => { |_| Tablump::C_Li }

  | link

  | tag!("&ol\t") => { |_| Tablump::Ol }
  | tag!("&/ol\t") => { |_| Tablump::C_Ol }

  | tag!("&p\t") => { |_| Tablump::P }
  | tag!("&/p\t") => { |_| Tablump::C_P }

  | tag!("&s\t") => { |_| Tablump::S }
  | tag!("&/s\t") => { |_| Tablump::C_S }

  | tag!("&sub\t") => { |_| Tablump::Sub }
  | tag!("&/sub\t") => { |_| Tablump::C_Sub }

  | tag!("&sup\t") => { |_| Tablump::Sup }
  | tag!("&/sup\t") => { |_| Tablump::C_Sup }

  | do_parse!(
        tag!("&thumb\t") >>
        arg1: arg >>
        arg2: arg >>
        arg3: arg >>
        arg4: arg >>
        arg5: arg >>
        arg6: arg >>
        (arg1, arg2, arg3, arg4, arg5, arg6)
    ) => { |(a,b,c,d,e,f)| Tablump::Thumb(a,b,c,d,e,f) }

  | tag!("&u\t") => { |_| Tablump::U }
  | tag!("&/u\t") => { |_| Tablump::C_U }

  | tag!("&ul\t") => { |_| Tablump::Ul }
  | tag!("&/ul\t") => { |_| Tablump::C_Ul }
));

named!(arg<Arg>, do_parse!(
    arg: take_until!("\t") >>
    tag!("\t") >>
    (arg.to_vec())
));

fn link(input: &[u8]) -> ::IResult<&[u8], Tablump> {
    let (i1, _) = try_parse!(input, tag!("&link\t"));
    let (i2, href) = try_parse!(i1, arg);
    let (i3, text) = try_parse!(i2, arg);
    match text.as_slice() {
        b"&" => ::IResult::Done(i3, Tablump::Link(href, None)),
        _ => {
            let (i4, _) = try_parse!(i3, tag!("&\t"));
            ::IResult::Done(i4, Tablump::Link(href, Some(text)))
        }
    }
}

impl Tablump {
    fn as_string(&self) -> String {
        use self::Tablump::*;

        match *self {
            A(ref x, ref y) => format!("<a href=\"{}\" title=\"{}\">", string!(x), string!(y)),
            C_A => "</a>".to_string(),
            Abbr(ref x) => format!("<abbr title=\"{}\">", string!(x)),
            C_Abbr => "</abbr>".to_string(),
            Acro(ref x) => format!("<acronym title=\"{}\">", string!(x)),
            C_Acro => "</acronym>".to_string(),
            Avatar(ref x, _) => format!(":icon{}:", string!(x)),
            B => "<b>".to_string(),
            C_B => "</b>".to_string(),
            Bcode => "<bcode>".to_string(),
            C_Bcode => "</bcode>".to_string(),
            Br => "<br/>".to_string(),
            Code => "<code>".to_string(),
            C_Code => "</code>".to_string(),
            Dev(_, ref a) => format!(":dev{}:", string!(a)),
            Embed(ref a, _, _) => format!("<embed src=\"{}\">", string!(a)),
            C_Embed => "</embed>".to_string(),
            Emote(ref a, _, _, _, _) => string!(a),
            I => "<i>".to_string(),
            C_I => "</i>".to_string(),
            Iframe(ref a, _, _) => format!("<iframe src=\"{}\">", string!(a)),
            C_Iframe => "</iframe>".to_string(),
            Img(ref a, _, _) => format!("<img src=\"{}\" />", string!(a)),
            Li => "<li>".to_string(),
            C_Li => "</li>".to_string(),
            Link(ref a, None) => string!(a),
            Link(ref a, Some(ref b)) => format!("{} ({})", string!(a), string!(b)),
            Ol => "<ol>".to_string(),
            C_Ol => "</ol>".to_string(),
            P => "<p>".to_string(),
            C_P => "</p>".to_string(),
            S => "<s>".to_string(),
            C_S => "</s>".to_string(),
            Sub => "<sub>".to_string(),
            C_Sub => "</sub>".to_string(),
            Sup => "<sup>".to_string(),
            C_Sup => "</sup>".to_string(),
            Thumb(ref a, _, _, _, _, _) => format!(":thumb{}:", string!(a)),
            U => "<u>".to_string(),
            C_U => "</u>".to_string(),
            Ul => "<ul>".to_string(),
            C_Ul => "</ul>".to_string(),
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

    for tok in t.into_iter() {
        match tok {
            Either::A(s) => {
                let str_ = string!(s);
                match htmlescape::decode_html(str_.as_str()) {
                    Ok(s_) => res.push_str(s_.as_str()),
                    Err(e) => {
                        warn!("HTML decoding error: {:?}", e);
                        res.push_str(str_.as_str());
                    }
                }
            },
            Either::B(l) => {
                res.push_str(l.as_string().as_str());
            }
        }
    }

    res
}
