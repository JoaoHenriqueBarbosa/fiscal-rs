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
            _ => return None,
        };
        Some(match env {
            SefazEnvironment::Production => urls.production,
            SefazEnvironment::Homologation => urls.homologation,
            _ => unreachable!(),
        })
    }
}

// ── Own authorizers ─────────────────────────────────────────────────────────

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
};

static MS: AuthorizerServices = AuthorizerServices {
    nfe_status_servico: ServiceUrls {
        production: "https://nfe.sefaz.ms.gov.br/ws/NFeStatusServico4",
        homologation: "https://homologacao.nfe.ms.gov.br/ws/NFeStatusServico4",
    },
    nfe_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.ms.gov.br/ws/NFeAutorizacao4",
        homologation: "https://homologacao.nfe.ms.gov.br/ws/NFeAutorizacao4",
    },
    nfe_ret_autorizacao: ServiceUrls {
        production: "https://nfe.sefaz.ms.gov.br/ws/NFeRetAutorizacao4",
        homologation: "https://homologacao.nfe.ms.gov.br/ws/NFeRetAutorizacao4",
    },
    nfe_consulta_protocolo: ServiceUrls {
        production: "https://nfe.sefaz.ms.gov.br/ws/NFeConsultaProtocolo4",
        homologation: "https://homologacao.nfe.ms.gov.br/ws/NFeConsultaProtocolo4",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfe.sefaz.ms.gov.br/ws/NFeInutilizacao4",
        homologation: "https://homologacao.nfe.ms.gov.br/ws/NFeInutilizacao4",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfe.sefaz.ms.gov.br/ws/NFeRecepcaoEvento4",
        homologation: "https://homologacao.nfe.ms.gov.br/ws/NFeRecepcaoEvento4",
    },
};

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
        production: "https://nfe.sefaz.mt.gov.br/nfews/v2/services/NfeRecepcaoEvento4",
        homologation: "https://homologacao.sefaz.mt.gov.br/nfews/v2/services/NfeRecepcaoEvento4",
    },
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
};

// ── SVRS (Sefaz Virtual do RS) ──────────────────────────────────────────────

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
};

// ── NFC-e authorizers (model 65) ──────────────────────────────────────────

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
};

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
        production: "https://nfce.svrs.rs.gov.br/ws/NfeConsulta/NFeConsulta4.asmx",
        homologation: "https://nfce-homologacao.svrs.rs.gov.br/ws/NfeConsulta/NFeConsulta4.asmx",
    },
    nfe_inutilizacao: ServiceUrls {
        production: "https://nfce.svrs.rs.gov.br/ws/NfeInutilizacao/NFeInutilizacao4.asmx",
        homologation: "https://nfce-homologacao.svrs.rs.gov.br/ws/NfeInutilizacao/NFeInutilizacao4.asmx",
    },
    recepcao_evento: ServiceUrls {
        production: "https://nfce.svrs.rs.gov.br/ws/NfeRecepcaoEvento/NFeRecepcaoEvento4.asmx",
        homologation: "https://nfce-homologacao.svrs.rs.gov.br/ws/NfeRecepcaoEvento/NFeRecepcaoEvento4.asmx",
    },
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
/// `"NfeConsultaProtocolo"`, `"NfeInutilizacao"`, `"RecepcaoEvento"`.
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
/// (PR has its own, other states use SVRS NFC-e).
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

/// Get the NFC-e authorizer for a given state (model 65).
/// PR has its own NFC-e endpoints; all others use SVRS NFC-e.
fn get_state_nfce_authorizer(uf: &str) -> Option<&'static AuthorizerServices> {
    match uf {
        "PR" => Some(&PR_NFCE),
        // All other states use SVRS NFC-e
        "AC" | "AL" | "AM" | "AP" | "BA" | "CE" | "DF" | "ES" | "GO" | "MA" | "MG" | "MS"
        | "MT" | "PA" | "PB" | "PE" | "PI" | "RJ" | "RN" | "RO" | "RR" | "RS" | "SC"
        | "SE" | "SP" | "TO" => Some(&SVRS_NFCE),
        _ => None,
    }
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
        _ => unreachable!(),
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
