//! Content policy: the first line of defence for names that players choose (fighters, content, profiles) and that other players see.
//!
//! This is deliberately small and honest. It checks the *shape* of a name (length, characters), refuses names that borrow someone else's
//! trademarks (the game is original; see `docs/CONTENT_POLICY.md`), and lets the project add more terms (offensive words, anything a
//! community wants gone) as plain text, one per line, without recompiling. It cannot judge intent, images or whole sentences, and a
//! blocklist is always beatable; a game with public servers needs reporting and human moderation on top (see the policy document).
//!
//! Matching ignores case, spacing, punctuation and the common look-alike substitutions (0 for o, 3 for e, 1 for i, 4 for a, 5 for s, 7 for
//! t, @ for a, $ for s), so "M4r10" is caught as "mario". Terms shorter than four letters are matched only as the whole name, so ordinary
//! words that happen to contain a short term are not caught.

/// Longest name any player-chosen text may have.
pub const MAX_NAME: usize = 24;

/// Names of other people's characters and franchises, which original content must not use. (Not exhaustive: a starting list.)
pub const TRADEMARKS: &[&str] = &[
    "mario",
    "luigi",
    "nintendo",
    "zelda",
    "ganondorf",
    "kirby",
    "pikachu",
    "pokemon",
    "samus",
    "metroid",
    "marth",
    "falco",
    "starfox",
    "smashbros",
    "supersmash",
    "yoshi",
    "bowser",
    "donkeykong",
    "sonic",
    "megaman",
    "pacman",
    "fireemblem",
    "ssbu",
];

/// Why a name is refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    Empty,
    TooLong,
    BadCharacter(char),
    /// Spaces at the ends or two in a row.
    Untidy,
    /// Contains a term on the blocklist (the term is not shown back to the player).
    Blocked,
}

impl Problem {
    /// A sentence to show the player.
    pub fn message(&self) -> String {
        match self {
            Problem::Empty => "Give it a name.".to_string(),
            Problem::TooLong => format!("Names can be at most {MAX_NAME} characters."),
            Problem::BadCharacter(c) => format!("`{c}` cannot be used: names can have letters, numbers, spaces, - _ ' and ."),
            Problem::Untidy => "Names cannot start or end with a space or have two spaces in a row.".to_string(),
            Problem::Blocked => "That name is not allowed (it uses someone else's trademark or a blocked word). Pick another.".to_string(),
        }
    }
}

/// Lower case, look-alikes folded, everything that is not a letter or digit dropped.
fn fold(text: &str) -> String {
    text.chars()
        .filter_map(|c| {
            let c = c.to_ascii_lowercase();
            let c = match c {
                '0' => 'o',
                '1' | '!' | '|' => 'i',
                '3' => 'e',
                '4' | '@' => 'a',
                '5' | '$' => 's',
                '7' => 't',
                other => other,
            };
            c.is_ascii_alphanumeric().then_some(c)
        })
        .collect()
}

/// A blocklist: the built-in trademark terms plus whatever the project adds.
#[derive(Clone, Debug)]
pub struct Policy {
    terms: Vec<String>,
}

impl Default for Policy {
    fn default() -> Policy {
        Policy::new("")
    }
}

impl Policy {
    /// The built-in terms plus `extra`: plain text, one term per line, `#` starts a comment.
    pub fn new(extra: &str) -> Policy {
        let mut terms: Vec<String> = TRADEMARKS.iter().map(|t| fold(t)).collect();
        for line in extra.lines() {
            let line = line.split('#').next().unwrap_or("");
            let term = fold(line);
            if !term.is_empty() && !terms.contains(&term) {
                terms.push(term);
            }
        }
        Policy { terms }
    }

    /// Whether the text contains a blocked term (no shape checks).
    pub fn is_blocked(&self, name: &str) -> bool {
        let folded = fold(name);
        self.terms.iter().any(|t| {
            if t.len() < 4 {
                folded == *t
            } else {
                folded.contains(t.as_str())
            }
        })
    }

