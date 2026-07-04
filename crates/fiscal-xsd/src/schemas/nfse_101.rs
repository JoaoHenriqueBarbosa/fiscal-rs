use crate::XsdSchema;

static DPS_FILES: &[(&str, &[u8])] = &[
    (
        "DPS_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/DPS_v1.01.xsd"),
    ),
    (
        "NFSe_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/NFSe_v1.01.xsd"),
    ),
    (
        "tiposComplexos_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/tiposComplexos_v1.01.xsd"),
    ),
    (
        "tiposSimples_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/tiposSimples_v1.01.xsd"),
    ),
    (
        "tiposEventos_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/tiposEventos_v1.01.xsd"),
    ),
    (
        "xmldsig-core-schema.xsd",
        include_bytes!("../../schemas/nfse_101/xmldsig-core-schema.xsd"),
    ),
];

static EVENTO_FILES: &[(&str, &[u8])] = &[
    (
        "pedRegEvento_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/pedRegEvento_v1.01.xsd"),
    ),
    (
        "tiposEventos_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/tiposEventos_v1.01.xsd"),
    ),
    (
        "tiposComplexos_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/tiposComplexos_v1.01.xsd"),
    ),
    (
        "tiposSimples_v1.01.xsd",
        include_bytes!("../../schemas/nfse_101/tiposSimples_v1.01.xsd"),
    ),
    (
        "xmldsig-core-schema.xsd",
        include_bytes!("../../schemas/nfse_101/xmldsig-core-schema.xsd"),
    ),
];

static DPS_SCHEMA: XsdSchema = XsdSchema::new("nfse_dps_101", DPS_FILES, "DPS_v1.01.xsd");
static EVENTO_SCHEMA: XsdSchema =
    XsdSchema::new("nfse_evento_101", EVENTO_FILES, "pedRegEvento_v1.01.xsd");

/// Returns the compiled NFS-e Nacional DPS 1.01 schema bundle.
pub fn dps() -> &'static XsdSchema {
    &DPS_SCHEMA
}

/// Returns the compiled NFS-e Nacional evento (pedRegEvento) 1.01 schema bundle.
pub fn nfse_evento() -> &'static XsdSchema {
    &EVENTO_SCHEMA
}
