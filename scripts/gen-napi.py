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
import re
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

    # SefazEnvironment → String (parsed at runtime)
    if t == "SefazEnvironment":
        return f"{p.name}: String", "env"

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

    # Option<&str>
    if t == "Option<&str>":
        return f"{p.name}: Option<String>", f"{p.name}.as_deref()"

    # Option<u32> etc
    m = re.match(r"Option<(u32|i32|u16|i16|bool|f64)>", t)
    if m:
        return f"{p.name}: Option<{m.group(1)}>", p.name

    # SignatureAlgorithm → String (parsed at runtime)
    if t == "SignatureAlgorithm":
        return f"{p.name}: String", "algo"

    # Complex struct types passed by value — need serde
    # (EpecData, Vec<EventItem>, etc.)
    # For now, skip these methods (they need manual handling or serde)
    return None, None


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

    for p in fn.params:
        decl, expr = map_param(p)
        if decl is None:
            return None  # unsupported param type, skip this fn
        napi_params.append(decl)
        call_args.append(expr)
        if p.rust_type.strip() == "SefazEnvironment":
            needs_env_parse = True

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

    if needs_env_parse:
        lines.append(f"{indent}    let env = parse_env(&environment)?;\n")

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
    needs_env_parse = False

    for p in fn.params:
        decl, expr = map_param(p)
        if decl is None:
            return None  # unsupported param type, skip
        napi_params.append(decl)
        call_args.append(expr)
        if p.rust_type.strip() == "SefazEnvironment":
            needs_env_parse = True

    params_str = ",\n{i}    ".format(i=indent).join(napi_params)
    args_str = ", ".join(call_args)

    lines = []
    lines.append(doc_comment(fn.doc, indent))

    if ret_handling == "serde":
        lines.append(f'{indent}#[napi(ts_return_type = "Promise<Record<string, unknown>>")]\n')
    else:
        lines.append(f"{indent}#[napi]\n")

    lines.append(f"{indent}pub async fn {fn.name}(\n{indent}    {params_str},\n{indent}) -> {ret_type} {{\n")

    if needs_env_parse:
        lines.append(f"{indent}    let env = parse_env(&environment)?;\n")

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
        if section_fns:
            out.append(f"// ── {label} {'─' * (60 - len(label))}\n\n")
            out.extend(section_fns)

    # Complement — only re-exported functions (from pub use in mod.rs)
    complement_dir = ROOT / "crates/fiscal-core/src/complement"
    if complement_dir.exists():
        # Get re-exported names from mod.rs
        mod_rs = complement_dir / "mod.rs"
        if mod_rs.exists():
            mod_root = parse_file(mod_rs)
            # Collect pub use re-exports AND pub fns defined in mod.rs
            exported_names = set()
            for node in mod_root.children:
                if node.type == "use_declaration":
                    # pub use foo::bar; or pub use foo::{bar, baz};
                    if any(c.type == "visibility_modifier" for c in node.children):
                        # Extract the imported names
                        for name in re.findall(r"\b(\w+)\b", txt(node)):
                            if name not in ("pub", "use", "crate", "super", "self"):
                                exported_names.add(name)

            complement_fns = []
            for rs_file in sorted(complement_dir.glob("*.rs")):
                fns = extract_pub_fns(parse_file(rs_file))
                for fn in fns:
                    if fn.name in exported_names:
                        wrapper = gen_fn_wrapper(fn, "fiscal_core::complement")
                        if wrapper:
                            complement_fns.append(wrapper)
            if complement_fns:
                out.append(f"// ── Complement {'─' * 44}\n\n")
                out.extend(complement_fns)

    return "".join(out)


