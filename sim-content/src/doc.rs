//! An editable content document: the tree of a bundle file, with sections that can be read, replaced, added and
//! removed, and checked as a whole. The editors work on this: every edit puts a section back and the whole document is
//! read and validated again, so Rust stays the only judge of what is valid and the editors need no rules of their own.

use crate::bundle::{manifest_block, read_manifest, Bundle};
use crate::format::{read_content, write_content};
use crate::tree::{self, Block, Item};
use sim_core::Content;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Doc {
    pub root: Block,
}

/// Top-level section kinds that appear at most once (the name is only a label).
const SINGLE: [&str; 3] = ["bundle", "ruleset", "stage"];

impl Doc {
    /// A document for `content` with the given manifest text.
    pub fn from_content(content: &Content, name: &str, author: &str, description: &str) -> Doc {
        let mut root = Block::new("file", None);
        root.push(manifest_block(name, author, description));
        for section in write_content(content) {
            root.push(section);
        }
        Doc { root }
    }

    /// Reads a document from bundle text (a loose or packed bundle; the hash and sim version are not checked here,
    /// only when the bundle is loaded for play).
    pub fn parse(text: &str) -> Result<Doc, String> {
        let root = tree::parse(text)?;
        Ok(Doc { root })
    }

    /// The `(kind, name)` of every top-level section, in file order.
    pub fn sections(&self) -> Vec<(String, String)> {
        self.root
            .items
            .iter()
            .filter_map(|i| match i {
                Item::Block(b) => Some((b.kind.clone(), b.name.clone().unwrap_or_default())),
                _ => None,
            })
            .collect()
    }

    fn position(&self, kind: &str, name: &str) -> Option<usize> {
        self.root.items.iter().position(|i| match i {
            Item::Block(b) => {
                b.kind == kind
                    && (SINGLE.contains(&kind) || b.name.as_deref().unwrap_or("") == name)
            }
            _ => false,
        })
    }

    pub fn get(&self, kind: &str, name: &str) -> Option<&Block> {
        match self.root.items.get(self.position(kind, name)?) {
            Some(Item::Block(b)) => Some(b),
            _ => None,
        }
    }

    /// Replaces the section with this block's kind and name, or adds it. Returns whether it replaced one.
    pub fn put(&mut self, block: Block) -> bool {
        let name = block.name.clone().unwrap_or_default();
        match self.position(&block.kind, &name) {
            Some(i) => {
                self.root.items[i] = Item::Block(block);
                true
            }
            None => {
                self.root.push(block);
                false
            }
        }
    }

    pub fn remove(&mut self, kind: &str, name: &str) -> bool {
        match self.position(kind, name) {
            Some(i) => {
                self.root.items.remove(i);
                true
            }
            None => false,
        }
    }

    /// Reads and validates the whole document. Every problem is returned, with file-style line numbers where the
    /// section came from text.
    pub fn build(&self) -> Result<Bundle, Vec<String>> {
        let mut errors = Vec::new();
        let manifest = read_manifest(&self.root, &mut errors);
        let content = read_content(&self.root, &mut errors);
        if !errors.is_empty() {
            return Err(errors);
        }
        crate::validate(&content)?;
        Ok(Bundle {
            manifest,
            content,
            verified: false,
        })
    }

    /// Canonical bundle text for the document (it must build). `pack` adds the content hash and sim version.
    pub fn to_text(&self, pack: bool) -> Result<String, Vec<String>> {
        let bundle = self.build()?;
        let m = &bundle.manifest;
        Ok(crate::bundle::to_text(
            &bundle.content,
            &m.name,
            &m.author,
            &m.description,
            pack,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc() -> Doc {
        Doc::from_content(&Content::placeholder(), "Test", "me", "d")
    }

    #[test]
    fn a_document_from_content_builds_back_to_the_same_content() {
        let b = doc().build().unwrap();
        assert_eq!(b.content, Content::placeholder());
        assert_eq!(b.manifest.name, "Test");
    }

    #[test]
    fn sections_can_be_read_replaced_added_and_removed() {
        let mut d = doc();
        let kinds: Vec<String> = d.sections().iter().map(|s| s.0.clone()).collect();
        assert_eq!(kinds[0], "bundle");
        assert!(d.get("fighter", "duelist").is_some());
        // Edit one value of a fighter and put it back.
        let mut f = d.get("fighter", "duelist").unwrap().clone();
        for item in &mut f.items {
            if let Item::Field { name, value, .. } = item {
                if name == "weight" {
                    *value = "95".to_string();
                }
            }
        }
        assert!(d.put(f), "replaced, not added");
        assert_eq!(
            d.build().unwrap().content.fighters[0].weight,
            sim_core::Fx::from_int(95)
        );
        // A brand-new fighter that inherits.
        let mut n = Block::new("fighter", Some("sprinter"));
        n.field("inherit", "brawler");
        n.field("walk_speed", "3");
        assert!(!d.put(n));
        assert_eq!(d.build().unwrap().content.fighters.len(), 3);
        assert!(d.remove("fighter", "sprinter"));
        assert_eq!(d.build().unwrap().content.fighters.len(), 2);
        assert!(!d.remove("fighter", "sprinter"));
    }

    #[test]
    fn a_bad_edit_is_reported_and_leaves_a_document_that_can_be_fixed() {
        let mut d = doc();
        let mut f = d.get("fighter", "duelist").unwrap().clone();
        for item in &mut f.items {
            if let Item::Field { name, value, .. } = item {
                if name == "gravity" {
                    *value = "0".to_string();
                }
                if name == "walk_speed" {
                    *value = "fast".to_string();
                }
            }
        }
        d.put(f);
        let errors = d.build().unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("walk_speed")),
            "{errors:?}"
        );
        // Validation errors (gravity zero) show up once the values parse.
        let mut f = d.get("fighter", "duelist").unwrap().clone();
        for item in &mut f.items {
            if let Item::Field { name, value, .. } = item {
                if name == "walk_speed" {
                    *value = "1".to_string();
                }
            }
        }
        d.put(f);
        let errors = d.build().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("gravity")), "{errors:?}");
    }

    #[test]
    fn the_stage_and_ruleset_are_single_sections_whatever_they_are_called() {
        let mut d = doc();
        let mut stage = d.get("stage", "").unwrap().clone();
        stage.name = Some("renamed".to_string());
        assert!(d.put(stage), "the stage is replaced even under a new name");
        assert_eq!(d.build().unwrap().content.names.stage, "renamed");
        assert_eq!(d.sections().iter().filter(|s| s.0 == "stage").count(), 1);
    }

    #[test]
    fn text_round_trips_through_a_document() {
        let text = doc().to_text(true).unwrap();
        let loaded = crate::load(&text).unwrap();
        assert!(loaded.verified);
        assert_eq!(loaded.content, Content::placeholder());
        let again = Doc::parse(&text).unwrap().to_text(true).unwrap();
        assert_eq!(text, again);
    }
}
