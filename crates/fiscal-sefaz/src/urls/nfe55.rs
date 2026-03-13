//! NF-e model 55 authorizer endpoint definitions.

use super::{AuthorizerServices, ServiceUrls};

// ── Own authorizers (NF-e model 55) ─────────────────────────────────────────

pub(super) static AM: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

pub(super) static BA: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

pub(super) static GO: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

pub(super) static MG: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

// Fix #1: MS homologation domain corrected from homologacao.nfe.ms.gov.br to hom.nfe.sefaz.ms.gov.br
pub(super) static MS: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

// Fix #2: MT RecepcaoEvento path corrected from NfeRecepcaoEvento4 to RecepcaoEvento4
pub(super) static MT: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

pub(super) static PE: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

pub(super) static PR: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

pub(super) static RS: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

pub(super) static SP: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

// ── SVRS (Sefaz Virtual do RS) ──────────────────────────────────────────────
// Fix #4: SVRS NFC-e URLs corrected to match PHP (lowercase paths for inutilizacao/recepcaoevento,
// NfeConsulta casing)

pub(super) static SVRS: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

// ── SVAN (Sefaz Virtual do Ambiente Nacional) — MA ──────────────────────────

pub(super) static SVAN: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

// ── Fix #5: AN (Ambiente Nacional) ──────────────────────────────────────────

pub(super) static AN: AuthorizerServices = AuthorizerServices {
    // AN does not have these standard services, but we need to fill the struct.
    // Using empty strings that will never be matched since AN is only used for
    // RecepcaoEvento, NfeDistribuicaoDFe, RecepcaoEPEC, NfeConsultaDest, and
    // NfeDownloadNF lookups.
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
    recepcao_epec: Some(ServiceUrls {
        production: "https://www.nfe.fazenda.gov.br/RecepcaoEvento/RecepcaoEvento.asmx",
        homologation: "https://hom.nfe.fazenda.gov.br/RecepcaoEvento/RecepcaoEvento.asmx",
    }),
    epec_status_servico: None,
    nfe_consulta_dest: Some(ServiceUrls {
        production: "https://www.nfe.fazenda.gov.br/NFeConsultaDest/NFeConsultaDest.asmx",
        homologation: "https://hom.nfe.fazenda.gov.br/NFeConsultaDest/NFeConsultaDest.asmx",
    }),
    nfe_download_nf: Some(ServiceUrls {
        production: "https://www.nfe.fazenda.gov.br/NfeDownloadNF/NfeDownloadNF.asmx",
        homologation: "https://hom.nfe.fazenda.gov.br/NfeDownloadNF/NfeDownloadNF.asmx",
    }),
};

// ── Fix #6: SVCAN (SVC-AN) — same URLs as SVAN ─────────────────────────────

pub(super) static SVCAN: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

// ── Fix #6: SVCRS (SVC-RS) — same URLs as SVRS ─────────────────────────────

pub(super) static SVCRS: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};
