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
            _ => return None,
        };
        Some(match env {
            SefazEnvironment::Production => urls.production,
            SefazEnvironment::Homologation => urls.homologation,
            _ => return None,
        })
    }
}

// ── Own authorizers (NF-e model 55) ─────────────────────────────────────────

static AM: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfe.sefaz.am.gov.br/services2/services/NfeStatusServico4",
        homologation: "https://homnfe.sefaz.am.gov.br/services2/services/NfeStatusServico4",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.am.gov.br/services2/services/NfeAutorizacao4",
        homologation: "https://homnfe.sefaz.am.gov.br/services2/services/NfeAutorizacao4",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.am.gov.br/services2/services/NfeRetAutorizacao4",
        homologation: "https://homnfe.sefaz.am.gov.br/services2/services/NfeRetAutorizacao4",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfe.sefaz.am.gov.br/services2/services/NfeConsulta4",
        homologation: "https://homnfe.sefaz.am.gov.br/services2/services/NfeConsulta4",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfe.sefaz.am.gov.br/services2/services/NfeInutilizacao4",
        homologation: "https://homnfe.sefaz.am.gov.br/services2/services/NfeInutilizacao4",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfe.sefaz.am.gov.br/services2/services/RecepcaoEvento4",
        homologation: "https://homnfe.sefaz.am.gov.br/services2/services/RecepcaoEvento4",
    },
    // PHP has empty NfeConsultaCadastro for AM
    nfe_consulta_cadastro: None,
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

static BA: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfe.sefaz.ba.gov.br/webservices/NFeStatusServico4/NFeStatusServico4.asmx",
        homologation: "https://hnfe.sefaz.ba.gov.br/webservices/NFeStatusServico4/NFeStatusServico4.asmx",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.ba.gov.br/webservices/NFeAutorizacao4/NFeAutorizacao4.asmx",
        homologation: "https://hnfe.sefaz.ba.gov.br/webservices/NFeAutorizacao4/NFeAutorizacao4.asmx",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.ba.gov.br/webservices/NFeRetAutorizacao4/NFeRetAutorizacao4.asmx",
        homologation: "https://hnfe.sefaz.ba.gov.br/webservices/NFeRetAutorizacao4/NFeRetAutorizacao4.asmx",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfe.sefaz.ba.gov.br/webservices/NFeConsultaProtocolo4/NFeConsultaProtocolo4.asmx",
        homologation: "https://hnfe.sefaz.ba.gov.br/webservices/NFeConsultaProtocolo4/NFeConsultaProtocolo4.asmx",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfe.sefaz.ba.gov.br/webservices/NFeInutilizacao4/NFeInutilizacao4.asmx",
        homologation: "https://hnfe.sefaz.ba.gov.br/webservices/NFeInutilizacao4/NFeInutilizacao4.asmx",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfe.sefaz.ba.gov.br/webservices/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx",
        homologation: "https://hnfe.sefaz.ba.gov.br/webservices/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx",
    },
    nfe_consulta_cadastro: Some(ServiceUrls {
        production: "https://nfe.sefaz.ba.gov.br/webservices/CadConsultaCadastro4/CadConsultaCadastro4.asmx",
        homologation: "https://hnfe.sefaz.ba.gov.br/webservices/CadConsultaCadastro4/CadConsultaCadastro4.asmx",
    }),
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

static GO: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfe.sefaz.go.gov.br/nfe/services/NFeStatusServico4",
        homologation: "https://homolog.sefaz.go.gov.br/nfe/services/NFeStatusServico4",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.go.gov.br/nfe/services/NFeAutorizacao4",
        homologation: "https://homolog.sefaz.go.gov.br/nfe/services/NFeAutorizacao4",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.go.gov.br/nfe/services/NFeRetAutorizacao4",
        homologation: "https://homolog.sefaz.go.gov.br/nfe/services/NFeRetAutorizacao4",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfe.sefaz.go.gov.br/nfe/services/NFeConsultaProtocolo4",
        homologation: "https://homolog.sefaz.go.gov.br/nfe/services/NFeConsultaProtocolo4",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfe.sefaz.go.gov.br/nfe/services/NFeInutilizacao4",
        homologation: "https://homolog.sefaz.go.gov.br/nfe/services/NFeInutilizacao4",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfe.sefaz.go.gov.br/nfe/services/NFeRecepcaoEvento4",
        homologation: "https://homolog.sefaz.go.gov.br/nfe/services/NFeRecepcaoEvento4",
    },
    nfe_consulta_cadastro: Some(ServiceUrls {
        production: "https://nfe.sefaz.go.gov.br/nfe/services/CadConsultaCadastro4",
        homologation: "https://homolog.sefaz.go.gov.br/nfe/services/CadConsultaCadastro4",
    }),
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

static MG: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfe.fazenda.mg.gov.br/nfe2/services/NFeStatusServico4",
        homologation: "https://hnfe.fazenda.mg.gov.br/nfe2/services/NFeStatusServico4",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfe.fazenda.mg.gov.br/nfe2/services/NFeAutorizacao4",
        homologation: "https://hnfe.fazenda.mg.gov.br/nfe2/services/NFeAutorizacao4",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfe.fazenda.mg.gov.br/nfe2/services/NFeRetAutorizacao4",
        homologation: "https://hnfe.fazenda.mg.gov.br/nfe2/services/NFeRetAutorizacao4",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfe.fazenda.mg.gov.br/nfe2/services/NFeConsultaProtocolo4",
        homologation: "https://hnfe.fazenda.mg.gov.br/nfe2/services/NFeConsultaProtocolo4",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfe.fazenda.mg.gov.br/nfe2/services/NFeInutilizacao4",
        homologation: "https://hnfe.fazenda.mg.gov.br/nfe2/services/NFeInutilizacao4",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfe.fazenda.mg.gov.br/nfe2/services/NFeRecepcaoEvento4",
        homologation: "https://hnfe.fazenda.mg.gov.br/nfe2/services/NFeRecepcaoEvento4",
    },
    nfe_consulta_cadastro: Some(ServiceUrls {
        production: "https://nfe.fazenda.mg.gov.br/nfe2/services/CadConsultaCadastro4",
        homologation: "https://hnfe.fazenda.mg.gov.br/nfe2/services/CadConsultaCadastro4",
    }),
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