    /// Checks a player-chosen name.
    pub fn check_name(&self, name: &str) -> Result<(), Problem> {
        if name.trim().is_empty() {
            return Err(Problem::Empty);
        }
        if name.chars().count() > MAX_NAME {
            return Err(Problem::TooLong);
        }
        for c in name.chars() {
            let ok = c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '-' | '\'' | '.');
            if !ok {
                return Err(Problem::BadCharacter(c));
            }
        }
        if name.starts_with(' ') || name.ends_with(' ') || name.contains("  ") {
            return Err(Problem::Untidy);
        }
        if self.is_blocked(name) {
            return Err(Problem::Blocked);
        }
        Ok(())
    }

    /// A name that arrived from another player, made safe to show: control and odd characters dropped, spaces tidied, cut to the limit, and
    /// replaced by `fallback` if it is empty or blocked. Never fails, never returns anything `check_name` would refuse for its shape.
    pub fn clean_remote_name(&self, name: &str, fallback: &str) -> String {
        let kept: String = name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '-' | '\'' | '.'))
            .collect();
        let tidy = kept.split_whitespace().collect::<Vec<_>>().join(" ");
        let cut: String = tidy.chars().take(MAX_NAME).collect();
        let cut = cut.trim().to_string();
        if cut.is_empty() || self.is_blocked(&cut) {
            fallback.to_string()
        } else {
            cut
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_names_pass() {
        let p = Policy::default();
        for n in [
            "Duelist",
            "Big Bertha",
            "tiny_tim",
            "Dr. Spin",
            "Al's Bar-B-Q",
            "Player 2",
            "x",
        ] {
            assert_eq!(p.check_name(n), Ok(()), "{n}");
        }
    }

    #[test]
    fn the_shape_of_a_name_is_checked() {
        let p = Policy::default();
        assert_eq!(p.check_name(""), Err(Problem::Empty));
        assert_eq!(p.check_name("   "), Err(Problem::Empty));
        assert_eq!(p.check_name(&"x".repeat(25)), Err(Problem::TooLong));
        assert_eq!(p.check_name("bad/name"), Err(Problem::BadCharacter('/')));
        assert_eq!(
            p.check_name("emoji \u{1F600}"),
            Err(Problem::BadCharacter('\u{1F600}'))
        );
        assert_eq!(p.check_name("tab\there"), Err(Problem::BadCharacter('\t')));
        assert_eq!(p.check_name(" lead"), Err(Problem::Untidy));
        assert_eq!(p.check_name("two  spaces"), Err(Problem::Untidy));
        assert_eq!(p.check_name("trail "), Err(Problem::Untidy));
    }

    #[test]
    fn trademarks_are_refused_even_disguised() {
        let p = Policy::default();
        for n in [
            "Mario",
            "M4r10",
            "mar-io",
            "super MARIO bro",
            "Z3lda",
            "p1kachu",
            "N I N T E N D O",
            "Marth",
            "Smash Bros",
        ] {
            assert_eq!(p.check_name(n), Err(Problem::Blocked), "{n}");
        }
        // Words that merely contain a short term are not caught.
        assert_eq!(p.check_name("Maria"), Ok(()));
        assert_eq!(p.check_name("Wolf"), Ok(()));
    }

    #[test]
    fn the_project_can_add_terms_without_recompiling() {
        let p = Policy::new("# a comment\nbadword\n  Spaced Out  # trailing comment\n\nab\n");
        assert_eq!(p.check_name("my badword here"), Err(Problem::Blocked));
        assert_eq!(p.check_name("B4DW0RD"), Err(Problem::Blocked));
        assert_eq!(p.check_name("spacedout"), Err(Problem::Blocked));
        // A term of three letters or fewer only matches a whole name.
        assert_eq!(p.check_name("ab"), Err(Problem::Blocked));
        assert_eq!(p.check_name("cabin"), Ok(()));
        assert_eq!(Policy::default().check_name("badword"), Ok(()));
    }

    #[test]
    fn names_from_other_players_are_made_safe() {
        let p = Policy::default();
        assert_eq!(p.clean_remote_name("  Big   Bob  ", "Player"), "Big Bob");
        assert_eq!(p.clean_remote_name("a\u{0}b\u{1b}c\nd", "Player"), "abcd");
        assert_eq!(
            p.clean_remote_name(&"y".repeat(100), "Player").len(),
            MAX_NAME
        );
        assert_eq!(p.clean_remote_name("", "Player"), "Player");
        assert_eq!(p.clean_remote_name("\u{202e}\u{1F600}", "Player"), "Player");
        assert_eq!(p.clean_remote_name("Mario", "Player"), "Player");
        // Whatever comes in, what goes out passes the shape check or is the fallback.
        for n in ["a\tb", "  ", "[[bb]]", "x y  z", "É"] {
            let cleaned = p.clean_remote_name(n, "Player");
            assert!(p.check_name(&cleaned).is_ok(), "{n:?} became {cleaned:?}");
        }
    }
}