def gen_certificate() -> str:
    out = [
        "use napi::bindgen_prelude::Buffer;\n",
        "use napi_derive::napi;\n\n",
        "fn to_napi(e: impl std::fmt::Display) -> napi::Error {\n",
        "    napi::Error::from_reason(e.to_string())\n",
        "}\n\n",
        "fn to_json(v: &impl serde::Serialize) -> napi::Result<serde_json::Value> {\n",
        "    serde_json::to_value(v).map_err(to_napi)\n",
        "}\n\n",
    ]

    # Only re-exported functions from certificate module
    cert_dir = ROOT / "crates/fiscal-crypto/src/certificate"
    all_fns = collect_exported_fns(cert_dir)

    for fn in all_fns:
            if fn.name == "ensure_modern_pfx":
                continue  # internal utility

            # Special cases: functions that take &[u8] (pfx_buffer) and return
            # CertificateData/CertificateInfo — these need serde serialization
            ret = fn.return_type or ""
            returns_struct = re.match(r"Result<(CertificateData|CertificateInfo)\s*,", ret)

            if returns_struct:
                # These return structs that have Serialize — use to_json
                struct_name = returns_struct.group(1)
                napi_params = []
                call_args = []
                for p in fn.params:
                    decl, expr = map_param(p)
                    if decl is None:
                        break
                    napi_params.append(decl)
                    call_args.append(expr)
                else:
                    params_str = ", ".join(napi_params)
                    args_str = ", ".join(call_args)

                    out.append(doc_comment(fn.doc))
                    out.append(f'#[napi(ts_return_type = "Record<string, unknown>")]\n')
                    out.append(f"pub fn {fn.name}({params_str}) -> napi::Result<serde_json::Value> {{\n")
                    out.append(f"    let result =\n")
                    out.append(f"        fiscal_crypto::certificate::{fn.name}({args_str}).map_err(to_napi)?;\n")
                    out.append(f"    to_json(&result)\n")
                    out.append(f"}}\n\n")
            else:
                # sign_* functions — all return Result<String, ...>
                # Check if it has an algorithm param (SignatureAlgorithm)
                has_algo = any(p.rust_type.strip() == "SignatureAlgorithm" for p in fn.params)

                napi_params = []
                call_args = []
                for p in fn.params:
                    t = p.rust_type.strip()
                    if t == "SignatureAlgorithm":
                        napi_params.append(f"{p.name}: String")
                        call_args.append("algo")
                    elif t in ("&str", "&String"):
                        napi_params.append(f"{p.name}: String")
                        call_args.append(f"&{p.name}")
                    elif t == "&[u8]":
                        napi_params.append(f"{p.name}: Buffer")
                        call_args.append(f"&{p.name}")
                    else:
                        napi_params.append(f"{p.name}: String")
                        call_args.append(f"&{p.name}")

                params_str = ", ".join(napi_params)
                args_str = ", ".join(call_args)

                out.append(doc_comment(fn.doc))
                out.append(f"#[napi]\n")
                out.append(f"pub fn {fn.name}({params_str}) -> napi::Result<String> {{\n")
                if has_algo:
                    out.append(f"    let algo = parse_algorithm(&algorithm)?;\n")
                out.append(f"    fiscal_crypto::certificate::{fn.name}({args_str})\n")
                out.append(f"        .map_err(to_napi)\n")
                out.append(f"}}\n\n")

    # parse_algorithm helper
    out.append("""fn parse_algorithm(s: &str) -> napi::Result<fiscal_crypto::SignatureAlgorithm> {
    match s.to_lowercase().as_str() {
        "sha1" => Ok(fiscal_crypto::SignatureAlgorithm::Sha1),
        "sha256" => Ok(fiscal_crypto::SignatureAlgorithm::Sha256),
        _ => Err(napi::Error::from_reason(format!(
            "Invalid algorithm: \\"{s}\\". Expected \\"sha1\\" or \\"sha256\\"."
        ))),
    }
}
""")

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

    # Filter: only pub methods, skip internal helpers (send_*, _raw, etc.)
    public_methods = [
        m for m in all_methods
        if not m.name.startswith("send_")
        and not m.name.endswith("_raw")
    ]

    # Generate class
    out.append("#[napi]\npub struct SefazClient {\n")
    out.append("    inner: fiscal_sefaz::client::SefazClient,\n")
    out.append("}\n\n")
    out.append("#[napi]\nimpl SefazClient {\n")

    skipped = []

    for method in public_methods:
        if method.name == "new":
            # Constructor — hardcoded because it's structural
            out.append("    /// Create a new SEFAZ client from a PKCS#12 (PFX) certificate buffer.\n")
            out.append("    #[napi(constructor)]\n")
            out.append("    pub fn new(pfx_buffer: Buffer, passphrase: String) -> napi::Result<Self> {\n")
            out.append("        let inner = fiscal_sefaz::client::SefazClient::new(&pfx_buffer, &passphrase)\n")
            out.append("            .map_err(to_napi)?;\n")
            out.append("        Ok(Self { inner })\n")
            out.append("    }\n\n")
            continue

        if method.name == "send":
            # Low-level send — special because it takes SefazService enum
            out.append(doc_comment(method.doc, "    "))
            out.append("    #[napi]\n")
            out.append("    pub async fn send(\n")
            out.append("        &self,\n")
            out.append("        service: String,\n")
            out.append("        uf: String,\n")
            out.append("        environment: String,\n")
            out.append("        request_xml: String,\n")
            out.append("    ) -> napi::Result<String> {\n")
            out.append("        let env = parse_env(&environment)?;\n")
            out.append("        let svc = parse_service(&service)?;\n")
            out.append("        self.inner\n")
            out.append("            .send(svc, &uf, env, &request_xml)\n")
            out.append("            .await\n")
            out.append("            .map_err(to_napi)\n")
            out.append("    }\n\n")
            continue

        if method.name == "send_model":
            continue  # internal variant of send

        wrapper = gen_method_wrapper(method)
        if wrapper:
            out.append(wrapper)
        else:
            skipped.append(method.name)

    out.append("}\n\n")

    if skipped:
        out.append(f"// Skipped methods (unsupported param types): {', '.join(skipped)}\n\n")

    # Helpers
    out.append("""// ── Helpers ─────────────────────────────────────────────────────────────────

fn to_napi(e: fiscal_core::FiscalError) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

fn to_json(v: &impl serde::Serialize) -> napi::Result<serde_json::Value> {
    serde_json::to_value(v).map_err(|e| napi::Error::from_reason(e.to_string()))
}

fn parse_env(s: &str) -> napi::Result<SefazEnvironment> {
    match s.to_lowercase().as_str() {
        "production" | "1" => Ok(SefazEnvironment::Production),
        "homologation" | "2" => Ok(SefazEnvironment::Homologation),
        _ => Err(napi::Error::from_reason(format!(
            "Invalid environment: \\"{s}\\". Expected \\"production\\" or \\"homologation\\"."
        ))),
    }
}

fn parse_service(s: &str) -> napi::Result<fiscal_sefaz::services::SefazService> {
    use fiscal_sefaz::services::SefazService;
""")

    # Auto-generate service variants from the enum
    services_path = ROOT / "crates/fiscal-sefaz/src/services.rs"
    if services_path.exists():
        root = parse_file(services_path)
        for node in root.children:
            if node.type == "enum_item":
                name_node = node.child_by_field_name("name")
                if name_node and txt(name_node) == "SefazService":
                    body = node.child_by_field_name("body")
                    if body:
                        out.append("    match s {\n")
                        for c in body.children:
                            if c.type == "enum_variant":
                                vname = txt(c.child_by_field_name("name"))
                                out.append(f'        "{vname}" => Ok(SefazService::{vname}),\n')
                        out.append('        _ => Err(napi::Error::from_reason(format!(\n')
                        out.append('            "Unknown service: \\"{s}\\""\n')
                        out.append("        ))),\n")
                        out.append("    }\n")

    out.append("}\n")

    return "".join(out)