// Fix #1: MS homologation domain corrected from homologacao.nfe.ms.gov.br to hom.nfe.sefaz.ms.gov.br
static MS: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfe.sefaz.ms.gov.br/ws/NFeStatusServico4",
        homologation: "https://hom.nfe.sefaz.ms.gov.br/ws/NFeStatusServico4",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.ms.gov.br/ws/NFeAutorizacao4",
        homologation: "https://hom.nfe.sefaz.ms.gov.br/ws/NFeAutorizacao4",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.ms.gov.br/ws/NFeRetAutorizacao4",
        homologation: "https://hom.nfe.sefaz.ms.gov.br/ws/NFeRetAutorizacao4",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfe.sefaz.ms.gov.br/ws/NFeConsultaProtocolo4",
        homologation: "https://hom.nfe.sefaz.ms.gov.br/ws/NFeConsultaProtocolo4",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfe.sefaz.ms.gov.br/ws/NFeInutilizacao4",
        homologation: "https://hom.nfe.sefaz.ms.gov.br/ws/NFeInutilizacao4",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfe.sefaz.ms.gov.br/ws/NFeRecepcaoEvento4",
        homologation: "https://hom.nfe.sefaz.ms.gov.br/ws/NFeRecepcaoEvento4",
    },
    nfe_consulta_cadastro: Some(ServiceUrls {
        production: "https://nfe.sefaz.ms.gov.br/ws/CadConsultaCadastro4",
        homologation: "https://hom.nfe.sefaz.ms.gov.br/ws/CadConsultaCadastro4",
    }),
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

// Fix #2: MT RecepcaoEvento path corrected from NfeRecepcaoEvento4 to RecepcaoEvento4
static MT: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfe.sefaz.mt.gov.br/nfews/v2/services/NfeStatusServico4",
        homologation: "https://homologacao.sefaz.mt.gov.br/nfews/v2/services/NfeStatusServico4",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.mt.gov.br/nfews/v2/services/NfeAutorizacao4",
        homologation: "https://homologacao.sefaz.mt.gov.br/nfews/v2/services/NfeAutorizacao4",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.mt.gov.br/nfews/v2/services/NfeRetAutorizacao4",
        homologation: "https://homologacao.sefaz.mt.gov.br/nfews/v2/services/NfeRetAutorizacao4",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfe.sefaz.mt.gov.br/nfews/v2/services/NfeConsulta4",
        homologation: "https://homologacao.sefaz.mt.gov.br/nfews/v2/services/NfeConsulta4",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfe.sefaz.mt.gov.br/nfews/v2/services/NfeInutilizacao4",
        homologation: "https://homologacao.sefaz.mt.gov.br/nfews/v2/services/NfeInutilizacao4",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfe.sefaz.mt.gov.br/nfews/v2/services/RecepcaoEvento4",
        homologation: "https://homologacao.sefaz.mt.gov.br/nfews/v2/services/RecepcaoEvento4",
    },
    nfe_consulta_cadastro: Some(ServiceUrls {
        production: "https://nfe.sefaz.mt.gov.br/nfews/v2/services/CadConsultaCadastro4",
        homologation: "https://homologacao.sefaz.mt.gov.br/nfews/v2/services/CadConsultaCadastro4",
    }),
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

static PE: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfe.sefaz.pe.gov.br/nfe-service/services/NFeStatusServico4",
        homologation: "https://nfehomolog.sefaz.pe.gov.br/nfe-service/services/NFeStatusServico4",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.pe.gov.br/nfe-service/services/NFeAutorizacao4",
        homologation: "https://nfehomolog.sefaz.pe.gov.br/nfe-service/services/NFeAutorizacao4",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.pe.gov.br/nfe-service/services/NFeRetAutorizacao4",
        homologation: "https://nfehomolog.sefaz.pe.gov.br/nfe-service/services/NFeRetAutorizacao4",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfe.sefaz.pe.gov.br/nfe-service/services/NFeConsultaProtocolo4",
        homologation: "https://nfehomolog.sefaz.pe.gov.br/nfe-service/services/NFeConsultaProtocolo4",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfe.sefaz.pe.gov.br/nfe-service/services/NFeInutilizacao4",
        homologation: "https://nfehomolog.sefaz.pe.gov.br/nfe-service/services/NFeInutilizacao4",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfe.sefaz.pe.gov.br/nfe-service/services/NFeRecepcaoEvento4",
        homologation: "https://nfehomolog.sefaz.pe.gov.br/nfe-service/services/NFeRecepcaoEvento4",
    },
    nfe_consulta_cadastro: Some(ServiceUrls {
        production: "https://nfe.sefaz.pe.gov.br/nfe-service/services/CadConsultaCadastro4",
        homologation: "https://nfehomolog.sefaz.pe.gov.br/nfe-service/services/CadConsultaCadastro4",
    }),
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

