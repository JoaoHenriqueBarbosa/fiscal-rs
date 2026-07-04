use crate::XsdSchema;

static FILES: &[(&str, &[u8])] = &[
    (
        "PedidoEnvioLoteRPS_v02.xsd",
        include_bytes!("../../schemas/saopaulo_v02/PedidoEnvioLoteRPS_v02.xsd"),
    ),
    (
        "TiposNFe_v02.xsd",
        include_bytes!("../../schemas/saopaulo_v02/TiposNFe_v02.xsd"),
    ),
    (
        "xmldsig-core-schema_v02.xsd",
        include_bytes!("../../schemas/saopaulo_v02/xmldsig-core-schema_v02.xsd"),
    ),
];

static SCHEMA: XsdSchema = XsdSchema::new("saopaulo_v02", FILES, "PedidoEnvioLoteRPS_v02.xsd");

/// Returns the compiled São Paulo PMSP v02 (reforma tributária) schema bundle.
pub fn sp_lote_rps_v2() -> &'static XsdSchema {
    &SCHEMA
}
