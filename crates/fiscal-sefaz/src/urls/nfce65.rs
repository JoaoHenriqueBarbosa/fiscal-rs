//! NFC-e model 65 authorizer endpoint definitions.

use super::{AuthorizerServices, ServiceUrls};

// ── NFC-e authorizers (model 65) ──────────────────────────────────────────

// Fix #3: Add dedicated NFC-e endpoints for AM, GO, MG, MS, MT, RS, SP

pub(super) static AM_NFCE: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

pub(super) static GO_NFCE: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

pub(super) static MG_NFCE: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

pub(super) static MS_NFCE: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

pub(super) static MT_NFCE: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

pub(super) static RS_NFCE: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

pub(super) static SP_NFCE: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

pub(super) static PR_NFCE: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};

// Fix #4: SVRS NFC-e URLs corrected to match PHP exactly
pub(super) static SVRS_NFCE: AuthorizerServices = AuthorizerServices {
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
    nfe_consulta_dest: None,
    nfe_download_nf: None,
};