static PR: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfe.sefa.pr.gov.br/nfe/NFeStatusServico4",
        homologation: "https://homologacao.nfe.sefa.pr.gov.br/nfe/NFeStatusServico4",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfe.sefa.pr.gov.br/nfe/NFeAutorizacao4",
        homologation: "https://homologacao.nfe.sefa.pr.gov.br/nfe/NFeAutorizacao4",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfe.sefa.pr.gov.br/nfe/NFeRetAutorizacao4",
        homologation: "https://homologacao.nfe.sefa.pr.gov.br/nfe/NFeRetAutorizacao4",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfe.sefa.pr.gov.br/nfe/NFeConsultaProtocolo4",
        homologation: "https://homologacao.nfe.sefa.pr.gov.br/nfe/NFeConsultaProtocolo4",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfe.sefa.pr.gov.br/nfe/NFeInutilizacao4",
        homologation: "https://homologacao.nfe.sefa.pr.gov.br/nfe/NFeInutilizacao4",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfe.sefa.pr.gov.br/nfe/NFeRecepcaoEvento4",
        homologation: "https://homologacao.nfe.sefa.pr.gov.br/nfe/NFeRecepcaoEvento4",
    },
    nfe_consulta_cadastro: Some(ServiceUrls {
        production: "https://nfe.sefa.pr.gov.br/nfe/CadConsultaCadastro4",
        homologation: "https://homologacao.nfe.sefa.pr.gov.br/nfe/CadConsultaCadastro4",
    }),
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

static RS: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfe.sefazrs.rs.gov.br/ws/NfeStatusServico/NfeStatusServico4.asmx",
        homologation: "https://nfe-homologacao.sefazrs.rs.gov.br/ws/NfeStatusServico/NfeStatusServico4.asmx",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfe.sefazrs.rs.gov.br/ws/NfeAutorizacao/NFeAutorizacao4.asmx",
        homologation: "https://nfe-homologacao.sefazrs.rs.gov.br/ws/NfeAutorizacao/NFeAutorizacao4.asmx",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfe.sefazrs.rs.gov.br/ws/NfeRetAutorizacao/NFeRetAutorizacao4.asmx",
        homologation: "https://nfe-homologacao.sefazrs.rs.gov.br/ws/NfeRetAutorizacao/NFeRetAutorizacao4.asmx",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfe.sefazrs.rs.gov.br/ws/NfeConsulta/NfeConsulta4.asmx",
        homologation: "https://nfe-homologacao.sefazrs.rs.gov.br/ws/NfeConsulta/NfeConsulta4.asmx",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfe.sefazrs.rs.gov.br/ws/nfeinutilizacao/nfeinutilizacao4.asmx",
        homologation: "https://nfe-homologacao.sefazrs.rs.gov.br/ws/nfeinutilizacao/nfeinutilizacao4.asmx",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfe.sefazrs.rs.gov.br/ws/recepcaoevento/recepcaoevento4.asmx",
        homologation: "https://nfe-homologacao.sefazrs.rs.gov.br/ws/recepcaoevento/recepcaoevento4.asmx",
    },
    nfe_consulta_cadastro: Some(ServiceUrls {
        production: "https://cad.svrs.rs.gov.br/ws/cadconsultacadastro/cadconsultacadastro4.asmx",
        homologation: "https://cad.svrs.rs.gov.br/ws/cadconsultacadastro/cadconsultacadastro4.asmx",
    }),
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

static SP: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfe.fazenda.sp.gov.br/ws/nfestatusservico4.asmx",
        homologation: "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfestatusservico4.asmx",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfe.fazenda.sp.gov.br/ws/nfeautorizacao4.asmx",
        homologation: "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfeautorizacao4.asmx",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfe.fazenda.sp.gov.br/ws/nferetautorizacao4.asmx",
        homologation: "https://homologacao.nfe.fazenda.sp.gov.br/ws/nferetautorizacao4.asmx",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfe.fazenda.sp.gov.br/ws/nfeconsultaprotocolo4.asmx",
        homologation: "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfeconsultaprotocolo4.asmx",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfe.fazenda.sp.gov.br/ws/nfeinutilizacao4.asmx",
        homologation: "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfeinutilizacao4.asmx",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfe.fazenda.sp.gov.br/ws/nferecepcaoevento4.asmx",
        homologation: "https://homologacao.nfe.fazenda.sp.gov.br/ws/nferecepcaoevento4.asmx",
    },
    nfe_consulta_cadastro: Some(ServiceUrls {
        production: "https://nfe.fazenda.sp.gov.br/ws/cadconsultacadastro4.asmx",
        homologation: "https://homologacao.nfe.fazenda.sp.gov.br/ws/cadconsultacadastro4.asmx",
    }),
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

// ── SVRS (Sefaz Virtual do RS) ──────────────────────────────────────────────
// Fix #4: SVRS NFC-e URLs corrected to match PHP (lowercase paths for inutilizacao/recepcaoevento,
// NfeConsulta casing)

static SVRS: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfe.svrs.rs.gov.br/ws/NfeStatusServico/NfeStatusServico4.asmx",
        homologation: "https://nfe-homologacao.svrs.rs.gov.br/ws/NfeStatusServico/NfeStatusServico4.asmx",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfe.svrs.rs.gov.br/ws/NfeAutorizacao/NFeAutorizacao4.asmx",
        homologation: "https://nfe-homologacao.svrs.rs.gov.br/ws/NfeAutorizacao/NFeAutorizacao4.asmx",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfe.svrs.rs.gov.br/ws/NfeRetAutorizacao/NFeRetAutorizacao4.asmx",
        homologation: "https://nfe-homologacao.svrs.rs.gov.br/ws/NfeRetAutorizacao/NFeRetAutorizacao4.asmx",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfe.svrs.rs.gov.br/ws/NfeConsulta/NfeConsulta4.asmx",
        homologation: "https://nfe-homologacao.svrs.rs.gov.br/ws/NfeConsulta/NfeConsulta4.asmx",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfe.svrs.rs.gov.br/ws/nfeinutilizacao/nfeinutilizacao4.asmx",
        homologation: "https://nfe-homologacao.svrs.rs.gov.br/ws/nfeinutilizacao/nfeinutilizacao4.asmx",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfe.svrs.rs.gov.br/ws/recepcaoevento/recepcaoevento4.asmx",
        homologation: "https://nfe-homologacao.svrs.rs.gov.br/ws/recepcaoevento/recepcaoevento4.asmx",
    },
    nfe_consulta_cadastro: Some(ServiceUrls {
        production: "https://cad.svrs.rs.gov.br/ws/cadconsultacadastro/cadconsultacadastro4.asmx",
        homologation: "https://cad-homologacao.svrs.rs.gov.br/ws/cadconsultacadastro/cadconsultacadastro4.asmx",
    }),
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

