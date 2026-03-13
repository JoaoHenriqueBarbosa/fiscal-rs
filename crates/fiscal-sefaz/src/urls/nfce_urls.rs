//! NFC-e consultation (urlChave) and QR Code URL tables.

use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;

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
/// Returns the URL used for NFC-e QR Code generation (`NfeConsultaQR`).
/// This is **different** from the consultation URL (`urlChave`) returned by
/// [`get_nfce_consult_url`].
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a valid Brazilian
/// state abbreviation.
pub fn get_nfce_qr_url(uf: &str, environment: SefazEnvironment) -> Result<String, FiscalError> {
    let url = match environment {
        SefazEnvironment::Production => match uf {
            "AC" => "http://www.sefaznet.ac.gov.br/nfce/qrcode",
            "AL" => "http://nfce.sefaz.al.gov.br/QRCode/consultarNFCe.jsp",
            "AM" => "https://sistemas.sefaz.am.gov.br/nfceweb/consultarNFCe.jsp",
            "AP" => "https://www.sefaz.ap.gov.br/nfce/nfcep.php",
            "BA" => "http://nfe.sefaz.ba.gov.br/servicos/nfce/qrcode.aspx",
            "CE" => "http://nfce.sefaz.ce.gov.br/pages/ShowNFCe.html",
            "DF" => "http://www.fazenda.df.gov.br/nfce/qrcode",
            "ES" => "http://app.sefaz.es.gov.br/ConsultaNFCe/qrcode.aspx",
            "GO" => "https://nfeweb.sefaz.go.gov.br/nfeweb/sites/nfce/danfeNFCe",
            "MA" => "http://www.nfce.sefaz.ma.gov.br/portal/consultarNFCe.jsp",
            "MG" => "https://portalsped.fazenda.mg.gov.br/portalnfce/sistema/qrcode.xhtml",
            "MS" => "http://www.dfe.ms.gov.br/nfce/qrcode",
            "MT" => "http://www.sefaz.mt.gov.br/nfce/consultanfce",
            "PA" => "https://appnfc.sefa.pa.gov.br/portal/view/consultas/nfce/nfceForm.seam",
            "PB" => "http://www.sefaz.pb.gov.br/nfce",
            "PE" => "http://nfce.sefaz.pe.gov.br/nfce/consulta",
            "PI" => "http://www.sefaz.pi.gov.br/nfce/qrcode",
            "PR" => "http://www.fazenda.pr.gov.br/nfce/qrcode",
            "RJ" => "https://consultadfe.fazenda.rj.gov.br/consultaNFCe/QRCode",
            "RN" => "http://nfce.set.rn.gov.br/consultarNFCe.aspx",
            "RO" => "http://www.nfce.sefin.ro.gov.br/consultanfce/consulta.jsp",
            "RR" => "https://www.sefaz.rr.gov.br/servlet/qrcode",
            "RS" => "https://www.sefaz.rs.gov.br/NFCE/NFCE-COM.aspx",
            "SC" => "https://sat.sef.sc.gov.br/nfce/consulta",
            "SE" => "http://www.nfce.se.gov.br/nfce/qrcode",
            "SP" => "https://www.nfce.fazenda.sp.gov.br/qrcode",
            "TO" => "http://www.sefaz.to.gov.br/nfce/qrcode",
            _ => return Err(FiscalError::InvalidStateCode(uf.to_string())),
        },
        SefazEnvironment::Homologation => match uf {
            "AC" => "http://www.hml.sefaznet.ac.gov.br/nfce/qrcode",
            "AL" => "http://nfce.sefaz.al.gov.br/QRCode/consultarNFCe.jsp",
            "AM" => "https://sistemas.sefaz.am.gov.br/nfceweb-hom/consultarNFCe.jsp",
            "AP" => "https://www.sefaz.ap.gov.br/nfcehml/nfce.php",
            "BA" => "http://hnfe.sefaz.ba.gov.br/servicos/nfce/qrcode.aspx",
            "CE" => "http://nfceh.sefaz.ce.gov.br/pages/ShowNFCe.html",
            "DF" => "http://dec.fazenda.df.gov.br/ConsultarNFCe.aspx",
            "ES" => "http://homologacao.sefaz.es.gov.br/ConsultaNFCe/qrcode.aspx",
            "GO" => "https://nfewebhomolog.sefaz.go.gov.br/nfeweb/sites/nfce/danfeNFCe",
            "MA" => "http://www.hom.nfce.sefaz.ma.gov.br/portal/consultarNFCe.jsp",
            "MG" => "https://portalsped.fazenda.mg.gov.br/portalnfce/sistema/qrcode.xhtml",
            "MS" => "http://www.dfe.ms.gov.br/nfce/qrcode",
            "MT" => "http://homologacao.sefaz.mt.gov.br/nfce/consultanfce",
            "PA" => {
                "https://appnfc.sefa.pa.gov.br/portal-homologacao/view/consultas/nfce/nfceForm.seam"
            }
            "PB" => "http://www.sefaz.pb.gov.br/nfcehom",
            "PE" => "http://nfcehomolog.sefaz.pe.gov.br/nfce/consulta",
            "PI" => "http://www.sefaz.pi.gov.br/nfce/qrcode",
            "PR" => "http://www.fazenda.pr.gov.br/nfce/qrcode",
            "RJ" => "http://www4.fazenda.rj.gov.br/consultaNFCe/QRCode",
            "RN" => "http://hom.nfce.set.rn.gov.br/consultarNFCe.aspx",
            "RO" => "http://www.nfce.sefin.ro.gov.br/consultanfce/consulta.jsp",
            "RR" => "http://200.174.88.103:8080/nfce/servlet/qrcode",
            "RS" => "https://www.sefaz.rs.gov.br/NFCE/NFCE-COM.aspx",
            "SC" => "https://hom.sat.sef.sc.gov.br/nfce/consulta",
            "SE" => "http://www.hom.nfe.se.gov.br/nfce/qrcode",
            "SP" => "https://www.homologacao.nfce.fazenda.sp.gov.br/qrcode",
            "TO" => "http://homologacao.sefaz.to.gov.br/nfce/qrcode",
            _ => return Err(FiscalError::InvalidStateCode(uf.to_string())),
        },
        _ => return Err(FiscalError::InvalidStateCode(uf.to_string())),
    };
    Ok(url.to_string())
}
