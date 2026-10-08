//! `ContentEditor`: the editors' view of a content document (plan 7.6).
//!
//! The editors never interpret content themselves. They read a section as a tree of dictionaries, change values (all
//! values are text, exactly as they would be in the file), put the section back, and this class re-reads and validates
//! the whole document with the same code the game uses to load a bundle. What it answers is therefore always what the
//! game would do with the file. Every edit can be undone.

use godot::prelude::*;
use sim_content::doc::Doc;
use sim_content::tree::{Block, Item};
use sim_core::moves::MoveId;
use sim_core::Content;

const MAX_UNDO: usize = 100;

#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct ContentEditor {
    base: Base<RefCounted>,
    doc: Doc,
    undo: Vec<Doc>,
    redo: Vec<Doc>,
    errors: Vec<String>,
    path: String,
}

fn block_to_dict(b: &Block) -> VarDictionary {
    let mut items = VarArray::new();
    for item in &b.items {
        let d = match item {
            Item::Field { name, value, .. } => {
                let mut d = VarDictionary::new();
                d.set("t", "field");
                d.set("name", name.as_str());
                d.set("value", value.as_str());
                d
            }
            Item::Block(inner) => {
                let mut d = block_to_dict(inner);
                d.set("t", "block");
                d
            }
            Item::Raw { kind, text, .. } => {
                let mut d = VarDictionary::new();
                d.set("t", "raw");
                d.set("kind", kind.as_str());
                d.set("text", text.as_str());
                d
            }
        };
        items.push(&d.to_variant());
    }
    let mut d = VarDictionary::new();
    d.set("kind", b.kind.as_str());
    d.set("name", b.name.clone().unwrap_or_default().as_str());
    d.set("items", &items.to_variant());
    d
}

fn text_of(d: &VarDictionary, key: &str) -> String {
    d.get(key)
        .and_then(|v| v.try_to::<GString>().ok())
        .map(|g| g.to_string())
        .unwrap_or_default()
}

fn dict_to_block(d: &VarDictionary) -> Block {
    let name = text_of(d, "name");
    let mut b = Block::new(
        &text_of(d, "kind"),
        (!name.is_empty()).then_some(name.as_str()),
    );
    if let Some(items) = d.get("items").and_then(|v| v.try_to::<VarArray>().ok()) {
        for v in items.iter_shared() {
            let Ok(item) = v.try_to::<VarDictionary>() else {
                continue;
            };
            match text_of(&item, "t").as_str() {
                "field" => b.field(&text_of(&item, "name"), text_of(&item, "value")),
                "raw" => b.items.push(Item::Raw {
                    kind: text_of(&item, "kind"),
                    text: text_of(&item, "text"),
                    line: 0,
                }),
                _ => b.push(dict_to_block(&item)),
            }
        }
    }
    b
}

fn packed(lines: &[String]) -> PackedStringArray {
    let v: Vec<GString> = lines.iter().map(|l| GString::from(l.as_str())).collect();
    PackedStringArray::from(v.as_slice())
}

#[godot_api]
impl IRefCounted for ContentEditor {
    fn init(base: Base<RefCounted>) -> Self {
        ContentEditor {
            base,
            doc: Doc::from_content(&Content::placeholder(), "My Pack", "", ""),
            undo: Vec::new(),
            redo: Vec::new(),
            errors: Vec::new(),
            path: String::new(),
        }
    }
}

#[godot_api]
impl ContentEditor {
    fn refresh(&mut self) {
        self.errors = match self.doc.build() {
            Ok(_) => Vec::new(),
            Err(e) => e,
        };
    }

