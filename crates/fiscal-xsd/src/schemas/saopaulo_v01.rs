use crate::XsdSchema;

static FILES: &[(&str, &[u8])] = &[
    (
        "PedidoEnvioLoteRPS_v01.xsd",
        include_bytes!("../../schemas/saopaulo_v01/PedidoEnvioLoteRPS_v01.xsd"),
    ),
    (
        "TiposNFe_v01.xsd",
        include_bytes!("../../schemas/saopaulo_v01/TiposNFe_v01.xsd"),
    ),
    (
        "xmldsig-core-schema_v01.xsd",
        include_bytes!("../../schemas/saopaulo_v01/xmldsig-core-schema_v01.xsd"),
    ),
];

static SCHEMA: XsdSchema = XsdSchema::new("saopaulo_v01", FILES, "PedidoEnvioLoteRPS_v01.xsd");

/// Returns the compiled São Paulo PMSP v01 schema bundle.
pub fn sp_lote_rps() -> &'static XsdSchema {
    &SCHEMA
}
