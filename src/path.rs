use std::{
    collections::HashMap,
    fmt::{Display, Write},
    ops::Add,
    str::FromStr,
};

// -----------------------------------------------------------------------------
//                                S E G M E N T
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    Parent,
    Parameter(String),
    Static(String),
}

impl Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::Parent => f.write_str(".."),
            Segment::Parameter(p) => write!(f, ":{p}"),
            Segment::Static(s) => f.write_str(s),
        }
    }
}

// -----------------------------------------------------------------------------
//                            P A T H  P A R S E R
// -----------------------------------------------------------------------------

#[derive(Default)]
struct Parser {
    allow_relative: bool,
    allow_parameters: bool,
}

impl Parser {
    pub fn allow_relative(mut self) -> Self {
        self.allow_relative = true;
        self
    }

    pub fn allow_parameters(mut self) -> Self {
        self.allow_parameters = true;
        self
    }

    pub fn parse(&self, value: &str) -> Result<Vec<Segment>, ()> {
        Ok(value.split('/').try_fold(Vec::new(), |mut acc, s| {
            match s {
                "" | "." => {
                    // Empty segments are skipped
                }
                ".." => {
                    if matches!(acc.last(), Some(Segment::Static(_))) {
                        acc.pop();
                    } else if self.allow_relative {
                        acc.push(Segment::Parent);
                    } else {
                        return Err(());
                    }
                }
                _ if s.starts_with(':') => {
                    if self.allow_parameters {
                        acc.push(Segment::Parameter(s.trim_start_matches(":").to_string()));
                    } else {
                        return Err(());
                    }
                }
                _ => {
                    acc.push(Segment::Static(s.to_string()));
                }
            }

            Ok(acc)
        })?)
    }
}

// -----------------------------------------------------------------------------
//                                  P A T H
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Path {
    Absolute(Vec<Segment>),
    Relative(Vec<Segment>),
}

impl Path {
    #[inline]
    pub fn is_absolute(&self) -> bool {
        matches!(self, Self::Absolute(_))
    }

    #[inline]
    pub fn segments(&self) -> &Vec<Segment> {
        match self {
            Self::Absolute(s) => s,
            Self::Relative(s) => s,
        }
    }

    #[inline]
    pub fn take_segments(self) -> Vec<Segment> {
        match self {
            Self::Absolute(s) => s,
            Self::Relative(s) => s,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.segments().len()
    }

    #[inline]
    pub fn skip(&self, n: usize) -> Self {
        if n == 0 {
            return self.clone();
        }

        match self {
            Self::Absolute(s) => Self::Absolute(s.clone().into_iter().skip(n).collect()),
            Self::Relative(s) => Self::Relative(s.clone().into_iter().skip(n).collect()),
        }
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::Absolute(Vec::new())
    }
}

impl FromStr for Path {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('.') {
            Ok::<_, ()>(Self::Relative(Parser::default().allow_relative().parse(s)?))
        } else {
            Ok::<_, ()>(Self::Absolute(Parser::default().parse(s)?))
        }
    }
}

impl<S> From<S> for Path
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        value.as_ref().parse().unwrap()
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_absolute() {
            f.write_char('/')?;
        }

        f.write_str(
            &(match self {
                Self::Absolute(s) => s,
                Self::Relative(s) => s,
            })
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("/"),
        )
    }
}

impl Add for Path {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let is_absolute = self.is_absolute();
        let mut segments = self.take_segments();

        if !rhs.is_absolute() {
            segments.pop();
        }

        let segments = rhs
            .take_segments()
            .into_iter()
            .fold(segments, |mut acc, s| {
                match s {
                    Segment::Parent => {
                        if matches!(acc.last(), Some(Segment::Static(_))) {
                            acc.pop();
                        } else if !is_absolute {
                            acc.push(Segment::Parent);
                        }
                    }
                    s @ Segment::Static(_) => {
                        acc.push(s);
                    }
                    Segment::Parameter(_) => {
                        unreachable!();
                    }
                }

                acc
            });

        if is_absolute {
            Self::Absolute(segments)
        } else {
            Self::Relative(segments)
        }
    }
}

// -----------------------------------------------------------------------------
//                                  R O U T E
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Route(Vec<Segment>);

impl Route {
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn match_path(&self, path: &Path) -> Option<(Path, HashMap<String, String>)> {
        if !path.is_absolute() || self.len() > path.len() {
            return None;
        }

        let mut segments = Vec::new();
        let mut parameters = HashMap::new();

        let mut route = self.0.iter();
        let mut subject = path.segments().iter();

        loop {
            match (route.next(), subject.next()) {
                (Some(Segment::Static(r)), Some(seg @ Segment::Static(p))) => {
                    if r == p {
                        segments.push(seg.clone());
                    } else {
                        return None;
                    }
                }
                (Some(Segment::Parameter(r)), Some(seg @ Segment::Static(p))) => {
                    segments.push(seg.clone());
                    parameters.insert(r.to_string(), p.to_string());
                }
                (Some(_), _) => {
                    return None;
                }
                (None, _) => {
                    break;
                }
            };
        }

        Some((Path::Absolute(segments), parameters))
    }
}

impl FromStr for Route {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Parser::default().allow_parameters().parse(s)?))
    }
}

impl Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &self
                .0
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("/"),
        )
    }
}

#[test]
fn test_parser() {
    assert_eq!("".parse(), Ok(Path::Absolute(Vec::new())));
    assert_eq!("/".parse(), Ok(Path::Absolute(Vec::new())));
    assert_eq!("///././/".parse(), Ok(Path::Absolute(Vec::new())));
    assert_eq!("a/..".parse(), Ok(Path::Absolute(Vec::new())));
    assert_eq!("a/.././..".parse::<Path>(), Err(()),);

    assert_eq!(".".parse(), Ok(Path::Relative(Vec::new())));
    assert_eq!("..".parse(), Ok(Path::Relative(vec![Segment::Parent])));

    assert_eq!(
        "../a".parse(),
        Ok(Path::Relative(vec![
            Segment::Parent,
            Segment::Static("a".to_string())
        ]))
    );

    assert_eq!(
        "a/b/c/../../d".parse(),
        Ok(Path::Absolute(vec![
            Segment::Static("a".to_string()),
            Segment::Static("d".to_string()),
        ]))
    );

    assert_eq!(
        "./a/b/../../..".parse(),
        Ok(Path::Relative(vec![Segment::Parent]))
    );

    assert_eq!(
        "./../a/b/../.".parse(),
        Ok(Path::Relative(vec![
            Segment::Parent,
            Segment::Static("a".to_string()),
        ]))
    );

    assert_eq!(
        "users/:id/details".parse(),
        Ok(Route(vec![
            Segment::Static("users".to_string()),
            Segment::Parameter("id".to_string()),
            Segment::Static("details".to_string())
        ]))
    );
}

#[test]
fn test_concatenation() {
    fn path(p: &str) -> Path {
        p.parse().unwrap()
    }

    assert_eq!(
        path("foo/bar") + path("./test"),
        Path::Absolute(vec![
            Segment::Static("foo".to_string()),
            Segment::Static("test".to_string()),
        ])
    );

    assert_eq!(
        path("foo/bar") + path("test"),
        Path::Absolute(vec![
            Segment::Static("foo".to_string()),
            Segment::Static("bar".to_string()),
            Segment::Static("test".to_string()),
        ])
    );

    assert_eq!(
        path("./.././foo/bar/test-a") + path("./test-b/../.."),
        Path::Relative(vec![Segment::Parent, Segment::Static("foo".to_string()),])
    );
}
