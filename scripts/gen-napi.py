#!/usr/bin/env python3
"""Generate crates/fiscal-napi/src/ from the Rust public API using tree-sitter.

Usage:
    python scripts/gen-napi.py          # preview stats
    python scripts/gen-napi.py --write  # overwrite crates/fiscal-napi/src/

Philosophy: NEVER assume anything about the API. Every function signature,
parameter type, and return type is read directly from the AST. When the
Rust API changes, re-run this script — no script changes needed.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass, field
from pathlib import Path

import tree_sitter as ts
import tree_sitter_rust as tsr

ROOT = Path(__file__).resolve().parent.parent
NAPI_SRC = ROOT / "crates" / "fiscal-napi" / "src"

PARSER = ts.Parser(ts.Language(tsr.language()))


def parse_file(path: Path) -> ts.Node:
    return PARSER.parse(path.read_bytes()).root_node


def txt(node: ts.Node) -> str:
    return node.text.decode("utf-8")


# ── Data types ───────────────────────────────────────────────────────────────


@dataclass
class Param:
    name: str
    rust_type: str


@dataclass
class FnSig:
    name: str
    is_async: bool
    params: list[Param]
    return_type: str | None
    doc: str = ""
    has_self: bool = False


@dataclass
class ImplBlock:
    type_name: str
    methods: list[FnSig] = field(default_factory=list)


# ── AST extraction (pure, no assumptions) ────────────────────────────────────


def extract_doc(node: ts.Node) -> str:
    lines = []
    prev = node.prev_sibling
    while prev and prev.type == "line_comment":
        t = txt(prev)
        if t.startswith("///"):
            lines.insert(0, t[3:].strip())
        else:
            break
        prev = prev.prev_sibling
    return "\n".join(lines)


def extract_fn(node: ts.Node) -> FnSig | None:
    vis_node = next((c for c in node.children if c.type == "visibility_modifier"), None)
    if not vis_node:
        return None
    # Skip pub(crate), pub(super), etc. — only plain `pub`
    vis_text = txt(vis_node)
    if vis_text != "pub":
        return None

    name_node = node.child_by_field_name("name")
    if not name_node:
        return None

    params_node = node.child_by_field_name("parameters")
    params = []
    has_self = False
    if params_node:
        for c in params_node.children:
            if c.type == "self_parameter":
                has_self = True
            elif c.type == "parameter":
                pn = c.child_by_field_name("pattern")
                pt = c.child_by_field_name("type")
                if pn and pt:
                    params.append(Param(txt(pn), txt(pt)))

    is_async = b"async" in node.text.split(b"fn")[0]

    ret_node = node.child_by_field_name("return_type")
    ret = txt(ret_node) if ret_node else None

    return FnSig(
        name=txt(name_node),
        is_async=is_async,
        params=params,
        return_type=ret,
        doc=extract_doc(node),
        has_self=has_self,
    )


def extract_pub_fns(root: ts.Node) -> list[FnSig]:
    return [f for n in root.children if n.type == "function_item" for f in [extract_fn(n)] if f]


def extract_impl_methods(root: ts.Node, type_name: str) -> list[FnSig]:
    methods = []
    for node in root.children:
        if node.type != "impl_item":
            continue
        tn = node.child_by_field_name("type")
        if not tn or txt(tn) != type_name:
            continue
        body = node.child_by_field_name("body")
        if not body:
            continue
        for child in body.children:
            if child.type == "function_item":
                fn = extract_fn(child)
                if fn:
                    methods.append(fn)
    return methods


# Enums that are accepted as strings from JS and parsed at runtime.
# Maps enum_name → local variable name used in generated code.
PARSEABLE_ENUMS = {"SefazEnvironment", "SignatureAlgorithm", "SefazService"}


# ── Type mapping (generic, based purely on the Rust type string) ─────────────


def map_param(p: Param) -> tuple[str, str]:
    """Return (napi_param_decl, call_expr) for a parameter.

    This is the ONLY place that knows how to convert Rust types to napi types.
    It must handle every type it encounters without hardcoding function names.
    """
    t = p.rust_type.strip()

    # &[u8] → Buffer
    if t == "&[u8]":
        return f"{p.name}: Buffer", f"&{p.name}"

    # &str / &String → String (owned, passed as &)
    if t in ("&str", "&String"):
        return f"{p.name}: String", f"&{p.name}"

    # String (owned) → String
    if t == "String":
        return f"{p.name}: String", p.name

    # Known enums → String (parsed at runtime via parse_<snake_name>)
    if t in PARSEABLE_ENUMS:
        local_var = _camel_to_snake(t)
        return f"{p.name}: String", local_var

    # Numeric primitives
    if t in ("u32", "i32", "u16", "i16"):
        return f"{p.name}: {t}", p.name
    if t in ("u8", "i8"):
        return f"{p.name}: u32", f"{p.name} as {t}"

    # bool
    if t == "bool":
        return f"{p.name}: bool", p.name

    # f64 / f32
    if t in ("f64", "f32"):
        return f"{p.name}: {t}", p.name

    # Option<T> — parse via AST
    type_node = parse_type_node(t)
    if type_node and type_node.type == "generic_type":
        base = type_node.children[0]
        if base.type == "type_identifier" and txt(base) == "Option":
            type_args = next((c for c in type_node.children if c.type == "type_arguments"), None)
            if type_args:
                inner = next((c for c in type_args.children if c.type not in ("<", ">", ",")), None)
                if inner:
                    inner_text = txt(inner)
                    # Option<&str>
                    if inner.type == "reference_type" and inner_text == "&str":
                        return f"{p.name}: Option<String>", f"{p.name}.as_deref()"
                    # Option<primitive>
                    if inner.type == "primitive_type":
                        return f"{p.name}: Option<{inner_text}>", p.name
                    # Option<String>
                    if inner.type == "type_identifier" and inner_text == "String":
                        return f"{p.name}: Option<String>", p.name
                    # Option of numeric type identifiers (u32, etc. parsed as type_identifier)
                    if inner.type == "type_identifier" and inner_text in ("u32", "i32", "u16", "i16", "u8", "i8", "f64", "f32"):
                        return f"{p.name}: Option<{inner_text}>", p.name

    # Any other known enum not in PARSEABLE_ENUMS — skip
    # (already handled above)

    # Complex struct types passed by value — need serde
    # (EpecData, Vec<EventItem>, etc.)
    # For now, skip these methods (they need manual handling or serde)
    return None, None


def parse_type_node(type_text: str) -> ts.Node | None:
    """Parse a type string into a tree-sitter type node."""
    tree = PARSER.parse(f"type _ = {type_text};".encode())
    alias = tree.root_node.children[0] if tree.root_node.children else None
    if not alias:
        return None
    # The type value is the child after '=', which is the 4th child
    # Layout: type(0) _(1) =(2) TYPE(3) ;(4)
    children = [c for c in alias.children]
    eq_idx = next((i for i, c in enumerate(children) if txt(c) == "="), None)
    if eq_idx is not None and eq_idx + 1 < len(children):
        candidate = children[eq_idx + 1]
        if candidate.type != ";":
            return candidate
    return None


def parse_return_type_node(ret_text: str) -> ts.Node | None:
    """Parse a return type string into a tree-sitter node."""
    tree = PARSER.parse(f"fn _() -> {ret_text} {{}}".encode())
    fn_node = tree.root_node.children[0]
    return fn_node.child_by_field_name("return_type")


def get_ok_type_name(ret_node: ts.Node) -> str | None:
    """Extract the Ok type name from a Result<T, E> AST node.

    Returns the *leaf* type identifier name (e.g. "String", "Value",
    "Map", "AuthorizationResponse", "bool", "Vec").
    """
    if ret_node.type != "generic_type":
        return None

    # Check it's Result<...>
    first = ret_node.children[0]
    if first.type != "type_identifier" or txt(first) != "Result":
        return None

    # Get type_arguments node
    type_args = next((c for c in ret_node.children if c.type == "type_arguments"), None)
    if not type_args:
        return None

    # First non-punctuation child of type_arguments is the Ok type
    ok_node = None
    for c in type_args.children:
        if c.type not in ("<", ">", ","):
            ok_node = c
            break

    if not ok_node:
        return None

    return ok_node


def map_return(ret: str | None) -> tuple[str, str]:
    """Return (napi_return_type, result_handling) using AST parsing.

    result_handling is either "direct" (return as-is) or "serde" (use to_json).
    """
    if not ret:
        return "napi::Result<()>", "direct"

    ret_node = parse_return_type_node(ret)
    if not ret_node:
        return "napi::Result<String>", "direct"

    ok_node = get_ok_type_name(ret_node)
    if not ok_node:
        # Not a Result<T, E> — might be a plain type
        return "napi::Result<String>", "direct"

    ok_text = txt(ok_node)
    ok_type = ok_node.type

    # Primitive types → direct
    if ok_type == "primitive_type":  # bool, i32, etc.
        return f"napi::Result<{ok_text}>", "direct"

    # Simple type_identifier
    if ok_type == "type_identifier":
        if ok_text == "String":
            return "napi::Result<String>", "direct"
        # Any other named type → serde serialize
        return "napi::Result<serde_json::Value>", "serde"

    # scoped_type_identifier (e.g. serde_json::Value)
    if ok_type == "scoped_type_identifier":
        # Get the leaf type name
        leaf = ok_node.children[-1]  # last child is the type name
        if txt(leaf) == "Value":
            return "napi::Result<serde_json::Value>", "direct"
        # Any other scoped type → serde
        return "napi::Result<serde_json::Value>", "serde"

    # generic_type (Vec<String>, Map<K,V>, HashMap<K,V>, etc.)
    if ok_type == "generic_type":
        # Get the base type name
        base = ok_node.children[0]
        base_name = txt(base) if base.type == "type_identifier" else ""

        if base_name == "Vec":
            # Check inner type
            inner_args = next((c for c in ok_node.children if c.type == "type_arguments"), None)
            if inner_args:
                inner = next((c for c in inner_args.children if c.type not in ("<", ">", ",")), None)
                if inner and txt(inner) == "String":
                    return "napi::Result<Vec<String>>", "direct"
            # Vec of anything else → serde
            return "napi::Result<serde_json::Value>", "serde"

        # Map, HashMap, etc. → always serde
        return "napi::Result<serde_json::Value>", "serde"

    # Fallback
    return "napi::Result<serde_json::Value>", "serde"


# ── Enum AST helpers ─────────────────────────────────────────────────────────


def extract_enum_variants(path: Path, enum_name: str) -> list[tuple[str, str | None]]:
    """Extract (variant_name, discriminant_value) from an enum in a file.

    Returns e.g. [("Sha1", None), ("Sha256", None)] or
    [("Production", "1"), ("Homologation", "2")].
    """
    root = parse_file(path)
    for node in root.children:
        if node.type != "enum_item":
            continue
        name_node = node.child_by_field_name("name")
        if not name_node or txt(name_node) != enum_name:
            continue
        body = node.child_by_field_name("body")
        if not body:
            continue
        variants = []
        for c in body.children:
            if c.type == "enum_variant":
                vname = txt(c.child_by_field_name("name"))
                # Check for discriminant (= value)
                disc = None
                for vc in c.children:
                    if vc.type == "integer_literal":
                        disc = txt(vc)
                variants.append((vname, disc))
        return variants
    return []


def gen_enum_parser(enum_name: str, rust_path: str, source_file: Path) -> str:
    """Generate a parse_<name> function that maps strings to enum variants.

    Reads variants from the AST. If variants have discriminant values,
    those are also accepted as string aliases.
    """
    variants = extract_enum_variants(source_file, enum_name)
    fn_name = f"parse_{_camel_to_snake(enum_name)}"

    lines = []
    lines.append(f"\nfn {fn_name}(s: &str) -> napi::Result<{rust_path}> {{\n")
    lines.append(f"    match s.to_lowercase().as_str() {{\n")
    for vname, disc in variants:
        # camelCase version of the variant name for JS
        js_name = _pascal_to_lower_camel(vname)
        match_arms = f'"{js_name}"'
        if disc:
            match_arms += f' | "{disc}"'
        lines.append(f"        {match_arms} => Ok({rust_path}::{vname}),\n")
    lines.append(f'        _ => Err(napi::Error::from_reason(format!(\n')
    lines.append(f'            "Invalid {enum_name}: \\"{{s}}\\""\n')
    lines.append(f"        ))),\n")
    lines.append(f"    }}\n")
    lines.append(f"}}\n")
    return "".join(lines)


def _camel_to_snake(name: str) -> str:
    """Convert CamelCase to snake_case."""
    result = []
    for i, c in enumerate(name):
        if c.isupper() and i > 0:
            result.append("_")
        result.append(c.lower())
    return "".join(result)


def _collect_type_idents(node: ts.Node, result: set[str]):
    """Recursively collect all type_identifier names from a type AST node."""
    if node.type == "type_identifier":
        result.add(txt(node))
    for c in node.children:
        _collect_type_idents(c, result)


def rewrite_crate_paths(type_text: str) -> str:
    """Rewrite `crate::` paths to `fiscal_core::` using AST.

    Parses the type, finds all scoped identifiers starting with `crate`,
    and replaces them with `fiscal_core`.
    """
    node = parse_type_node(type_text)
    if not node:
        return type_text

    # Collect all byte ranges where `crate` appears as a path root
    result = bytearray(type_text.encode())
    replacements = []

    def find_crate_roots(n: ts.Node):
        if n.type == "crate" and n.parent and n.parent.type == "scoped_type_identifier":
            replacements.append((n.start_byte, n.end_byte))
        for c in n.children:
            find_crate_roots(c)

    # We need byte offsets relative to the type text, not the wrapper.
    # parse_type_node wraps as "type _ = <type>;" — offset is len("type _ = ")
    offset = len("type _ = ")

    def find_crate_in_wrapper(n: ts.Node):
        if n.type == "crate":
            parent = n.parent
            if parent and parent.type in ("scoped_type_identifier", "scoped_identifier"):
                # byte range relative to the type text
                start = n.start_byte - offset
                end = n.end_byte - offset
                if 0 <= start < len(type_text):
                    replacements.append((start, end))
        for c in n.children:
            find_crate_in_wrapper(c)

    # Re-parse the full wrapper to get correct byte offsets
    wrapper = f"type _ = {type_text};"
    tree = PARSER.parse(wrapper.encode())
    find_crate_in_wrapper(tree.root_node)

    if not replacements:
        return type_text

    # Apply replacements in reverse order to maintain offsets
    text_bytes = bytearray(type_text.encode())
    for start, end in sorted(replacements, reverse=True):
        text_bytes[start:end] = b"fiscal_core"

    return text_bytes.decode()


def _pascal_to_lower_camel(name: str) -> str:
    """Convert PascalCase to lowerCamelCase (first char lowercase)."""
    if not name:
        return name
    return name[0].lower() + name[1:]


# ── Skipped tracking ────────────────────────────────────────────────────────

skipped: list[tuple[str, str]] = []  # (fn_name, reason)


# ── Code generators ─────────────────────────────────────────────────────────


def doc_comment(doc: str, indent: str = "") -> str:
    if not doc:
        return ""
    lines = doc.split("\n")
    return "".join(f"{indent}/// {l}\n" if l else f"{indent}///\n" for l in lines)


def gen_fn_wrapper(fn: FnSig, mod_path: str, indent: str = "") -> str | None:
    """Generate a #[napi] wrapper for a standalone pub fn.

    Returns None if the function has unsupported parameter types.
    """
    ret_type, ret_handling = map_return(fn.return_type)

    napi_params = []
    call_args = []
    needs_env_parse = False

    enum_parses = []  # lines like "let sefaz_environment = parse_sefaz_environment(&param)?;"

    for p in fn.params:
        decl, expr = map_param(p)
        if decl is None:
            return None  # unsupported param type, skip this fn
        napi_params.append(decl)
        call_args.append(expr)
        if p.rust_type.strip() in PARSEABLE_ENUMS:
            local_var = _camel_to_snake(p.rust_type.strip())
            parse_fn = f"parse_{local_var}"
            enum_parses.append(f"{indent}    let {local_var} = {parse_fn}(&{p.name})?;\n")

    params_str = ", ".join(napi_params)
    args_str = ", ".join(call_args)

    lines = []
    lines.append(doc_comment(fn.doc, indent))

    # napi attribute
    if "serde_json::Value" in ret_type and ret_handling == "direct":
        lines.append(f'{indent}#[napi(ts_return_type = "Record<string, unknown>")]\n')
    else:
        lines.append(f"{indent}#[napi]\n")

    lines.append(f"{indent}pub fn {fn.name}({params_str}) -> {ret_type} {{\n")

    for ep in enum_parses:
        lines.append(ep)

    if ret_handling == "serde":
        lines.append(f"{indent}    let result = {mod_path}::{fn.name}({args_str})\n")
        lines.append(f"{indent}        .map_err(|e| napi::Error::from_reason(e.to_string()))?;\n")
        lines.append(f"{indent}    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))\n")
    else:
        lines.append(f"{indent}    {mod_path}::{fn.name}({args_str})\n")
        lines.append(f"{indent}        .map_err(|e| napi::Error::from_reason(e.to_string()))\n")

    lines.append(f"{indent}}}\n\n")
    return "".join(lines)


def gen_method_wrapper(fn: FnSig, indent: str = "    ") -> str | None:
    """Generate a napi method wrapper for an impl method.

    Returns None if the method has unsupported parameter types.
    """
    ret_type, ret_handling = map_return(fn.return_type)

    napi_params = ["&self"]
    call_args = []
    enum_parses = []

    for p in fn.params:
        decl, expr = map_param(p)
        if decl is None:
            return None  # unsupported param type, skip
        napi_params.append(decl)
        call_args.append(expr)
        if p.rust_type.strip() in PARSEABLE_ENUMS:
            local_var = _camel_to_snake(p.rust_type.strip())
            parse_fn = f"parse_{local_var}"
            enum_parses.append(f"{indent}    let {local_var} = {parse_fn}(&{p.name})?;\n")

    params_str = ",\n{i}    ".format(i=indent).join(napi_params)
    args_str = ", ".join(call_args)

    lines = []
    lines.append(doc_comment(fn.doc, indent))

    if ret_handling == "serde":
        lines.append(f'{indent}#[napi(ts_return_type = "Promise<Record<string, unknown>>")]\n')
    else:
        lines.append(f"{indent}#[napi]\n")

    lines.append(f"{indent}pub async fn {fn.name}(\n{indent}    {params_str},\n{indent}) -> {ret_type} {{\n")

    for ep in enum_parses:
        lines.append(ep)

    if ret_handling == "serde":
        lines.append(f"{indent}    let resp = self\n")
        lines.append(f"{indent}        .inner\n")
        lines.append(f"{indent}        .{fn.name}({args_str})\n")
        lines.append(f"{indent}        .await\n")
        lines.append(f"{indent}        .map_err(to_napi)?;\n")
        lines.append(f"{indent}    to_json(&resp)\n")
    else:
        lines.append(f"{indent}    self.inner\n")
        lines.append(f"{indent}        .{fn.name}({args_str})\n")
        lines.append(f"{indent}        .await\n")
        lines.append(f"{indent}        .map_err(to_napi)\n")

    lines.append(f"{indent}}}\n\n")
    return "".join(lines)


# ── File generators ─────────────────────────────────────────────────────────


def gen_utils() -> str:
    out = ["use napi_derive::napi;\n\n"]

    # Modules to scan for standalone pub fns
    scan = [
        ("crates/fiscal-core/src/standardize.rs", "fiscal_core::standardize", "Standardize"),
        ("crates/fiscal-core/src/convert/mod.rs", "fiscal_core::convert", "Convert"),
    ]

    for rel_path, mod_path, label in scan:
        path = ROOT / rel_path
        if not path.exists():
            continue
        fns = extract_pub_fns(parse_file(path))
        section_fns = []
        for fn in fns:
            wrapper = gen_fn_wrapper(fn, mod_path)
            if wrapper:
                section_fns.append(wrapper)
            else:
                skipped.append((fn.name, "unsupported param type"))
        if section_fns:
            out.append(f"// ── {label} {'─' * (60 - len(label))}\n\n")
            out.extend(section_fns)

    # Complement — only re-exported functions (from pub use in mod.rs)
    complement_dir = ROOT / "crates/fiscal-core/src/complement"
    if complement_dir.exists():
        complement_fns_list = collect_exported_fns(complement_dir)
        section_fns = []
        for fn in complement_fns_list:
            wrapper = gen_fn_wrapper(fn, "fiscal_core::complement")
            if wrapper:
                section_fns.append(wrapper)
            else:
                skipped.append((fn.name, "unsupported param type"))
        if section_fns:
            out.append(f"// ── Complement {'─' * 44}\n\n")
            out.extend(section_fns)

    return "".join(out)


def gen_certificate() -> str:
    out = [
        "use napi::bindgen_prelude::Buffer;\n",
        "use napi_derive::napi;\n\n",
    ]

    # Reuse gen_fn_wrapper for all re-exported certificate functions
    cert_dir = ROOT / "crates/fiscal-crypto/src/certificate"
    all_fns = collect_exported_fns(cert_dir)
    has_enum_params = set()  # track which enums need parse helpers

    for fn in all_fns:
        wrapper = gen_fn_wrapper(fn, "fiscal_crypto::certificate")
        if wrapper:
            out.append(wrapper)
            # Check if any param is an enum that needs a parser
            for p in fn.params:
                t = p.rust_type.strip()
                if t == "SignatureAlgorithm":
                    has_enum_params.add(t)
        else:
            skipped.append((fn.name, "unsupported param type"))

    # Generate enum parse helpers from AST
    if "SignatureAlgorithm" in has_enum_params:
        out.append(gen_enum_parser(
            "SignatureAlgorithm",
            "fiscal_crypto::SignatureAlgorithm",
            ROOT / "crates/fiscal-crypto/src/certificate/pfx.rs",
        ))

    return "".join(out)


def gen_client() -> str:
    out = [
        "use napi::bindgen_prelude::Buffer;\n",
        "use napi_derive::napi;\n\n",
        "use fiscal_core::types::SefazEnvironment;\n\n",
    ]

    # Collect ALL SefazClient methods from all files
    client_dir = ROOT / "crates/fiscal-sefaz/src/client"
    all_methods: list[FnSig] = []

    for rs_file in sorted(client_dir.glob("*.rs")):
        root = parse_file(rs_file)
        methods = extract_impl_methods(root, "SefazClient")
        all_methods.extend(methods)

    # No name-based filtering — extract_fn already filters by visibility (pub only)

    # Generate class
    out.append("#[napi]\npub struct SefazClient {\n")
    out.append("    inner: fiscal_sefaz::client::SefazClient,\n")
    out.append("}\n\n")
    out.append("#[napi]\nimpl SefazClient {\n")

    for method in all_methods:
        if method.name == "new":
            # Constructor — read params from AST but structure is fixed
            # (must create Self, can't be auto-generated as async method)
            napi_params = []
            call_args = []
            for p in method.params:
                decl, expr = map_param(p)
                if decl is None:
                    break
                napi_params.append(decl)
                call_args.append(expr)

            params_str = ", ".join(napi_params)
            args_str = ", ".join(call_args)

            out.append(doc_comment(method.doc, "    "))
            out.append("    #[napi(constructor)]\n")
            out.append(f"    pub fn new({params_str}) -> napi::Result<Self> {{\n")
            out.append(f"        let inner = fiscal_sefaz::client::SefazClient::new({args_str})\n")
            out.append("            .map_err(to_napi)?;\n")
            out.append("        Ok(Self { inner })\n")
            out.append("    }\n\n")
            continue

        wrapper = gen_method_wrapper(method)
        if wrapper:
            out.append(wrapper)
        else:
            skipped.append((method.name, "unsupported param type"))

    out.append("}\n\n")

    if skipped:
        names = ", ".join(f"{n} ({r})" for n, r in skipped)
        out.append(f"// Skipped: {names}\n\n")

    # Helpers
    out.append("""// ── Helpers ─────────────────────────────────────────────────────────────────

