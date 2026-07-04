use crate::XsdSchema;

static FILES: &[(&str, &[u8])] = &[
    (
        "bpe_v1.00.xsd",
        include_bytes!("../../schemas/bpe_100/bpe_v1.00.xsd"),
    ),
    (
        "bpeTiposBasico_v1.00.xsd",
        include_bytes!("../../schemas/bpe_100/bpeTiposBasico_v1.00.xsd"),
    ),
    (
        "tiposGeralBPe_v1.00.xsd",
        include_bytes!("../../schemas/bpe_100/tiposGeralBPe_v1.00.xsd"),
    ),
    (
        "xmldsig-core-schema_v1.01.xsd",
        include_bytes!("../../schemas/bpe_100/xmldsig-core-schema_v1.01.xsd"),
    ),
];

static SCHEMA: XsdSchema = XsdSchema::new("bpe_100", FILES, "bpe_v1.00.xsd");

/// Returns the compiled BP-e 1.00 schema bundle.
pub fn bpe() -> &'static XsdSchema {
    &SCHEMA
}