// ── SVAN (Sefaz Virtual do Ambiente Nacional) — MA ──────────────────────────

static SVAN: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://www.sefazvirtual.fazenda.gov.br/NFeStatusServico4/NFeStatusServico4.asmx",
        homologation: "https://hom.sefazvirtual.fazenda.gov.br/NFeStatusServico4/NFeStatusServico4.asmx",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://www.sefazvirtual.fazenda.gov.br/NFeAutorizacao4/NFeAutorizacao4.asmx",
        homologation: "https://hom.sefazvirtual.fazenda.gov.br/NFeAutorizacao4/NFeAutorizacao4.asmx",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://www.sefazvirtual.fazenda.gov.br/NFeRetAutorizacao4/NFeRetAutorizacao4.asmx",
        homologation: "https://hom.sefazvirtual.fazenda.gov.br/NFeRetAutorizacao4/NFeRetAutorizacao4.asmx",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://www.sefazvirtual.fazenda.gov.br/NFeConsultaProtocolo4/NFeConsultaProtocolo4.asmx",
        homologation: "https://hom.sefazvirtual.fazenda.gov.br/NFeConsultaProtocolo4/NFeConsultaProtocolo4.asmx",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://www.sefazvirtual.fazenda.gov.br/NFeInutilizacao4/NFeInutilizacao4.asmx",
        homologation: "https://hom.sefazvirtual.fazenda.gov.br/NFeInutilizacao4/NFeInutilizacao4.asmx",
    },
    recepcao_evento: ServiceUrls {
        production: "https://www.sefazvirtual.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx",
        homologation: "https://hom.sefazvirtual.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx",
    },
    // SVAN has no NfeConsultaCadastro (empty in PHP)
    nfe_consulta_cadastro: None,
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

// ── Fix #5: AN (Ambiente Nacional) ──────────────────────────────────────────

static AN: AuthorizerServices = AuthorizerServices {
    // AN does not have these standard services, but we need to fill the struct.
    // Using empty strings that will never be matched since AN is only used for
    // RecepcaoEvento and NfeDistribuicaoDFe lookups.
    nfe_status_servico: ServiceUrls {
        production: "",
        homologation: "",
    },
    nfe_autorizacao: ServiceUrls {
        production: "",
        homologation: "",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "",
        homologation: "",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "",
        homologation: "",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "",
        homologation: "",
    },
    recepcao_evento: ServiceUrls {
        production: "https://www.nfe.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx",
        homologation: "https://hom1.nfe.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx",
    },
    nfe_consulta_cadastro: None,
    nfe_distribuicao_dfe: Some(ServiceUrls {
        production: "https://www1.nfe.fazenda.gov.br/NFeDistribuicaoDFe/NFeDistribuicaoDFe.asmx",
        homologation: "https://hom1.nfe.fazenda.gov.br/NFeDistribuicaoDFe/NFeDistribuicaoDFe.asmx",
    }),
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

// ── Fix #6: SVCAN (SVC-AN) — same URLs as SVAN ─────────────────────────────

static SVCAN: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://www.sefazvirtual.fazenda.gov.br/NFeStatusServico4/NFeStatusServico4.asmx",
        homologation: "https://hom.sefazvirtual.fazenda.gov.br/NFeStatusServico4/NFeStatusServico4.asmx",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://www.sefazvirtual.fazenda.gov.br/NFeAutorizacao4/NFeAutorizacao4.asmx",
        homologation: "https://hom.sefazvirtual.fazenda.gov.br/NFeAutorizacao4/NFeAutorizacao4.asmx",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://www.sefazvirtual.fazenda.gov.br/NFeRetAutorizacao4/NFeRetAutorizacao4.asmx",
        homologation: "https://hom.sefazvirtual.fazenda.gov.br/NFeRetAutorizacao4/NFeRetAutorizacao4.asmx",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://www.sefazvirtual.fazenda.gov.br/NFeConsultaProtocolo4/NFeConsultaProtocolo4.asmx",
        homologation: "https://hom.sefazvirtual.fazenda.gov.br/NFeConsultaProtocolo4/NFeConsultaProtocolo4.asmx",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://www.sefazvirtual.fazenda.gov.br/NFeInutilizacao4/NFeInutilizacao4.asmx",
        homologation: "https://hom.sefazvirtual.fazenda.gov.br/NFeInutilizacao4/NFeInutilizacao4.asmx",
    },
    recepcao_evento: ServiceUrls {
        production: "https://www.sefazvirtual.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx",
        homologation: "https://hom.sefazvirtual.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx",
    },
    nfe_consulta_cadastro: None,
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

// ── Fix #6: SVCRS (SVC-RS) — same URLs as SVRS ─────────────────────────────

