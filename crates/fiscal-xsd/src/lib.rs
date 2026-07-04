//! Fast, opt-in XSD validation for Brazilian fiscal XML (NF-e, MDF-e, …).
//!
//! # Design — why it's fast
//!
//! The expensive part of XSD validation is **compiling** the schema (parsing the
//! `.xsd` + its `<xs:include>` graph into libxml2's internal automaton). Once
//! compiled, **validating** a document is cheap — O(document size), typically
//! sub-millisecond for a ~10 KB fiscal document. This crate therefore compiles
//! each schema **at most once per worker thread** ([`thread_local`] cache) and
//! reuses it for every subsequent validation. The hot path is just the parse of
//! the (small) incoming XML, so validation cost is negligible next to a SEFAZ
//! round-trip.
//!
//! # Design — why it's opt-in
//!
//! XSD validation pulls in `libxml` (a C dependency). This crate is **separate**
//! from `fiscal-core`/the builders, so projects that don't want it simply don't
//! depend on it — zero cost, `libxml` is never linked. The actual schema
//! bundles are behind cargo features (e.g. `mdfe`), so the binary only embeds
//! the schemas it uses.
//!
//! # Thread-safety
//!
//! libxml2's compiled schema / validation context wrap raw C pointers and are
//! `!Send` + `!Sync`, so they cannot be shared across threads via a global.
//! Instead each thread keeps its own compiled context, keyed by schema id.
//!
//! # Example
//!
//! ```
//! use fiscal_xsd::XsdSchema;
//!
//! static FILES: &[(&str, &[u8])] = &[(
//!     "root.xsd",
//!     br#"<?xml version="1.0"?>
//!         <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
//!           <xs:element name="a" type="xs:string"/>
//!         </xs:schema>"#,
//! )];
//! static SCHEMA: XsdSchema = XsdSchema::new("demo", FILES, "root.xsd");
//!
//! assert!(SCHEMA.validate("<a>ok</a>").is_ok());
//! assert!(SCHEMA.validate("<b/>").is_err());
//! ```

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use libxml::parser::Parser;
use libxml::schemas::{SchemaParserContext, SchemaValidationContext};
use tempfile::TempDir;

/// A schema bundle: the embedded `.xsd` files plus the root file to validate
/// against. Construct as a `static` (it is `const`-constructible) so the same
/// instance — and thus the same per-thread compiled context — is reused.
///
/// `files` carries every file referenced by the `<xs:include>` graph, keyed by
/// the **exact filename** used in the `schemaLocation` attributes (libxml2
/// resolves includes by filename on disk).
pub struct XsdSchema {
    id: &'static str,
    files: &'static [(&'static str, &'static [u8])],
    root: &'static str,
}

impl XsdSchema {
    /// Build a schema bundle. `id` must be process-unique (it keys the temp-dir
    /// and per-thread context caches).
    pub const fn new(
        id: &'static str,
        files: &'static [(&'static str, &'static [u8])],
        root: &'static str,
    ) -> Self {
        Self { id, files, root }
    }

    /// Validate an XML string against this schema.
    ///
    /// Returns `Ok(())` when valid, or `Err(messages)` listing the schema
    /// violations (one per failing element, as reported by libxml2 — e.g.
    /// `"Element '…xFant': … minLength"`). A malformed XML returns a single
    /// "XML mal-formado" message.
    ///
    /// The schema is compiled on first use on the calling thread and cached;
    /// later calls only parse the incoming document.
    pub fn validate(&self, xml: &str) -> Result<(), Vec<String>> {
        let doc = Parser::default()
            .parse_string(xml)
            .map_err(|e| vec![format!("XML mal-formado: {e}")])?;

        CTXS.with(|cell| {
            let mut map = cell.borrow_mut();
            if !map.contains_key(self.id) {
                let ctx = compile(self)?;
                map.insert(self.id, ctx);
            }
            let ctx = map.get_mut(self.id).expect("ctx inserted above");
            ctx.validate_document(&doc)
                .map_err(|errors| collect(&errors))
        })
    }
}