    fn checkpoint(&mut self) {
        self.undo.push(self.doc.clone());
        if self.undo.len() > MAX_UNDO {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Starts from the built-in roster.
    #[func]
    fn new_from_builtin(&mut self, name: GString) {
        self.doc = Doc::from_content(&Content::placeholder(), &name.to_string(), "", "");
        self.undo.clear();
        self.redo.clear();
        self.path.clear();
        self.refresh();
    }

    /// Opens bundle text. Returns an error message (empty on success). The document may still be invalid: see `errors`.
    #[func]
    fn open_text(&mut self, text: GString) -> GString {
        match Doc::parse(&text.to_string()) {
            Ok(d) => {
                self.doc = d;
                self.undo.clear();
                self.redo.clear();
                self.refresh();
                GString::new()
            }
            Err(e) => GString::from(e.as_str()),
        }
    }

    #[func]
    fn open_file(&mut self, path: GString) -> GString {
        let p = path.to_string();
        match std::fs::read_to_string(&p) {
            Ok(t) => {
                let r = self.open_text(GString::from(t.as_str()));
                if r.is_empty() {
                    self.path = p;
                }
                r
            }
            Err(e) => GString::from(format!("cannot read {p}: {e}").as_str()),
        }
    }

    /// Where the document was opened from or last saved to (empty for a new one).
    #[func]
    fn path(&self) -> GString {
        GString::from(self.path.as_str())
    }

    /// Problems with the document as it is now (empty if it is valid and could be played).
    #[func]
    fn errors(&self) -> PackedStringArray {
        packed(&self.errors)
    }

    #[func]
    fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Every top-level section as `{kind, name}`.
    #[func]
    fn sections(&self) -> VarArray {
        let mut out = VarArray::new();
        for (kind, name) in self.doc.sections() {
            let mut d = VarDictionary::new();
            d.set("kind", kind.as_str());
            d.set("name", name.as_str());
            out.push(&d.to_variant());
        }
        out
    }

    /// One section as a tree of dictionaries (`kind`, `name`, `items`; items are `{t: field|block|raw, ...}`), or an
    /// empty dictionary if there is none.
    #[func]
    fn get_section(&self, kind: GString, name: GString) -> VarDictionary {
        match self.doc.get(&kind.to_string(), &name.to_string()) {
            Some(b) => block_to_dict(b),
            None => VarDictionary::new(),
        }
    }

    /// Puts a section back (replacing the one of that kind and name, or adding it) and re-checks the document.
    /// Returns the problems found (empty if the document is valid now). The edit stays either way; undo takes it back.
    #[func]
    fn put_section(&mut self, section: VarDictionary) -> PackedStringArray {
        self.checkpoint();
        self.doc.put(dict_to_block(&section));
        self.refresh();
        packed(&self.errors)
    }

    #[func]
    fn remove_section(&mut self, kind: GString, name: GString) -> PackedStringArray {
        self.checkpoint();
        self.doc.remove(&kind.to_string(), &name.to_string());
        self.refresh();
        packed(&self.errors)
    }

    #[func]
    fn undo(&mut self) -> bool {
        match self.undo.pop() {
            Some(d) => {
                self.redo.push(std::mem::replace(&mut self.doc, d));
                self.refresh();
                true
            }
            None => false,
        }
    }

    #[func]
    fn redo(&mut self) -> bool {
        match self.redo.pop() {
            Some(d) => {
                self.undo.push(std::mem::replace(&mut self.doc, d));
                self.refresh();
                true
            }
            None => false,
        }
    }

    #[func]
    fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    #[func]
    fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// The canonical bundle text (with the content hash if `pack`), or an empty string if the document is invalid.
    #[func]
    fn text(&self, pack: bool) -> GString {
        match self.doc.to_text(pack) {
            Ok(t) => GString::from(t.as_str()),
            Err(_) => GString::new(),
        }
    }

    /// Saves the bundle. Returns an error message (empty on success); an invalid document cannot be saved.
    #[func]
    fn save(&mut self, path: GString, pack: bool) -> GString {
        let text = match self.doc.to_text(pack) {
            Ok(t) => t,
            Err(e) => {
                return GString::from(
                    format!("not saved, the content has problems: {}", e.join("; ")).as_str(),
                )
            }
        };
        let p = path.to_string();
        match std::fs::write(&p, text) {
            Ok(()) => {
                self.path = p;
                GString::new()
            }
            Err(e) => GString::from(format!("cannot write {p}: {e}").as_str()),
        }
    }

    /// The manifest fields (`name`, `author`, `description`) for the bundle.
    #[func]
    fn manifest(&self) -> VarDictionary {
        match self.doc.get("bundle", "") {
            Some(b) => block_to_dict(b),
            None => VarDictionary::new(),
        }
    }

    /// Names of every fighter parameter a fighter section takes, in file order.
    #[func]
    fn param_names(&self) -> PackedStringArray {
        let v: Vec<GString> = sim_content::format::PARAM_NAMES
            .iter()
            .map(|n| GString::from(*n))
            .collect();
        PackedStringArray::from(v.as_slice())
    }

    #[func]
    fn rule_names(&self) -> PackedStringArray {
        let v: Vec<GString> = sim_content::format::RULE_NAMES
            .iter()
            .map(|n| GString::from(*n))
            .collect();
        PackedStringArray::from(v.as_slice())
    }

    /// Every move slot's file name, in slot order.
    #[func]
    fn move_keys(&self) -> PackedStringArray {
        let v: Vec<GString> = MoveId::all().map(|m| GString::from(m.key())).collect();
        PackedStringArray::from(v.as_slice())
    }
}