static SVCRS: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfe.svrs.rs.gov.br/ws/NfeStatusServico/NfeStatusServico4.asmx",
        homologation: "https://nfe-homologacao.svrs.rs.gov.br/ws/NfeStatusServico/NfeStatusServico4.asmx",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfe.svrs.rs.gov.br/ws/NfeAutorizacao/NFeAutorizacao4.asmx",
        homologation: "https://nfe-homologacao.svrs.rs.gov.br/ws/NfeAutorizacao/NFeAutorizacao4.asmx",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfe.svrs.rs.gov.br/ws/NfeRetAutorizacao/NFeRetAutorizacao4.asmx",
        homologation: "https://nfe-homologacao.svrs.rs.gov.br/ws/NfeRetAutorizacao/NFeRetAutorizacao4.asmx",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfe.svrs.rs.gov.br/ws/NfeConsulta/NfeConsulta4.asmx",
        homologation: "https://nfe-homologacao.svrs.rs.gov.br/ws/NfeConsulta/NfeConsulta4.asmx",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfe.svrs.rs.gov.br/ws/nfeinutilizacao/nfeinutilizacao4.asmx",
        homologation: "https://nfe-homologacao.svrs.rs.gov.br/ws/nfeinutilizacao/nfeinutilizacao4.asmx",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfe.svrs.rs.gov.br/ws/recepcaoevento/recepcaoevento4.asmx",
        homologation: "https://nfe-homologacao.svrs.rs.gov.br/ws/recepcaoevento/recepcaoevento4.asmx",
    },
    nfe_consulta_cadastro: Some(ServiceUrls {
        production: "https://cad.svrs.rs.gov.br/ws/cadconsultacadastro/cadconsultacadastro4.asmx",
        homologation: "",
    }),
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

// ── NFC-e authorizers (model 65) ──────────────────────────────────────────

// Fix #3: Add dedicated NFC-e endpoints for AM, GO, MG, MS, MT, RS, SP

static AM_NFCE: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfce.sefaz.am.gov.br/nfce-services/services/NfeStatusServico4",
        homologation: "https://homnfce.sefaz.am.gov.br/nfce-services/services/NfeStatusServico4",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfce.sefaz.am.gov.br/nfce-services/services/NfeAutorizacao4",
        homologation: "https://homnfce.sefaz.am.gov.br/nfce-services/services/NfeAutorizacao4",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfce.sefaz.am.gov.br/nfce-services/services/NfeRetAutorizacao4",
        homologation: "https://homnfce.sefaz.am.gov.br/nfce-services/services/NfeRetAutorizacao4",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfce.sefaz.am.gov.br/nfce-services/services/NfeConsulta4",
        homologation: "https://homnfce.sefaz.am.gov.br/nfce-services/services/NfeConsulta4",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfce.sefaz.am.gov.br/nfce-services/services/NfeInutilizacao4",
        homologation: "https://homnfce.sefaz.am.gov.br/nfce-services/services/NfeInutilizacao4",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfce.sefaz.am.gov.br/nfce-services/services/RecepcaoEvento4",
        homologation: "https://homnfce.sefaz.am.gov.br/nfce-services/services/RecepcaoEvento4",
    },
    nfe_consulta_cadastro: None,
    nfe_distribuicao_dfe: None,
    csc_nfce: Some(ServiceUrls {
        production: "https://nfce.sefaz.am.gov.br/nfce-services/services/CscNFCe",
        homologation: "https://homnfce.sefaz.am.gov.br/nfce-services/services/CscNFCe",
    }),
    recepcao_epec: None,
    epec_status_servico: None,
};

static GO_NFCE: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfe.sefaz.go.gov.br/nfe/services/NFeStatusServico4",
        homologation: "https://homolog.sefaz.go.gov.br/nfe/services/NFeStatusServico4",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.go.gov.br/nfe/services/NFeAutorizacao4",
        homologation: "https://homolog.sefaz.go.gov.br/nfe/services/NFeAutorizacao4",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.go.gov.br/nfe/services/NFeRetAutorizacao4",
        homologation: "https://homolog.sefaz.go.gov.br/nfe/services/NFeRetAutorizacao4",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfe.sefaz.go.gov.br/nfe/services/NFeConsultaProtocolo4",
        homologation: "https://homolog.sefaz.go.gov.br/nfe/services/NFeConsultaProtocolo4",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfe.sefaz.go.gov.br/nfe/services/NFeInutilizacao4",
        homologation: "https://homolog.sefaz.go.gov.br/nfe/services/NFeInutilizacao4",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfe.sefaz.go.gov.br/nfe/services/NFeRecepcaoEvento4",
        homologation: "https://homolog.sefaz.go.gov.br/nfe/services/NFeRecepcaoEvento4",
    },
    nfe_consulta_cadastro: None,
    nfe_distribuicao_dfe: None,
    csc_nfce: Some(ServiceUrls {
        production: "https://nfe.sefaz.go.gov.br/nfe/services/v2/CscNFCe",
        homologation: "https://homolog.sefaz.go.gov.br/nfe/services/v2/CscNFCe",
    }),
    recepcao_epec: None,
    epec_status_servico: None,
};

static MG_NFCE: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfce.fazenda.mg.gov.br/nfce/services/NFeStatusServico4",
        homologation: "https://hnfce.fazenda.mg.gov.br/nfce/services/NFeStatusServico4",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfce.fazenda.mg.gov.br/nfce/services/NFeAutorizacao4",
        homologation: "https://hnfce.fazenda.mg.gov.br/nfce/services/NFeAutorizacao4",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfce.fazenda.mg.gov.br/nfce/services/NFeRetAutorizacao4",
        homologation: "https://hnfce.fazenda.mg.gov.br/nfce/services/NFeRetAutorizacao4",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfce.fazenda.mg.gov.br/nfce/services/NFeConsultaProtocolo4",
        homologation: "https://hnfce.fazenda.mg.gov.br/nfce/services/NFeConsultaProtocolo4",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfce.fazenda.mg.gov.br/nfce/services/NFeInutilizacao4",
        homologation: "https://hnfce.fazenda.mg.gov.br/nfce/services/NFeInutilizacao4",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfce.fazenda.mg.gov.br/nfce/services/NFeRecepcaoEvento4",
        homologation: "https://hnfce.fazenda.mg.gov.br/nfce/services/NFeRecepcaoEvento4",
    },
    nfe_consulta_cadastro: None,
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

