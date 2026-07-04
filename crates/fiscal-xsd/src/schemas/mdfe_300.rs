use crate::XsdSchema;

static FILES: &[(&str, &[u8])] = &[
    (
        "mdfe_v3.00.xsd",
        include_bytes!("../../schemas/mdfe_300/mdfe_v3.00.xsd"),
    ),
    (
        "mdfeTiposBasico_v3.00.xsd",
        include_bytes!("../../schemas/mdfe_300/mdfeTiposBasico_v3.00.xsd"),
    ),
    (
        "tiposGeralMDFe_v3.00.xsd",
        include_bytes!("../../schemas/mdfe_300/tiposGeralMDFe_v3.00.xsd"),
    ),
    (
        "mdfeModalRodoviario_v3.00.xsd",
        include_bytes!("../../schemas/mdfe_300/mdfeModalRodoviario_v3.00.xsd"),
    ),
    (
        "mdfeModalAereo_v3.00.xsd",
        include_bytes!("../../schemas/mdfe_300/mdfeModalAereo_v3.00.xsd"),
    ),
    (
        "mdfeModalAquaviario_v3.00.xsd",
        include_bytes!("../../schemas/mdfe_300/mdfeModalAquaviario_v3.00.xsd"),
    ),
    (
        "mdfeModalFerroviario_v3.00.xsd",
        include_bytes!("../../schemas/mdfe_300/mdfeModalFerroviario_v3.00.xsd"),
    ),
    (
        "xmldsig-core-schema_v1.01.xsd",
        include_bytes!("../../schemas/mdfe_300/xmldsig-core-schema_v1.01.xsd"),
    ),
];

static SCHEMA: XsdSchema = XsdSchema::new("mdfe_300", FILES, "mdfe_v3.00.xsd");

/// Returns the compiled MDF-e 3.00 schema bundle.
pub fn mdfe() -> &'static XsdSchema {
    &SCHEMA
}
