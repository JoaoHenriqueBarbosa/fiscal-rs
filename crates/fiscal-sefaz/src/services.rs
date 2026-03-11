//! SEFAZ web service operation metadata.
//!
//! Maps each NF-e/NFC-e web service to its SOAP method name, WSDL operation
//! identifier, and XML schema version — the three pieces needed to build the
//! SOAP envelope and `Content-Type` action header.

/// SOAP metadata for a single SEFAZ web service.
///
/// Constructed only via [`SefazService::meta`]; the fields are read-only to
/// external crates thanks to `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct ServiceMeta {
    /// SOAP method name (e.g. `"nfeStatusServicoNF"`).
    pub method: &'static str,
    /// WSDL operation identifier used to build the namespace
    /// (e.g. `"NFeStatusServico4"`).
    pub operation: &'static str,
    /// XML schema version sent in `<versaoDados>` (e.g. `"4.00"`).
    pub version: &'static str,
}

/// NF-e/NFC-e SEFAZ web service operations.
///
/// Each variant maps to one WSDL endpoint and carries fixed SOAP metadata
/// retrievable via [`meta()`](SefazService::meta).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SefazService {
    /// `NFeStatusServico4` — check SEFAZ operational status.
    StatusServico,
    /// `NFeAutorizacao4` — submit NF-e batch for authorization.
    Autorizacao,
    /// `NFeRetAutorizacao4` — query batch result by receipt number.
    RetAutorizacao,
    /// `NFeConsultaProtocolo4` — consult NF-e by 44-digit access key.
    ConsultaProtocolo,
    /// `NFeInutilizacao4` — void an unused invoice number range.
    Inutilizacao,
    /// `NFeRecepcaoEvento4` — submit events (cancel, CCe, manifest, …).
    RecepcaoEvento,
    /// `NFeDistribuicaoDFe` — distribute fiscal documents (DF-e).
    DistribuicaoDFe,
    /// `CadConsultaCadastro4` — query taxpayer registration.
    ConsultaCadastro,
}

impl SefazService {
    /// Return the SOAP metadata for this service.
    ///
    /// # Examples
    ///
    /// ```
    /// use fiscal_sefaz::services::SefazService;
    ///
    /// let meta = SefazService::StatusServico.meta();
    /// assert_eq!(meta.method, "nfeStatusServicoNF");
    /// assert_eq!(meta.operation, "NFeStatusServico4");
    /// assert_eq!(meta.version, "4.00");
    /// ```
    pub fn meta(self) -> ServiceMeta {
        match self {
            Self::StatusServico => ServiceMeta {
                method: "nfeStatusServicoNF",
                operation: "NFeStatusServico4",
                version: "4.00",
            },
            Self::Autorizacao => ServiceMeta {
                method: "nfeAutorizacaoLote",
                operation: "NFeAutorizacao4",
                version: "4.00",
            },
            Self::RetAutorizacao => ServiceMeta {
                method: "nfeRetAutorizacaoLote",
                operation: "NFeRetAutorizacao4",
                version: "4.00",
            },
            Self::ConsultaProtocolo => ServiceMeta {
                method: "nfeConsultaNF",
                operation: "NFeConsultaProtocolo4",
                version: "4.00",
            },
            Self::Inutilizacao => ServiceMeta {
                method: "nfeInutilizacaoNF",
                operation: "NFeInutilizacao4",
                version: "4.00",
            },
            Self::RecepcaoEvento => ServiceMeta {
                method: "nfeRecepcaoEvento",
                operation: "NFeRecepcaoEvento4",
                version: "1.00",
            },
            Self::DistribuicaoDFe => ServiceMeta {
                method: "nfeDistDFeInteresse",
                operation: "NFeDistribuicaoDFe",
                version: "1.01",
            },
            Self::ConsultaCadastro => ServiceMeta {
                method: "consultaCadastro",
                operation: "CadConsultaCadastro4",
                version: "2.00",
            },
        }
    }

    /// Service name used as lookup key in [`crate::urls::get_sefaz_url`].
    pub fn url_key(self) -> &'static str {
        match self {
            Self::StatusServico => "NfeStatusServico",
            Self::Autorizacao => "NfeAutorizacao",
            Self::RetAutorizacao => "NfeRetAutorizacao",
            Self::ConsultaProtocolo => "NfeConsultaProtocolo",
            Self::Inutilizacao => "NfeInutilizacao",
            Self::RecepcaoEvento => "RecepcaoEvento",
            Self::DistribuicaoDFe => "NfeDistribuicaoDFe",
            Self::ConsultaCadastro => "NfeConsultaCadastro",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_services_have_non_empty_meta() {
        let services = [
            SefazService::StatusServico,
            SefazService::Autorizacao,
            SefazService::RetAutorizacao,
            SefazService::ConsultaProtocolo,
            SefazService::Inutilizacao,
            SefazService::RecepcaoEvento,
            SefazService::DistribuicaoDFe,
            SefazService::ConsultaCadastro,
        ];
        for svc in services {
            let meta = svc.meta();
            assert!(!meta.method.is_empty(), "{svc:?} has empty method");
            assert!(!meta.operation.is_empty(), "{svc:?} has empty operation");
            assert!(!meta.version.is_empty(), "{svc:?} has empty version");
        }
    }

    #[test]
    fn url_keys_match_urls_module_expectations() {
        // These keys must match the `service` parameter accepted by
        // `urls::get_sefaz_url` — if they drift, URL lookups will fail.
        assert_eq!(SefazService::StatusServico.url_key(), "NfeStatusServico");
        assert_eq!(SefazService::Autorizacao.url_key(), "NfeAutorizacao");
        assert_eq!(SefazService::RetAutorizacao.url_key(), "NfeRetAutorizacao");
        assert_eq!(
            SefazService::ConsultaProtocolo.url_key(),
            "NfeConsultaProtocolo"
        );
        assert_eq!(SefazService::Inutilizacao.url_key(), "NfeInutilizacao");
        assert_eq!(SefazService::RecepcaoEvento.url_key(), "RecepcaoEvento");
    }

    #[test]
    fn meta_versions_are_well_formed() {
        for svc in [
            SefazService::StatusServico,
            SefazService::Autorizacao,
            SefazService::ConsultaProtocolo,
            SefazService::Inutilizacao,
        ] {
            assert_eq!(
                svc.meta().version,
                "4.00",
                "{svc:?} should use version 4.00"
            );
        }
        assert_eq!(SefazService::RecepcaoEvento.meta().version, "1.00");
        assert_eq!(SefazService::DistribuicaoDFe.meta().version, "1.01");
        assert_eq!(SefazService::ConsultaCadastro.meta().version, "2.00");
    }
}
