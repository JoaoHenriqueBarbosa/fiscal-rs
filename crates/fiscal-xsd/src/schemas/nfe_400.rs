use crate::XsdSchema;

static FILES: &[(&str, &[u8])] = &[
    (
        "enviNFe_v4.00.xsd",
        include_bytes!("../../schemas/nfe_pl010/enviNFe_v4.00.xsd"),
    ),
    (
        "leiauteNFe_v4.00.xsd",
        include_bytes!("../../schemas/nfe_pl010/leiauteNFe_v4.00.xsd"),
    ),
    (
        "tiposBasico_v4.00.xsd",
        include_bytes!("../../schemas/nfe_pl010/tiposBasico_v4.00.xsd"),
    ),
    (
        "DFeTiposBasicos_v1.00.xsd",
        include_bytes!("../../schemas/nfe_pl010/DFeTiposBasicos_v1.00.xsd"),
    ),
    (
        "xmldsig-core-schema_v1.01.xsd",
        include_bytes!("../../schemas/nfe_pl010/xmldsig-core-schema_v1.01.xsd"),
    ),
];

static SCHEMA: XsdSchema = XsdSchema::new("nfe_400", FILES, "enviNFe_v4.00.xsd");

/// Returns the compiled NF-e/NFC-e 4.00 (PL_010) schema bundle.
pub fn nfe_lote() -> &'static XsdSchema {
    &SCHEMA
}