static MS_NFCE: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfce.sefaz.ms.gov.br/ws/NFeStatusServico4",
        homologation: "https://hom.nfce.sefaz.ms.gov.br/ws/NFeStatusServico4",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfce.sefaz.ms.gov.br/ws/NFeAutorizacao4",
        homologation: "https://hom.nfce.sefaz.ms.gov.br/ws/NFeAutorizacao4",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfce.sefaz.ms.gov.br/ws/NFeRetAutorizacao4",
        homologation: "https://hom.nfce.sefaz.ms.gov.br/ws/NFeRetAutorizacao4",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfce.sefaz.ms.gov.br/ws/NFeConsultaProtocolo4",
        homologation: "https://hom.nfce.sefaz.ms.gov.br/ws/NFeConsultaProtocolo4",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfce.sefaz.ms.gov.br/ws/NFeInutilizacao4",
        homologation: "https://hom.nfce.sefaz.ms.gov.br/ws/NFeInutilizacao4",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfce.sefaz.ms.gov.br/ws/NFeRecepcaoEvento4",
        homologation: "https://hom.nfce.sefaz.ms.gov.br/ws/NFeRecepcaoEvento4",
    },
    nfe_consulta_cadastro: None,
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

static MT_NFCE: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfce.sefaz.mt.gov.br/nfcews/services/NfeStatusServico4",
        homologation: "https://homologacao.sefaz.mt.gov.br/nfcews/services/NfeStatusServico4",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfce.sefaz.mt.gov.br/nfcews/services/NfeAutorizacao4",
        homologation: "https://homologacao.sefaz.mt.gov.br/nfcews/services/NfeAutorizacao4",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfce.sefaz.mt.gov.br/nfcews/services/NfeRetAutorizacao4",
        homologation: "https://homologacao.sefaz.mt.gov.br/nfcews/services/NfeRetAutorizacao4",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfce.sefaz.mt.gov.br/nfcews/services/NfeConsulta4",
        homologation: "https://homologacao.sefaz.mt.gov.br/nfcews/services/NfeConsulta4",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfce.sefaz.mt.gov.br/nfcews/services/NfeInutilizacao4",
        homologation: "https://homologacao.sefaz.mt.gov.br/nfcews/services/NfeInutilizacao4",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfce.sefaz.mt.gov.br/nfcews/services/RecepcaoEvento4",
        homologation: "https://homologacao.sefaz.mt.gov.br/nfcews/services/RecepcaoEvento4",
    },
    nfe_consulta_cadastro: None,
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

static RS_NFCE: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfce.sefazrs.rs.gov.br/ws/NfeStatusServico/NfeStatusServico4.asmx",
        homologation: "https://nfce-homologacao.sefazrs.rs.gov.br/ws/NfeStatusServico/NfeStatusServico4.asmx",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfce.sefazrs.rs.gov.br/ws/NfeAutorizacao/NFeAutorizacao4.asmx",
        homologation: "https://nfce-homologacao.sefazrs.rs.gov.br/ws/NfeAutorizacao/NFeAutorizacao4.asmx",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfce.sefazrs.rs.gov.br/ws/NfeRetAutorizacao/NFeRetAutorizacao4.asmx",
        homologation: "https://nfce-homologacao.sefazrs.rs.gov.br/ws/NfeRetAutorizacao/NFeRetAutorizacao4.asmx",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfce.sefazrs.rs.gov.br/ws/NfeConsulta/NfeConsulta4.asmx",
        homologation: "https://nfce-homologacao.sefazrs.rs.gov.br/ws/NfeConsulta/NfeConsulta4.asmx",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfce.sefazrs.rs.gov.br/ws/nfeinutilizacao/nfeinutilizacao4.asmx",
        homologation: "https://nfce-homologacao.sefazrs.rs.gov.br/ws/nfeinutilizacao/nfeinutilizacao4.asmx",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfce.sefazrs.rs.gov.br/ws/recepcaoevento/recepcaoevento4.asmx",
        homologation: "https://nfce-homologacao.sefazrs.rs.gov.br/ws/recepcaoevento/recepcaoevento4.asmx",
    },
    nfe_consulta_cadastro: None,
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

static SP_NFCE: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfce.fazenda.sp.gov.br/ws/NFeStatusServico4.asmx",
        homologation: "https://homologacao.nfce.fazenda.sp.gov.br/ws/NFeStatusServico4.asmx",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfce.fazenda.sp.gov.br/ws/NFeAutorizacao4.asmx",
        homologation: "https://homologacao.nfce.fazenda.sp.gov.br/ws/NFeAutorizacao4.asmx",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfce.fazenda.sp.gov.br/ws/NFeRetAutorizacao4.asmx",
        homologation: "https://homologacao.nfce.fazenda.sp.gov.br/ws/NFeRetAutorizacao4.asmx",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfce.fazenda.sp.gov.br/ws/NFeConsultaProtocolo4.asmx",
        homologation: "https://homologacao.nfce.fazenda.sp.gov.br/ws/NFeConsultaProtocolo4.asmx",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfce.fazenda.sp.gov.br/ws/NFeInutilizacao4.asmx",
        homologation: "https://homologacao.nfce.fazenda.sp.gov.br/ws/NFeInutilizacao4.asmx",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfce.fazenda.sp.gov.br/ws/NFeRecepcaoEvento4.asmx",
        homologation: "https://homologacao.nfce.fazenda.sp.gov.br/ws/NFeRecepcaoEvento4.asmx",
    },
    nfe_consulta_cadastro: None,
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: Some(ServiceUrls {
        production: "https://nfce.epec.fazenda.sp.gov.br/EPECws/RecepcaoEPEC.asmx",
        homologation: "https://homologacao.nfce.epec.fazenda.sp.gov.br/EPECws/RecepcaoEPEC.asmx",
    }),
    epec_status_servico: Some(ServiceUrls {
        production: "https://nfce.epec.fazenda.sp.gov.br/EPECws/EPECStatusServico.asmx",
        homologation: "https://homologacao.nfce.epec.fazenda.sp.gov.br/EPECws/EPECStatusServico.asmx",
    }),
};

