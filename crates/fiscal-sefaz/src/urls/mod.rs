use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;

/// Production and homologation URLs for a SEFAZ web service.
struct ServiceUrls {
    production: &'static str,
    homologation: &'static str,
}

/// All SEFAZ services for a given authorizer.
struct AuthorizerServices {
    nfe_status_servico: ServiceUrls,
    nfe_autorizacao: ServiceUrls,
    nfe_ret_autorizacao: ServiceUrls,
    nfe_consulta_protocolo: ServiceUrls,
    nfe_inutilizacao: ServiceUrls,
    recepcao_evento: ServiceUrls,
    nfe_consulta_cadastro: Option<ServiceUrls>,
    nfe_distribuicao_dfe: Option<ServiceUrls>,
    csc_nfce: Option<ServiceUrls>,
    recepcao_epec: Option<ServiceUrls>,
    epec_status_servico: Option<ServiceUrls>,
    nfe_consulta_dest: Option<ServiceUrls>,
    nfe_download_nf: Option<ServiceUrls>,
}

impl AuthorizerServices {
    /// Resolve the URL for a given service name and environment.
    fn get_url(&self, service: &str, env: SefazEnvironment) -> Option<&'static str> {
        let urls = match service {
            "NfeStatusServico" => &self.nfe_status_servico,
            "NfeAutorizacao" => &self.nfe_autorizacao,
            "NfeRetAutorizacao" => &self.nfe_ret_autorizacao,
            "NfeConsultaProtocolo" => &self.nfe_consulta_protocolo,
            "NfeInutilizacao" => &self.nfe_inutilizacao,
            "RecepcaoEvento" => &self.recepcao_evento,
            "NfeConsultaCadastro" => {
                return self.nfe_consulta_cadastro.as_ref().and_then(|u| {
                    let url = match env {
                        SefazEnvironment::Production => u.production,
                        SefazEnvironment::Homologation => u.homologation,
                        _ => return None,
                    };
                    if url.is_empty() { None } else { Some(url) }
                });
            }
            "NfeDistribuicaoDFe" => {
                return self.nfe_distribuicao_dfe.as_ref().and_then(|u| {
                    let url = match env {
                        SefazEnvironment::Production => u.production,
                        SefazEnvironment::Homologation => u.homologation,
                        _ => return None,
                    };
                    if url.is_empty() { None } else { Some(url) }
                });
            }
            "CscNFCe" => {
                return self.csc_nfce.as_ref().and_then(|u| {
                    let url = match env {
                        SefazEnvironment::Production => u.production,
                        SefazEnvironment::Homologation => u.homologation,
                        _ => return None,
                    };
                    if url.is_empty() { None } else { Some(url) }
                });
            }
            "RecepcaoEPEC" => {
                return self.recepcao_epec.as_ref().and_then(|u| {
                    let url = match env {
                        SefazEnvironment::Production => u.production,
                        SefazEnvironment::Homologation => u.homologation,
                        _ => return None,
                    };
                    if url.is_empty() { None } else { Some(url) }
                });
            }
            "EPECStatusServico" => {
                return self.epec_status_servico.as_ref().and_then(|u| {
                    let url = match env {
                        SefazEnvironment::Production => u.production,
                        SefazEnvironment::Homologation => u.homologation,
                        _ => return None,
                    };
                    if url.is_empty() { None } else { Some(url) }
                });
            }
            "NfeConsultaDest" => {
                return self.nfe_consulta_dest.as_ref().and_then(|u| {
                    let url = match env {
                        SefazEnvironment::Production => u.production,
                        SefazEnvironment::Homologation => u.homologation,
                        _ => return None,
                    };
                    if url.is_empty() { None } else { Some(url) }
                });
            }
            "NfeDownloadNF" => {
                return self.nfe_download_nf.as_ref().and_then(|u| {
                    let url = match env {
                        SefazEnvironment::Production => u.production,
                        SefazEnvironment::Homologation => u.homologation,
                        _ => return None,
                    };
                    if url.is_empty() { None } else { Some(url) }
                });
            }
            _ => return None,
        };
        Some(match env {
            SefazEnvironment::Production => urls.production,
            SefazEnvironment::Homologation => urls.homologation,
            _ => return None,
        })
    }
}

mod nfce65;
mod nfce_urls;
mod nfe55;

pub use nfce_urls::{get_nfce_consult_url, get_nfce_qr_url};
use nfce65::*;
use nfe55::*;

/// Get the authorizer for a given state (NF-e model 55).
fn get_state_authorizer(uf: &str) -> Option<&'static AuthorizerServices> {
    match uf {
        "AM" => Some(&AM),
        "BA" => Some(&BA),
        "GO" => Some(&GO),
        "MG" => Some(&MG),
        "MS" => Some(&MS),
        "MT" => Some(&MT),
        "PE" => Some(&PE),
        "PR" => Some(&PR),
        "RS" => Some(&RS),
        "SP" => Some(&SP),
        // SVAN
        "MA" => Some(&SVAN),
        // SVRS (all remaining states)
        "AC" | "AL" | "AP" | "CE" | "DF" | "ES" | "PA" | "PB" | "PI" | "RJ" | "RN" | "RO"
        | "RR" | "SC" | "SE" | "TO" => Some(&SVRS),
        _ => None,
    }
}

