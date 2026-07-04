use crate::XsdSchema;

static FILES: &[(&str, &[u8])] = &[
    (
        "nfse.xsd",
        include_bytes!("../../schemas/abrasf_203/nfse.xsd"),
    ),
    (
        "nfse_servico.xsd",
        include_bytes!("../../schemas/abrasf_203/nfse_servico.xsd"),
    ),
    (
        "xmldsig-core-schema.xsd",
        include_bytes!("../../schemas/abrasf_203/xmldsig-core-schema.xsd"),
    ),
];

static SCHEMA: XsdSchema = XsdSchema::new("abrasf_203", FILES, "nfse.xsd");

/// Returns the compiled ABRASF 2.03 schema bundle (GerarNfseEnvio).
pub fn abrasf_gerar_nfse() -> &'static XsdSchema {
    &SCHEMA
}