fn to_napi(e: fiscal_core::FiscalError) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

fn to_json(v: &impl serde::Serialize) -> napi::Result<serde_json::Value> {
    serde_json::to_value(v).map_err(|e| napi::Error::from_reason(e.to_string()))
}
""")

    # Auto-generate enum parsers from AST
    out.append(gen_enum_parser(
        "SefazEnvironment",
        "SefazEnvironment",
        ROOT / "crates/fiscal-core/src/types/enums.rs",
    ))
    out.append(gen_enum_parser(
        "SefazService",
        "fiscal_sefaz::services::SefazService",
        ROOT / "crates/fiscal-sefaz/src/services.rs",
    ))

    return "".join(out)


def gen_builder() -> str:
    """Generate builder.rs — thin wrapper over InvoiceBuildData + build_from_data.

    Since InvoiceBuildData has Deserialize, serde does all the work.
    No BuildInvoiceConfig, no if-let chains.
    """
    return '''use napi_derive::napi;

/// Build an NF-e/NFC-e XML from a configuration object.
///
/// Accepts the full invoice data as a single JSON object (matching
/// `InvoiceBuildData` fields in camelCase) and returns
/// `{ xml: string, accessKey: string }`.
#[napi(ts_return_type = "{ xml: string; accessKey: string }")]
pub fn build_invoice(config: serde_json::Value) -> napi::Result<serde_json::Value> {
    let data: fiscal_core::types::InvoiceBuildData = serde_json::from_value(config)
        .map_err(|e| napi::Error::from_reason(format!("Invalid config: {e}")))?;

    let result = fiscal_core::xml_builder::build_from_data(&data)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Build and sign an NF-e/NFC-e XML in one step.
///
/// Same as `buildInvoice` but also signs the XML using the provided
/// PEM-encoded private key and certificate.
#[napi(ts_return_type = "{ xml: string; signedXml: string; accessKey: string }")]
pub fn build_and_sign_invoice(
    config: serde_json::Value,
    private_key: String,
    certificate: String,
) -> napi::Result<serde_json::Value> {
    let data: fiscal_core::types::InvoiceBuildData = serde_json::from_value(config)
        .map_err(|e| napi::Error::from_reason(format!("Invalid config: {e}")))?;

    let result = fiscal_core::xml_builder::build_from_data(&data)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let signed_xml =
        fiscal_crypto::certificate::sign_xml(&result.xml, &private_key, &certificate)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    Ok(serde_json::json!({
        "xml": result.xml,
        "signedXml": signed_xml,
        "accessKey": result.access_key,
    }))
}
'''


def _collect_use_names(node: ts.Node, names: set[str]):
    """Recursively collect imported identifiers from a use_declaration AST node."""
    if node.type == "identifier":
        names.add(txt(node))
    elif node.type == "use_list":
        for c in node.children:
            _collect_use_names(c, names)
    elif node.type == "scoped_use_list":
        # path::{a, b} — recurse into the use_list child
        for c in node.children:
            if c.type == "use_list":
                _collect_use_names(c, names)
    elif node.type == "scoped_identifier":
        # path::name — the last identifier is the imported name
        last_ident = [c for c in node.children if c.type == "identifier"]
        if last_ident:
            names.add(txt(last_ident[-1]))


def get_reexported_names(mod_rs: Path) -> set[str]:
    """Parse a mod.rs and return names from `pub use ...` statements using AST."""
    if not mod_rs.exists():
        return set()
    root = parse_file(mod_rs)
    names = set()
    for node in root.children:
        if node.type == "use_declaration":
            if any(c.type == "visibility_modifier" for c in node.children):
                # Walk the AST children to extract imported names
                for c in node.children:
                    if c.type in ("scoped_use_list", "scoped_identifier", "use_list", "identifier"):
                        _collect_use_names(c, names)
        elif node.type == "function_item":
            fn = extract_fn(node)
            if fn:
                names.add(fn.name)
    return names


def collect_exported_fns(mod_dir: Path) -> list[FnSig]:
    """Collect all publicly re-exported functions from a module directory."""
    mod_rs = mod_dir / "mod.rs"
    exported = get_reexported_names(mod_rs)

    fns = []
    for rs_file in sorted(mod_dir.glob("*.rs")):
        for fn in extract_pub_fns(parse_file(rs_file)):
            if fn.name in exported:
                fns.append(fn)
    return fns


def gen_lib() -> str:
    return '#![doc = "Node.js native binding for fiscal-rs via napi-rs."]\n\nmod builder;\nmod certificate;\nmod client;\nmod utils;\n'


# ── Main ─────────────────────────────────────────────────────────────────────


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()

    files = {
        "lib.rs": gen_lib(),
        "builder.rs": gen_builder(),
        "certificate.rs": gen_certificate(),
        "client.rs": gen_client(),
        "utils.rs": gen_utils(),
    }

    for name, content in files.items():
        if args.write:
            path = NAPI_SRC / name
            NAPI_SRC.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        print(f"  {name:20s} {len(content.splitlines()):>4d} lines")

    print(f"\nTotal: {sum(len(c.splitlines()) for c in files.values())} lines")

    if skipped:
        print(f"\nSkipped ({len(skipped)}):")
        for fn_name, reason in skipped:
            print(f"  - {fn_name}: {reason}")

    if args.write:
        print("\nRun: cargo fmt -p fiscal-napi && cargo check -p fiscal-napi")
    else:
        print("\nUse --write to generate files.")


if __name__ == "__main__":
    main()
