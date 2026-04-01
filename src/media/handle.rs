use std::sync::LazyLock;

use regex::Regex;

static HANDLE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(.+)\s\((\d{4})\)$").unwrap());

pub struct Handle {
    pub name: String,
    pub year: Option<u16>,
}

impl Handle {
    pub fn from_user_input(input: impl Into<String> + Copy) -> Self {
        match HANDLE_RE.captures(&input.into()) {
            Some(caps) => Handle {
                name: caps.get(1).unwrap().as_str().into(),
                year: caps.get(2).unwrap().as_str().parse::<u16>().ok(),
            },
            None => Handle {
                name: input.into(),
                year: None,
            },
        }
    }
}

impl std::fmt::Display for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.year {
            Some(y) => write!(f, "{} ({})", self.name, y),
            None => write!(f, "{}", self.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_from_user_input() {
        let handle = Handle::from_user_input("Alien (1979)");
        assert_eq!(handle.name, "Alien");
        assert_eq!(handle.year, Some(1979));

        let handle = Handle::from_user_input("Alien");
        assert_eq!(handle.name, "Alien");
        assert_eq!(handle.year, None);
    }
}