/// Get the SEFAZ service URL for a given state, environment, service name,
/// and invoice model (55 for NF-e, 65 for NFC-e).
///
/// The `service` parameter must be one of:
/// `"NfeStatusServico"`, `"NfeAutorizacao"`, `"NfeRetAutorizacao"`,
/// `"NfeConsultaProtocolo"`, `"NfeInutilizacao"`, `"RecepcaoEvento"`,
/// `"NfeConsultaCadastro"`, `"NfeDistribuicaoDFe"`, `"CscNFCe"`,
/// `"RecepcaoEPEC"` (SP NFC-e only), `"EPECStatusServico"` (SP NFC-e only),
/// `"NfeConsultaDest"` (AN only), `"NfeDownloadNF"` (AN only).
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a valid Brazilian
/// state abbreviation, or [`FiscalError::XmlGeneration`] if the service name
/// is unknown.
pub fn get_sefaz_url(
    uf: &str,
    environment: SefazEnvironment,
    service: &str,
) -> Result<String, FiscalError> {
    get_sefaz_url_for_model(uf, environment, service, 55)
}

/// Get the SEFAZ service URL for a specific invoice model.
///
/// Use model `55` for NF-e and `65` for NFC-e. NFC-e uses dedicated endpoints
/// for AM, GO, MG, MS, MT, PR, RS, SP; other states use SVRS NFC-e.
pub fn get_sefaz_url_for_model(
    uf: &str,
    environment: SefazEnvironment,
    service: &str,
    model: u8,
) -> Result<String, FiscalError> {
    let authorizer = if model == 65 {
        get_state_nfce_authorizer(uf)
    } else {
        get_state_authorizer(uf)
    }
    .ok_or_else(|| FiscalError::InvalidStateCode(uf.to_string()))?;

    authorizer
        .get_url(service, environment)
        .map(|s| s.to_string())
        .ok_or_else(|| {
            FiscalError::XmlGeneration(format!("Service '{service}' not found for state {uf}"))
        })
}

/// Get the Ambiente Nacional (AN) service URL.
///
/// AN provides RecepcaoEvento, NfeDistribuicaoDFe, RecepcaoEPEC,
/// NfeConsultaDest, and NfeDownloadNF services.
pub fn get_an_url(environment: SefazEnvironment, service: &str) -> Result<String, FiscalError> {
    AN.get_url(service, environment)
        .map(|s| s.to_string())
        .ok_or_else(|| FiscalError::XmlGeneration(format!("Service '{service}' not found for AN")))
}

/// Get the NFC-e authorizer for a given state (model 65).
/// AM, GO, MG, MS, MT, PR, RS, SP have their own NFC-e endpoints;
/// all others use SVRS NFC-e.
fn get_state_nfce_authorizer(uf: &str) -> Option<&'static AuthorizerServices> {
    match uf {
        "AM" => Some(&AM_NFCE),
        "GO" => Some(&GO_NFCE),
        "MG" => Some(&MG_NFCE),
        "MS" => Some(&MS_NFCE),
        "MT" => Some(&MT_NFCE),
        "PR" => Some(&PR_NFCE),
        "RS" => Some(&RS_NFCE),
        "SP" => Some(&SP_NFCE),
        // All other states use SVRS NFC-e
        "AC" | "AL" | "AP" | "BA" | "CE" | "DF" | "ES" | "MA" | "PA" | "PB" | "PE" | "PI"
        | "RJ" | "RN" | "RO" | "RR" | "SC" | "SE" | "TO" => Some(&SVRS_NFCE),
        _ => None,
    }
}

/// Get the contingency authorizer for a given state (SVC-AN or SVC-RS).
///
/// Mapping follows the PHP sped-nfe Contingency.php:
/// - SVC-AN (SVCAN): AC, AL, AP, CE, DF, ES, MG, PA, PB, PI, RJ, RN, RO, RR, RS, SC, SE, SP, TO
/// - SVC-RS (SVCRS): AM, BA, GO, MA, MS, MT, PE, PR
pub fn get_state_contingency_authorizer(uf: &str) -> Option<&'static str> {
    match uf {
        "AC" | "AL" | "AP" | "CE" | "DF" | "ES" | "MG" | "PA" | "PB" | "PI" | "RJ" | "RN"
        | "RO" | "RR" | "RS" | "SC" | "SE" | "SP" | "TO" => Some("SVCAN"),
        "AM" | "BA" | "GO" | "MA" | "MS" | "MT" | "PE" | "PR" => Some("SVCRS"),
        _ => None,
    }
}

/// Get the SEFAZ contingency service URL for a given state and environment.
///
/// Resolves the contingency authorizer (SVCAN or SVCRS) for the state and
/// returns the appropriate service URL.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a valid Brazilian
/// state abbreviation or has no contingency mapping.
pub fn get_sefaz_contingency_url(
    uf: &str,
    environment: SefazEnvironment,
    service: &str,
) -> Result<String, FiscalError> {
    let contingency_type = get_state_contingency_authorizer(uf)
        .ok_or_else(|| FiscalError::InvalidStateCode(uf.to_string()))?;

    let authorizer = match contingency_type {
        "SVCAN" => &SVCAN,
        "SVCRS" => &SVCRS,
        _ => return Err(FiscalError::InvalidStateCode(uf.to_string())),
    };

    authorizer
        .get_url(service, environment)
        .map(|s| s.to_string())
        .ok_or_else(|| {
            FiscalError::XmlGeneration(format!(
                "Service '{service}' not found for contingency {contingency_type}"
            ))
        })
}

#[cfg(test)]
mod tests;