def gen_builder() -> str:
    """Generate builder.rs from InvoiceBuilder<Draft> setter methods."""
    builder_path = ROOT / "crates/fiscal-core/src/xml_builder/builder.rs"
    root = parse_file(builder_path)

    # Find InvoiceBuilder impl blocks and collect Draft setters
    all_methods = extract_impl_methods(root, "InvoiceBuilder<Draft>")
    # Also try just "InvoiceBuilder" in case tree-sitter parses differently
    all_methods.extend(extract_impl_methods(root, "InvoiceBuilder"))

    # Setters: methods that return Self (consume self)
    setters = []
    seen = set()
    for m in all_methods:
        if m.name in seen:
            continue
        seen.add(m.name)
        if m.has_self and m.return_type and "Self" in m.return_type and m.name not in ("new", "build"):
            if m.params:  # must have at least one value param
                setters.append(m)

    # Categorize params for the config struct
    required_fields = {"items", "payments"}  # These are Vec, always required

    out = [
        "use napi_derive::napi;\n",
        "use serde::Deserialize;\n\n",
        "use fiscal_core::newtypes::Cents;\n",
        "use fiscal_core::types::*;\n",
        "use fiscal_core::xml_builder::InvoiceBuilder;\n\n",
    ]

    # ── buildInvoice ──
    out.append("""/// Build an NF-e/NFC-e XML from a configuration object.
///
/// Accepts the full invoice data as a single JSON object and returns
/// `{ xml: string, accessKey: string }`.
#[napi(ts_return_type = "{ xml: string; accessKey: string }")]
pub fn build_invoice(config: serde_json::Value) -> napi::Result<serde_json::Value> {
    let cfg: BuildInvoiceConfig = serde_json::from_value(config)
        .map_err(|e| napi::Error::from_reason(format!("Invalid config: {e}")))?;

    let mut builder = InvoiceBuilder::new(cfg.issuer, cfg.environment, cfg.model);

""")

    for s in setters:
        name = s.name
        p = s.params[0]
        t = p.rust_type.strip()

        if name in required_fields:
            out.append(f"    builder = builder.{name}(cfg.{name});\n")
        elif t.startswith("DateTime<"):
            out.append(f"    if let Some(ref v) = cfg.{name} {{\n")
            out.append(f"        let dt = chrono::DateTime::parse_from_rfc3339(v)\n")
            out.append(f'            .map_err(|e| napi::Error::from_reason(format!("Invalid {name}: {{e}}")))?;\n')
            out.append(f"        builder = builder.{name}(dt);\n")
            out.append(f"    }}\n")
        elif t.startswith("impl Into<String>"):
            out.append(f"    if let Some(v) = cfg.{name} {{\n")
            out.append(f"        builder = builder.{name}(v);\n")
            out.append(f"    }}\n")
        else:
            out.append(f"    if let Some(v) = cfg.{name} {{\n")
            out.append(f"        builder = builder.{name}(v);\n")
            out.append(f"    }}\n")

    out.append("""
    let built = builder
        .build()
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    Ok(serde_json::json!({
        "xml": built.xml(),
        "accessKey": built.access_key(),
    }))
}

/// Build and sign an NF-e/NFC-e XML in one step.
#[napi(ts_return_type = "{ xml: string; signedXml: string; accessKey: string }")]
pub fn build_and_sign_invoice(
    config: serde_json::Value,
    private_key: String,
    certificate: String,
) -> napi::Result<serde_json::Value> {
    let result = build_invoice(config)?;
    let xml = result["xml"].as_str().unwrap();
    let access_key = result["accessKey"].as_str().unwrap().to_string();

    let signed_xml = fiscal_crypto::certificate::sign_xml(xml, &private_key, &certificate)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    Ok(serde_json::json!({
        "xml": xml,
        "signedXml": signed_xml,
        "accessKey": access_key,
    }))
}

""")

    # ── BuildInvoiceConfig struct ──
    out.append("// ── Config (auto-generated from InvoiceBuilder<Draft> setters) ──\n\n")
    out.append("#[derive(Deserialize)]\n")
    out.append('#[serde(rename_all = "camelCase")]\n')
    out.append("struct BuildInvoiceConfig {\n")
    out.append("    // Required\n")
    out.append("    issuer: IssuerData,\n")
    out.append("    environment: SefazEnvironment,\n")
    out.append("    model: InvoiceModel,\n")

    for s in setters:
        name = s.name
        p = s.params[0]
        t = p.rust_type.strip()

        if name in required_fields:
            # Required Vec fields
            out.append(f"    {name}: {t},\n")
        elif t.startswith("impl Into<String>"):
            out.append(f"    {name}: Option<String>,\n")
        elif t.startswith("DateTime<"):
            out.append(f"    /// ISO 8601 string\n")
            out.append(f"    {name}: Option<String>,\n")
        elif t.startswith("Vec<"):
            # Replace crate:: with fiscal_core:: since we're in a different crate
            fixed_t = t.replace("crate::", "fiscal_core::")
            out.append(f"    {name}: Option<{fixed_t}>,\n")
        else:
            fixed_t = t.replace("crate::", "fiscal_core::")
            out.append(f"    {name}: Option<{fixed_t}>,\n")

    out.append("}\n")

    return "".join(out)