static PR_NFCE: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfce.sefa.pr.gov.br/nfce/NFeStatusServico4",
        homologation: "https://homologacao.nfce.sefa.pr.gov.br/nfce/NFeStatusServico4",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfce.sefa.pr.gov.br/nfce/NFeAutorizacao4",
        homologation: "https://homologacao.nfce.sefa.pr.gov.br/nfce/NFeAutorizacao4",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfce.sefa.pr.gov.br/nfce/NFeRetAutorizacao4",
        homologation: "https://homologacao.nfce.sefa.pr.gov.br/nfce/NFeRetAutorizacao4",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfce.sefa.pr.gov.br/nfce/NFeConsultaProtocolo4",
        homologation: "https://homologacao.nfce.sefa.pr.gov.br/nfce/NFeConsultaProtocolo4",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfce.sefa.pr.gov.br/nfce/NFeInutilizacao4",
        homologation: "https://homologacao.nfce.sefa.pr.gov.br/nfce/NFeInutilizacao4",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfce.sefa.pr.gov.br/nfce/NFeRecepcaoEvento4",
        homologation: "https://homologacao.nfce.sefa.pr.gov.br/nfce/NFeRecepcaoEvento4",
    },
    nfe_consulta_cadastro: None,
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

// Fix #4: SVRS NFC-e URLs corrected to match PHP exactly
static SVRS_NFCE: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfce.svrs.rs.gov.br/ws/NfeStatusServico/NfeStatusServico4.asmx",
        homologation: "https://nfce-homologacao.svrs.rs.gov.br/ws/NfeStatusServico/NfeStatusServico4.asmx",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfce.svrs.rs.gov.br/ws/NfeAutorizacao/NFeAutorizacao4.asmx",
        homologation: "https://nfce-homologacao.svrs.rs.gov.br/ws/NfeAutorizacao/NFeAutorizacao4.asmx",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfce.svrs.rs.gov.br/ws/NfeRetAutorizacao/NFeRetAutorizacao4.asmx",
        homologation: "https://nfce-homologacao.svrs.rs.gov.br/ws/NfeRetAutorizacao/NFeRetAutorizacao4.asmx",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfce.svrs.rs.gov.br/ws/NfeConsulta/NfeConsulta4.asmx",
        homologation: "https://nfce-homologacao.svrs.rs.gov.br/ws/NfeConsulta/NfeConsulta4.asmx",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfce.svrs.rs.gov.br/ws/nfeinutilizacao/nfeinutilizacao4.asmx",
        homologation: "https://nfce-homologacao.svrs.rs.gov.br/ws/nfeinutilizacao/nfeinutilizacao4.asmx",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfce.svrs.rs.gov.br/ws/recepcaoevento/recepcaoevento4.asmx",
        homologation: "https://nfce-homologacao.svrs.rs.gov.br/ws/recepcaoevento/recepcaoevento4.asmx",
    },
    nfe_consulta_cadastro: None,
    nfe_distribuicao_dfe: None,
    csc_nfce: None,
    recepcao_epec: None,
    epec_status_servico: None,
};

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
/// `"RecepcaoEPEC"` (SP NFC-e only), `"EPECStatusServico"` (SP NFC-e only).
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
/// AN provides RecepcaoEvento and NfeDistribuicaoDFe services.
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

// ── NFC-e consultation URIs (urlChave) ──────────────────────────────────────