thread_local! {
    /// Per-thread compiled validation contexts, keyed by schema id.
    static CTXS: RefCell<HashMap<&'static str, SchemaValidationContext>> =
        RefCell::new(HashMap::new());
}

/// Compile a schema bundle into a fresh validation context (materializes the
/// files to a temp dir first so `<xs:include>` resolves).
fn compile(schema: &XsdSchema) -> Result<SchemaValidationContext, Vec<String>> {
    let root = materialize(schema)?;
    let root_str = root.to_string_lossy().into_owned();
    let mut parser = SchemaParserContext::from_file(&root_str);
    SchemaValidationContext::from_parser(&mut parser).map_err(|errors| collect(&errors))
}

/// Materialize the embedded schema files into a secure temp directory (once per
/// schema id, process-wide). Uses [`tempfile::TempDir`] so the directory gets a
/// random name and restricted permissions (0700 on Unix), which prevents both
/// symlink attacks and TOCTOU races that a predictable path would allow.
///
/// The [`TempDir`] handle is kept alive in the global registry for the lifetime
/// of the process; when the process exits the OS reclaims it automatically.
fn materialize(schema: &XsdSchema) -> Result<PathBuf, Vec<String>> {
    // We keep (TempDir, root_path) so the TempDir handle is not dropped
    // (which would delete the directory while libxml2 still needs the files).
    static DIRS: OnceLock<Mutex<HashMap<&'static str, (TempDir, PathBuf)>>> = OnceLock::new();
    let dirs = DIRS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = dirs.lock().expect("xsd dir registry poisoned");

    if let Some((_, root)) = guard.get(schema.id) {
        return Ok(root.clone());
    }

    let tmp = tempfile::Builder::new()
        .prefix("fiscal-xsd-")
        .tempdir()
        .map_err(|e| vec![format!("criar temp dir: {e}")])?;

    for (name, bytes) in schema.files {
        let mut f = std::fs::File::create(tmp.path().join(name))
            .map_err(|e| vec![format!("escrever {name}: {e}")])?;
        f.write_all(bytes)
            .map_err(|e| vec![format!("escrever {name}: {e}")])?;
    }

    let root = tmp.path().join(schema.root);
    guard.insert(schema.id, (tmp, root.clone()));
    Ok(root)
}

/// Flatten libxml2 structured errors into trimmed, non-empty messages.
fn collect(errors: &[libxml::error::StructuredError]) -> Vec<String> {
    let msgs: Vec<String> = errors
        .iter()
        .filter_map(|e| e.message.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if msgs.is_empty() {
        vec!["documento não satisfaz o schema XSD".to_string()]
    } else {
        msgs
    }
}

/// Embedded SEFAZ schema bundles (feature-gated).
pub mod schemas;

#[cfg(test)]
mod tests {
    use super::*;

    static FILES: &[(&str, &[u8])] = &[(
        "root.xsd",
        br#"<?xml version="1.0"?>
            <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
              <xs:element name="a" type="xs:string"/>
            </xs:schema>"#,
    )];
    static SCHEMA: XsdSchema = XsdSchema::new("test-demo", FILES, "root.xsd");

    #[test]
    fn valid_document_passes() {
        assert!(SCHEMA.validate("<a>ok</a>").is_ok());
    }

    #[test]
    fn invalid_element_fails() {
        assert!(SCHEMA.validate("<b/>").is_err());
    }

    #[test]
    fn broken_input_fails() {
        // Whether libxml flags it as malformed or schema-invalid, broken input
        // must surface as Err with a non-empty message.
        let err = SCHEMA.validate("<a><b></a>").unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn second_call_reuses_cached_ctx() {
        // Two calls on the same thread — the second must hit the cached context
        // (no panic, same result).
        assert!(SCHEMA.validate("<a>1</a>").is_ok());
        assert!(SCHEMA.validate("<a>2</a>").is_ok());
    }
}