def get_reexported_names(mod_rs: Path) -> set[str]:
    """Parse a mod.rs and return names from `pub use ...` statements."""
    if not mod_rs.exists():
        return set()
    root = parse_file(mod_rs)
    names = set()
    for node in root.children:
        if node.type == "use_declaration":
            if any(c.type == "visibility_modifier" for c in node.children):
                text = txt(node)
                # Extract identifiers from `pub use foo::{bar, baz};` or `pub use foo::bar;`
                # Remove the `pub use path::` prefix and parse the names
                for name in re.findall(r"\b([a-z_][a-z0-9_]*)\b", text):
                    if name not in ("pub", "use", "crate", "super", "self", "mod"):
                        names.add(name)
        elif node.type == "function_item":
            # pub fn defined directly in mod.rs
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

    if args.write:
        NAPI_SRC.mkdir(parents=True, exist_ok=True)
        for name, content in files.items():
            path = NAPI_SRC / name
            path.write_text(content)
            print(f"  {name:20s} {len(content.splitlines()):>4d} lines")
        print(f"\nTotal: {sum(len(c.splitlines()) for c in files.values())} lines")
        print("Run: cargo fmt -p fiscal-napi && cargo check -p fiscal-napi")
    else:
        for name, content in files.items():
            print(f"  {name:20s} {len(content.splitlines()):>4d} lines")
        print(f"\nTotal: {sum(len(c.splitlines()) for c in files.values())} lines")
        print("Use --write to generate files.")


if __name__ == "__main__":
    main()