/// Get the NFC-e consultation URL (urlChave) for a given state and environment.
///
/// Returns the base URL used for DANFCE consultation links.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a valid Brazilian
/// state abbreviation.
pub fn get_nfce_consult_url(
    uf: &str,
    environment: SefazEnvironment,
) -> Result<String, FiscalError> {
    let url = match environment {
        SefazEnvironment::Production => match uf {
            "AC" => "www.sefaznet.ac.gov.br/nfce/consulta",
            "AL" => "www.sefaz.al.gov.br/nfce/consulta",
            "AP" => "www.sefaz.ap.gov.br/nfce/consulta",
            "AM" => "www.sefaz.am.gov.br/nfce/consulta",
            "BA" => "http://www.sefaz.ba.gov.br/nfce/consulta",
            "CE" => "www.sefaz.ce.gov.br/nfce/consulta",
            "DF" => "www.fazenda.df.gov.br/nfce/consulta",
            "ES" => "www.sefaz.es.gov.br/nfce/consulta",
            "GO" => "www.sefaz.go.gov.br/nfce/consulta",
            "MA" => "www.sefaz.ma.gov.br/nfce/consulta",
            "MG" => "https://portalsped.fazenda.mg.gov.br/portalnfce",
            "MS" => "http://www.dfe.ms.gov.br/nfce/consulta",
            "MT" => "http://www.sefaz.mt.gov.br/nfce/consultanfce",
            "PA" => "www.sefa.pa.gov.br/nfce/consulta",
            "PB" => "www.sefaz.pb.gov.br/nfce/consulta",
            "PE" => "nfce.sefaz.pe.gov.br/nfce/consulta",
            "PR" => "http://www.fazenda.pr.gov.br/nfce/consulta",
            "PI" => "www.sefaz.pi.gov.br/nfce/consulta",
            "RJ" => "www.fazenda.rj.gov.br/nfce/consulta",
            "RN" => "www.set.rn.gov.br/nfce/consulta",
            "RO" => "www.sefin.ro.gov.br/nfce/consulta",
            "RR" => "www.sefaz.rr.gov.br/nfce/consulta",
            "RS" => "www.sefaz.rs.gov.br/nfce/consulta",
            "SC" => "https://sat.sef.sc.gov.br/nfce/consulta",
            "SE" => "http://www.nfce.se.gov.br/nfce/consulta",
            "SP" => "https://www.nfce.fazenda.sp.gov.br/NFCeConsultaPublica",
            "TO" => "www.sefaz.to.gov.br/nfce/consulta",
            _ => return Err(FiscalError::InvalidStateCode(uf.to_string())),
        },
        SefazEnvironment::Homologation => match uf {
            "AC" => "www.sefaznet.ac.gov.br/nfce/consulta",
            "AL" => "www.sefaz.al.gov.br/nfce/consulta",
            "AP" => "www.sefaz.ap.gov.br/nfce/consulta",
            "AM" => "www.sefaz.am.gov.br/nfce/consulta",
            "BA" => "http://hinternet.sefaz.ba.gov.br/nfce/consulta",
            "CE" => "www.sefaz.ce.gov.br/nfce/consulta",
            "DF" => "www.fazenda.df.gov.br/nfce/consulta",
            "ES" => "www.sefaz.es.gov.br/nfce/consulta",
            "GO" => "www.nfce.go.gov.br/post/ver/214413/consulta-nfc-e-homologacao",
            "MA" => "www.sefaz.ma.gov.br/nfce/consulta",
            "MG" => "https://hportalsped.fazenda.mg.gov.br/portalnfce",
            "MS" => "http://www.dfe.ms.gov.br/nfce/consulta",
            "MT" => "http://homologacao.sefaz.mt.gov.br/nfce/consultanfce",
            "PA" => "www.sefa.pa.gov.br/nfce/consulta",
            "PB" => "www.sefaz.pb.gov.br/nfcehom",
            "PE" => "nfce.sefaz.pe.gov.br/nfce/consulta",
            "PR" => "http://www.fazenda.pr.gov.br/nfce/consulta",
            "PI" => "www.sefaz.pi.gov.br/nfce/consulta",
            "RJ" => "www.fazenda.rj.gov.br/nfce/consulta",
            "RN" => "www.set.rn.gov.br/nfce/consulta",
            "RO" => "www.sefin.ro.gov.br/nfce/consulta",
            "RR" => "www.sefaz.rr.gov.br/nfce/consulta",
            "RS" => "www.sefaz.rs.gov.br/nfce/consulta",
            "SC" => "https://hom.sat.sef.sc.gov.br/nfce/consulta",
            "SE" => "http://www.hom.nfe.se.gov.br/nfce/consulta",
            "SP" => "https://www.homologacao.nfce.fazenda.sp.gov.br/NFCeConsultaPublica",
            "TO" => "http://homologacao.sefaz.to.gov.br/nfce/consulta.jsf",
            _ => return Err(FiscalError::InvalidStateCode(uf.to_string())),
        },
        _ => return Err(FiscalError::InvalidStateCode(uf.to_string())),
    };
    Ok(url.to_string())
}

/// Get the NFC-e QR Code base URL for a given state and environment.
///
/// This returns the same consultation URL used for QR Code generation.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a valid Brazilian
/// state abbreviation.
pub fn get_nfce_qr_url(uf: &str, environment: SefazEnvironment) -> Result<String, FiscalError> {
    get_nfce_consult_url(uf, environment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sp_nfce_recepcao_epec_production() {
        let url = get_sefaz_url_for_model("SP", SefazEnvironment::Production, "RecepcaoEPEC", 65)
            .unwrap();
        assert_eq!(
            url,
            "https://nfce.epec.fazenda.sp.gov.br/EPECws/RecepcaoEPEC.asmx"
        );
    }

    #[test]
    fn sp_nfce_recepcao_epec_homologation() {
        let url = get_sefaz_url_for_model("SP", SefazEnvironment::Homologation, "RecepcaoEPEC", 65)
            .unwrap();
        assert_eq!(
            url,
            "https://homologacao.nfce.epec.fazenda.sp.gov.br/EPECws/RecepcaoEPEC.asmx"
        );
    }

    #[test]
    fn sp_nfce_epec_status_servico_production() {
        let url =
            get_sefaz_url_for_model("SP", SefazEnvironment::Production, "EPECStatusServico", 65)
                .unwrap();
        assert_eq!(
            url,
            "https://nfce.epec.fazenda.sp.gov.br/EPECws/EPECStatusServico.asmx"
        );
    }

    #[test]
    fn sp_nfce_epec_status_servico_homologation() {
        let url = get_sefaz_url_for_model(
            "SP",
            SefazEnvironment::Homologation,
            "EPECStatusServico",
            65,
        )
        .unwrap();
        assert_eq!(
            url,
            "https://homologacao.nfce.epec.fazenda.sp.gov.br/EPECws/EPECStatusServico.asmx"
        );
    }

    #[test]
    fn epec_services_not_available_for_other_states() {
        // EPEC NFC-e services are SP-only; other states should return an error.
        let result =
            get_sefaz_url_for_model("MG", SefazEnvironment::Production, "RecepcaoEPEC", 65);
        assert!(result.is_err());

        let result =
            get_sefaz_url_for_model("MG", SefazEnvironment::Production, "EPECStatusServico", 65);
        assert!(result.is_err());
    }

    #[test]
    fn epec_services_not_available_for_nfe_model_55() {
        // EPEC NFC-e services should not be available on NF-e model 55.
        let result =
            get_sefaz_url_for_model("SP", SefazEnvironment::Production, "RecepcaoEPEC", 55);
        assert!(result.is_err());

        let result =
            get_sefaz_url_for_model("SP", SefazEnvironment::Production, "EPECStatusServico", 55);
        assert!(result.is_err());
    }
}
