use std::collections::HashMap;
use std::ops::Add;
use std::str::FromStr;

#[derive(Debug)]
pub enum PathError {}

#[derive(Debug, PartialEq, Clone)]
pub struct Path {
    segments: Vec<String>,
    is_absolute: bool,
}

impl Default for Path {
    fn default() -> Self {
        Self {
            segments: Default::default(),
            is_absolute: true,
        }
    }
}

impl ToString for Path {
    fn to_string(&self) -> String {
        if self.is_absolute {
            String::from("/") + &self.segments.join("/")
        } else {
            self.segments.join("/")
        }
    }
}

impl FromStr for Path {
    type Err = PathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parser::parse_path(s)
    }
}

impl Path {
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    pub fn is_absolute(&self) -> bool {
        self.is_absolute
    }

    pub fn is_relative(&self) -> bool {
        !self.is_absolute
    }

    fn canonicalized(mut self) -> Self {
        // TODO: Enable relative paths too
        if self.is_relative() {
            return self;
        }

        self.segments = self.segments.into_iter().fold(vec![], |mut acc, seg| {
            match seg.as_str() {
                "." => (),
                ".." => {
                    acc.pop();
                }
                s => {
                    acc.push(s.to_string());
                }
            }

            acc
        });

        self
    }
}

impl Add for Path {
    type Output = Path;

    fn add(mut self, rhs: Self) -> Self::Output {
        for seg in rhs.segments {
            match seg.as_str() {
                ".." => {
                    self.segments.pop();
                }
                s => {
                    self.segments.push(s.to_string());
                }
            };
        }

        self
    }
}

#[derive(Debug, PartialEq, Clone)]
enum RouteSegment {
    Static(String),
    Param(String),
    Continue,
    SegmentWildcard,
    FullWildcard,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Params(HashMap<String, String>);

impl Params {
    pub fn get(&self, k: &str) -> Option<&String> {
        self.0.get(k)
    }
}

#[derive(Default)]
pub struct RouteMatch {
    pub remainder: Path,
    pub params: Params,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Route(Vec<RouteSegment>);

impl FromStr for Route {
    type Err = PathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parser::parse_route(s)
    }
}

impl Route {
    pub fn matches(&self, path: &Path) -> Option<RouteMatch> {
        let mut s = self.0.iter();
        let mut p = path.segments.iter();
        let mut mtch = RouteMatch::default();

        loop {
            match (s.next(), p.next()) {
                (Some(RouteSegment::FullWildcard), _) => {
                    return Some(mtch);
                }
                (Some(RouteSegment::SegmentWildcard), Some(_)) => {
                    continue;
                }
                (Some(RouteSegment::Static(seg)), Some(s)) if seg == s => {
                    continue;
                }
                (Some(RouteSegment::Param(p)), Some(s)) => {
                    mtch.params.0.insert(p.to_string(), s.to_string());
                }
                (Some(RouteSegment::Continue), Some(s)) => {
                    mtch.remainder.segments.push(s.to_string());
                }
                (Some(RouteSegment::Continue), None) => {
                    break;
                }
                (None, Some(s)) if !mtch.remainder.segments.is_empty() => {
                    mtch.remainder.segments.push(s.to_string());
                }
                (None, None) => {
                    break;
                }
                _ => {
                    return None;
                }
            }
        }

        Some(mtch)
    }
}

struct Parser<'p> {
    input: &'p str,
    index: usize,
}

impl<'p> Parser<'p> {
    pub fn parse_route(path: &'p str) -> Result<Route, PathError> {
        let mut result = vec![];

        let mut p = Self {
            input: path,
            index: 0,
        };

        p.skip_while(char::is_whitespace);

        loop {
            if p.peek() == '/' {
                p.consume_char();
            }

            if p.eol() {
                break;
            }

            match p.parse_segment() {
                Some(RouteSegment::Continue) => {
                    result.push(RouteSegment::Continue);
                    break;
                }
                Some(seg) => result.push(seg),
                None => (),
            }
        }

        Ok(Route(result))
    }

    pub fn parse_path(path: &'p str) -> Result<Path, PathError> {
        let mut segments = vec![];

        let mut p = Self {
            input: path,
            index: 0,
        };

        p.skip_while(char::is_whitespace);

        let is_absolute = p.peek() != '.';

        loop {
            if p.peek() == '/' {
                p.consume_char();
            }

            if p.eol() {
                break;
            }

            if p.peek() == '.' {
                match p.consume_while(|c| c == '.') {
                    s if s == ".." => segments.push(s),
                    s if s == "." => (),
                    _ => todo!(),
                }
            }

            match p.parse_static() {
                Some(RouteSegment::Static(s)) => segments.push(s),
                _ => (),
            };
        }

        Ok(Path {
            segments,
            is_absolute,
        }
        .canonicalized())
    }

