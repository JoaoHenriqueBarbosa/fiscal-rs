//! Roteamento: código IBGE do município → provedor.

#![cfg(feature = "client")]

use crate::provider::MunicipalProvider;
use crate::providers::{DSF, SAOPAULO, SIGISS, SIMPLISS};

/// Todos os provedores registrados.
static ALL: &[&'static dyn MunicipalProvider] = &[&DSF, &SIGISS, &SAOPAULO, &SIMPLISS];

/// Resolve o provedor municipal para um código IBGE (7 dígitos). `None` quando
/// nenhum provedor atende — nesse caso o município pode ser **nacional**
/// (emitir pelo SEFIN) ou ainda não suportado.
pub fn resolve(ibge: &str) -> Option<&'static dyn MunicipalProvider> {
    ALL.iter().copied().find(|p| p.municipios().contains(&ibge))
}

/// `true` se há provedor municipal próprio para o IBGE (ou seja, **não** usa o
/// emissor nacional).
pub fn is_municipal(ibge: &str) -> bool {
    resolve(ibge).is_some()
}

/// Para municípios que usam o **layout nacional (DPS)** num endpoint municipal
/// próprio (REST, ex.: Simpliss/Santana de Parnaíba), retorna a URL de POST do
/// ambiente. `None` para municípios ABRASF/próprios (que não usam DPS nacional).
pub fn national_layout_endpoint(ibge: &str, producao: bool) -> Option<&'static str> {
    match ibge {
        // Santana de Parnaíba — Simpliss (`/v2/nfsen`).
        "3547304" => Some(if producao {
            crate::providers::Simpliss::ENDPOINTS.producao
        } else {
            crate::providers::Simpliss::ENDPOINTS.homologacao
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn resolve_conhecidos() {
        assert_eq!(super::resolve("3550308").unwrap().nome(), "SAOPAULO");
        assert_eq!(super::resolve("3552205").unwrap().nome(), "DSF");
        assert_eq!(super::resolve("3513801").unwrap().nome(), "SigISS");
        assert_eq!(super::resolve("3547304").unwrap().nome(), "Simpliss");
        // GINFES removido do registro até homologação validada.
        assert!(super::resolve("3518800").is_none());
        // Rio de Janeiro é nacional → sem provedor municipal.
        assert!(super::resolve("3304557").is_none());
    }
}
