use std::{collections::HashMap, ops::Add};

#[derive(Clone, Debug, PartialEq)]
pub enum Path {
    Absolute(Vec<String>),
    Relative(Vec<String>),
}

impl Default for Path {
    fn default() -> Self {
        Self::Absolute(Vec::new())
    }
}

impl Path {
    pub fn new(path: &str) -> Self {
        if path.starts_with('.') {
            Self::Relative(path.split('/').map(str::to_string).collect())
        } else {
            Self::Absolute(path.split('/').map(str::to_string).collect())
        }
        .canonicalize()
    }

    pub fn segments(&self) -> impl Iterator<Item = &String> {
        match self {
            Self::Relative(p) => p.iter(),
            Self::Absolute(p) => p.iter(),
        }
    }

    pub fn into_segments(self) -> impl Iterator<Item = String> {
        match self {
            Self::Relative(p) => p.into_iter(),
            Self::Absolute(p) => p.into_iter(),
        }
    }

    fn canonicalize(self) -> Self {
        let segments = match self {
            s @ Path::Relative(_) => return s,
            Path::Absolute(p) => p,
        };

        Self::Absolute(segments.into_iter().fold(Vec::new(), |mut acc, seg| {
            match seg.as_str() {
                "." | "" => (),
                ".." => {
                    acc.pop();
                }
                _ => {
                    acc.push(seg);
                }
            }

            acc
        }))
    }

    fn into_inner(self) -> Vec<String> {
        match self {
            Self::Relative(p) => p,
            Self::Absolute(p) => p,
        }
    }
}

impl Add for Path {
    type Output = Path;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Self::Absolute(mut p) => Self::Absolute({
                p.append(&mut rhs.into_inner());
                p
            }),
            Self::Relative(mut p) => Self::Relative({
                p.append(&mut rhs.into_inner());
                p
            }),
        }
        .canonicalize()
    }
}

impl ToString for Path {
    fn to_string(&self) -> String {
        match self {
            Self::Absolute(p) => String::from("/") + &p.join("/"),
            Self::Relative(p) => String::from("./") + &p.join("/"),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Params(HashMap<String, String>);

impl Params {
    pub fn get(&self, k: &str) -> Option<&String> {
        self.0.get(k)
    }

    fn insert(&mut self, k: &str, v: &str) {
        self.0.insert(k.to_owned(), v.to_owned());
    }
}

#[derive(Debug, Default, Clone)]
pub struct PathMatch {
    pub params: Params,
    pub segments: Vec<String>,
}

impl PartialEq for PathMatch {
    fn eq(&self, other: &Self) -> bool {
        self.segments == other.segments
    }
}

impl Path {
    pub fn matches(&self, other: &Path, offset: usize) -> Option<PathMatch> {
        let (mut r, mut p) = (self.segments(), other.segments().skip(offset));
        let mut mtch = PathMatch::default();

        loop {
            match (r.next(), p.next()) {
                (Some(r), Some(p)) if r.starts_with(':') => {
                    mtch.params.insert(r.strip_prefix(':').unwrap(), p);
                    mtch.segments.push(p.to_owned());
                }
                (Some(r), Some(p)) if r == p => {
                    mtch.segments.push(p.to_owned());
                }
                (None, _) => {
                    return Some(mtch);
                }
                _ => {
                    return None;
                }
            }
        }
    }
}

#[test]
fn test() {
    assert_eq!(
        String::from("/foo/bar/1/10"),
        (Path::new("foo///bar/1/2/3") + Path::new("././../../10")).to_string()
    );
}
