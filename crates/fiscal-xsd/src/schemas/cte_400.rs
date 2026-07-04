use crate::XsdSchema;

static CTE_FILES: &[(&str, &[u8])] = &[
    (
        "cte_v4.00.xsd",
        include_bytes!("../../schemas/cte_400/cte_v4.00.xsd"),
    ),
    (
        "cteTiposBasico_v4.00.xsd",
        include_bytes!("../../schemas/cte_400/cteTiposBasico_v4.00.xsd"),
    ),
    (
        "tiposGeralCTe_v4.00.xsd",
        include_bytes!("../../schemas/cte_400/tiposGeralCTe_v4.00.xsd"),
    ),
    (
        "xmldsig-core-schema_v1.01.xsd",
        include_bytes!("../../schemas/cte_400/xmldsig-core-schema_v1.01.xsd"),
    ),
];

static CTEOS_FILES: &[(&str, &[u8])] = &[
    (
        "cteOS_v4.00.xsd",
        include_bytes!("../../schemas/cte_400/cteOS_v4.00.xsd"),
    ),
    (
        "cteTiposBasico_v4.00.xsd",
        include_bytes!("../../schemas/cte_400/cteTiposBasico_v4.00.xsd"),
    ),
    (
        "tiposGeralCTe_v4.00.xsd",
        include_bytes!("../../schemas/cte_400/tiposGeralCTe_v4.00.xsd"),
    ),
    (
        "xmldsig-core-schema_v1.01.xsd",
        include_bytes!("../../schemas/cte_400/xmldsig-core-schema_v1.01.xsd"),
    ),
];

static GTVE_FILES: &[(&str, &[u8])] = &[
    (
        "GTVe_v4.00.xsd",
        include_bytes!("../../schemas/cte_400/GTVe_v4.00.xsd"),
    ),
    (
        "cteTiposBasico_v4.00.xsd",
        include_bytes!("../../schemas/cte_400/cteTiposBasico_v4.00.xsd"),
    ),
    (
        "tiposGeralCTe_v4.00.xsd",
        include_bytes!("../../schemas/cte_400/tiposGeralCTe_v4.00.xsd"),
    ),
    (
        "xmldsig-core-schema_v1.01.xsd",
        include_bytes!("../../schemas/cte_400/xmldsig-core-schema_v1.01.xsd"),
    ),
];

static CTE_SCHEMA: XsdSchema = XsdSchema::new("cte_400", CTE_FILES, "cte_v4.00.xsd");
static CTEOS_SCHEMA: XsdSchema = XsdSchema::new("cteos_400", CTEOS_FILES, "cteOS_v4.00.xsd");
static GTVE_SCHEMA: XsdSchema = XsdSchema::new("gtve_400", GTVE_FILES, "GTVe_v4.00.xsd");

/// Returns the compiled CT-e 4.00 schema bundle.
pub fn cte() -> &'static XsdSchema {
    &CTE_SCHEMA
}

/// Returns the compiled CT-e OS 4.00 schema bundle.
pub fn cteos() -> &'static XsdSchema {
    &CTEOS_SCHEMA
}

/// Returns the compiled GTV-e 4.00 schema bundle.
pub fn gtve() -> &'static XsdSchema {
    &GTVE_SCHEMA
}
