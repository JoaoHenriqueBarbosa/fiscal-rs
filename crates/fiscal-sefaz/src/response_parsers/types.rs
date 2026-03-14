//! Typed result structs for all SEFAZ response parsers.

use serde::{Deserialize, Serialize};

/// Parsed result of a SEFAZ NF-e authorization (`retEnviNFe`) response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AuthorizationResponse {
    /// SEFAZ status code (`cStat`).
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// Protocol number (`nProt`), present when the NF-e was processed.
    pub protocol_number: Option<String>,
    /// Raw `<protNFe>...</protNFe>` XML fragment for storage/attachment.
    pub protocol_xml: Option<String>,
    /// Timestamp when SEFAZ received/authorized the document (`dhRecbto`).
    pub authorized_at: Option<String>,
    /// Receipt number (`nRec`), present for asynchronous batch submissions
    /// (`indSinc=0`). Use this with [`SefazClient::consult_receipt`] to
    /// poll for the batch processing result.
    pub receipt_number: Option<String>,
}

/// Parsed result of a SEFAZ service status (`retConsStatServ`) response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StatusResponse {
    /// SEFAZ status code (`cStat`).
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// Average processing time in seconds (`tMed`).
    pub average_time: Option<String>,
}

/// Parsed result of a SEFAZ cancellation event (`retEvento`) response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CancellationResponse {
    /// SEFAZ status code (`cStat`).
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// Protocol number (`nProt`), present when the event was registered.
    pub protocol_number: Option<String>,
}

/// Parsed result of a SEFAZ DistDFe (`retDistDFeInt`) response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DistDFeResponse {
    /// SEFAZ status code (`cStat`).
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// Last NSU returned (`ultNSU`).
    pub ult_nsu: Option<String>,
    /// Maximum NSU available (`maxNSU`).
    pub max_nsu: Option<String>,
    /// Raw XML of individual `<docZip>` or `<loteDistDFeInt>` entries.
    pub raw_xml: String,
}

/// Parsed result of a SEFAZ inutilização (`retInutNFe`) response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct InutilizacaoResponse {
    /// Environment type (`tpAmb`): 1 = Produção, 2 = Homologação.
    pub tp_amb: String,
    /// SEFAZ application version (`verAplic`).
    pub ver_aplic: String,
    /// SEFAZ status code (`cStat`).
    pub c_stat: String,
    /// Human-readable status message (`xMotivo`).
    pub x_motivo: String,
    /// UF code (`cUF`).
    pub c_uf: String,
    /// Year of the inutilização (`ano`).
    pub ano: String,
    /// CNPJ of the emitter (may be empty if CPF was used).
    pub cnpj: String,
    /// CPF of the emitter (for MT and other states that use CPF instead of CNPJ).
    pub cpf: Option<String>,
    /// Fiscal document model (`mod`).
    pub modelo: String,
    /// Series number (`serie`).
    pub serie: String,
    /// Initial NF-e number (`nNFIni`).
    pub n_nf_ini: String,
    /// Final NF-e number (`nNFFin`).
    pub n_nf_fin: String,
    /// Timestamp when SEFAZ received the request (`dhRecbto`).
    pub dh_recbto: Option<String>,
    /// Protocol number (`nProt`).
    pub n_prot: Option<String>,
}

/// A single CSC token (id + secret) from the NFC-e CSC administration response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CscToken {
    /// CSC identifier (`idCsc`).
    pub id_csc: String,
    /// CSC secret value (`CSC`).
    pub csc: String,
}

/// Parsed result of a SEFAZ NFC-e CSC administration (`retAdmCscNFCe`) response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CscResponse {
    /// Environment type (`tpAmb`): 1 = production, 2 = homologation.
    pub tp_amb: String,
    /// Operation indicator (`indOp`): 1 = consulta, 2 = novo, 3 = revogar.
    pub ind_op: String,
    /// SEFAZ status code (`cStat`).
    pub c_stat: String,
    /// Human-readable status message (`xMotivo`).
    pub x_motivo: String,
    /// Active CSC tokens (`idCsc` + `CSC` pairs), present for indOp 1 or 2.
    pub tokens: Vec<CscToken>,
}

/// Parsed result of a SEFAZ Cadastro (`retConsCad`) response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CadastroResponse {
    /// SEFAZ status code (`cStat`).
    pub status_code: String,
    /// Human-readable status message (`xMotivo`).
    pub status_message: String,
    /// Raw inner XML of `<infCons>` for detailed parsing.
    pub raw_xml: String,
}

/// A single protocol from a `retConsReciNFe` response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ProtocolInfo {
    /// Environment type (`tpAmb`).
    pub tp_amb: String,
    /// SEFAZ application version (`verAplic`).
    pub ver_aplic: String,
    /// NF-e access key (`chNFe`).
    pub ch_nfe: String,
    /// Timestamp when SEFAZ received the document (`dhRecbto`).
    pub dh_recbto: Option<String>,
    /// Protocol number (`nProt`).
    pub n_prot: Option<String>,
    /// Digest value (`digVal`).
    pub dig_val: Option<String>,
    /// SEFAZ status code (`cStat`).
    pub c_stat: String,
    /// Human-readable status message (`xMotivo`).
    pub x_motivo: String,
}

/// Parsed result of a SEFAZ consulta recibo (`retConsReciNFe`) response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ConsultaReciboResponse {
    /// Environment type (`tpAmb`): 1 = Produção, 2 = Homologação.
    pub tp_amb: String,
    /// SEFAZ application version (`verAplic`).
    pub ver_aplic: String,
    /// Receipt number (`nRec`).
    pub n_rec: String,
    /// SEFAZ status code (`cStat`).
    pub c_stat: String,
    /// Human-readable status message (`xMotivo`).
    pub x_motivo: String,
    /// UF code (`cUF`).
    pub c_uf: String,
    /// Protocols for each NF-e in the batch.
    pub protocols: Vec<ProtocolInfo>,
}

/// Parsed result of a SEFAZ consulta situação (`retConsSitNFe`) response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ConsultaSituacaoResponse {
    /// Environment type (`tpAmb`): 1 = Produção, 2 = Homologação.
    pub tp_amb: String,
    /// SEFAZ application version (`verAplic`).
    pub ver_aplic: String,
    /// SEFAZ status code (`cStat`).
    pub c_stat: String,
    /// Human-readable status message (`xMotivo`).
    pub x_motivo: String,
    /// UF code (`cUF`).
    pub c_uf: String,
    /// NF-e access key (`chNFe`).
    pub ch_nfe: Option<String>,
    /// Protocol XML (`<protNFe>...</protNFe>`), present when the NF-e exists.
    pub protocol_xml: Option<String>,
    /// Raw event XML fragments (`<retEvento>...</retEvento>`), if events exist.
    pub event_xmls: Vec<String>,
}