    fn parse_segment(&mut self) -> Option<RouteSegment> {
        match self.peek() {
            '{' => self.parse_param(),
            '.' => self.parse_continue(),
            '*' => self.parse_wildcard(),
            _ => self.parse_static(),
        }
    }

    fn parse_static(&mut self) -> Option<RouteSegment> {
        match self.consume_while(|c| c != '/').trim() {
            s if s.is_empty() => None,
            s => Some(RouteSegment::Static(s.to_string())),
        }
    }

    fn parse_param(&mut self) -> Option<RouteSegment> {
        self.consume_char();

        match self.consume_while(|c| c != '}' && c != '/') {
            s if s.is_empty() => None,
            s => {
                self.consume_char();
                Some(RouteSegment::Param(s))
            }
        }
    }

    fn parse_continue(&mut self) -> Option<RouteSegment> {
        match self.consume_while(|c| c == '.').as_str() {
            "..." => Some(RouteSegment::Continue),
            _ => None,
        }
    }

    fn parse_wildcard(&mut self) -> Option<RouteSegment> {
        match self.consume_while(|c| c == '*').as_str() {
            "*" => Some(RouteSegment::SegmentWildcard),
            "**" => Some(RouteSegment::FullWildcard),
            _ => None,
        }
    }

    fn consume_while<F>(&mut self, cond: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();

        while !self.eol() && cond(self.peek()) {
            result.push(self.consume_char());
        }

        result
    }

    fn skip_while<F>(&mut self, cond: F)
    where
        F: Fn(char) -> bool,
    {
        while !self.eol() && cond(self.peek()) {
            self.index += 1;
        }
    }

    fn consume_char(&mut self) -> char {
        self.index += 1;
        self.input.chars().nth(self.index - 1).unwrap_or_default()
    }

    fn eol(&self) -> bool {
        self.index >= self.input.len()
    }

    fn peek(&self) -> char {
        self.input.chars().nth(self.index).unwrap_or_default()
    }
}

#[test]
fn test_route_parser() {
    use RouteSegment::*;

    assert_eq!(Route::from_str("").unwrap(), Route(vec![]));
    assert_eq!(Route::from_str("/").unwrap(), Route(vec![]));
    assert_eq!(Route::from_str("//").unwrap(), Route(vec![]));

    assert_eq!(
        Route::from_str("foo//bar/{id}///...").unwrap(),
        Route(vec![
            Static("foo".to_string()),
            Static("bar".to_string()),
            Param("id".to_string()),
            Continue,
        ]),
    );
}

#[cfg(test)]
macro_rules! assert_path {
    ($path: literal, $expected: expr) => {
        let rel = $path.starts_with(".");
        let p = Path::from_str($path).unwrap();
        let e: &[&str] = $expected;
        assert!(p.is_absolute() == !rel);
        assert!(p.segments.iter().eq(e.iter()));
    };

    ($a: literal + $b: literal, $expected: expr) => {
        let rel = $a.starts_with(".");
        let p = dbg!(Path::from_str($a).unwrap() + Path::from_str($b).unwrap());
        let e: &[&str] = $expected;
        assert!(p.is_absolute() == !rel);
        assert!(p.segments.iter().eq(e.iter()));
    };
}

#[test]
fn test_path_parser() {
    assert_path!("", &[]);
    assert_path!("/", &[]);
    assert_path!("foobar", &["foobar"]);
    assert_path!("/foo/bar", &["foo", "bar"]);

    assert_path!(".", &[]);
    assert_path!("..", &[".."]);
    assert_path!("./../.", &[".."]);
}

#[test]
fn test_canonicalization() {
    assert_path!("/bla/../../../goo", &["goo"]);

    // TODO: Make it work!
    // assert_path!("./bla/../../../goo", &["..", "..", "goo"]);
}

#[test]
fn test_path_concatenation() {
    assert_path!("/foo/bar" + "goo", &["foo", "bar", "goo"]);
    assert_path!("/foo/bar" + "../goo", &["foo", "goo"]);
    assert_path!("/foo/bar" + "/bla/../../goo", &["foo", "bar", "goo"]);
    assert_path!("../foo" + "./bar", &["..", "foo", "bar"]);
    assert_path!("../foo" + "../bar", &["..", "bar"]);

    // TODO: Make it work!
    // assert_path!("../foo" + "../../bar", &["..", "..", "bar"]);
}
