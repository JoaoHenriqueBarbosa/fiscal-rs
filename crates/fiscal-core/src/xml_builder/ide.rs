//! Build the `<ide>` (identification) group of the NF-e XML.

use super::tax_id::TaxId;
use crate::types::{InvoiceBuildData, ReferenceDoc};
use crate::xml_utils::{TagContent, tag};

/// Format date/time for NF-e (ISO 8601 with Brazil timezone offset).
///
/// SEFAZ rejects UTC `"Z"` suffix — requires explicit offset like `-03:00`.
pub fn format_datetime_nfe(dt: &chrono::DateTime<chrono::FixedOffset>, state_code: &str) -> String {
    let offset = match state_code {
        "AC" => "-05:00",
        "AM" | "RO" | "RR" | "MT" | "MS" => "-04:00",
        _ => "-03:00",
    };
    format!("{}{offset}", dt.format("%Y-%m-%dT%H:%M:%S"))
}

/// Build the `<ide>` element with all identification fields.
pub(crate) fn build_ide(
    data: &InvoiceBuildData,
    state_ibge: &str,
    numeric_code: &str,
    access_key: &str,
) -> String {
    let ref_elements = build_references(data.references.as_deref());

    let mut children = vec![
        tag("cUF", &[], TagContent::Text(state_ibge)),
        tag("cNF", &[], TagContent::Text(numeric_code)),
        tag("natOp", &[], TagContent::Text(&data.operation_nature)),
        tag("mod", &[], TagContent::Text(data.model.as_str())),
        tag("serie", &[], TagContent::Text(&data.series.to_string())),
        tag("nNF", &[], TagContent::Text(&data.number.to_string())),
        tag(
            "dhEmi",
            &[],
            TagContent::Text(&format_datetime_nfe(
                &data.issued_at,
                &data.issuer.state_code,
            )),
        ),
        tag(
            "tpNF",
            &[],
            TagContent::Text(&data.operation_type.unwrap_or(1).to_string()),
        ),
        tag("idDest", &[], TagContent::Text("1")),
        tag("cMunFG", &[], TagContent::Text(&data.issuer.city_code.0)),
        tag(
            "tpImp",
            &[],
            TagContent::Text(data.print_format.as_deref().unwrap_or("1")),
        ),
        tag("tpEmis", &[], TagContent::Text(data.emission_type.as_str())),
        tag("cDV", &[], TagContent::Text(&access_key[43..44])),
        tag("tpAmb", &[], TagContent::Text(data.environment.as_str())),
        tag(
            "finNFe",
            &[],
            TagContent::Text(&data.purpose_code.unwrap_or(1).to_string()),
        ),
        tag(
            "indFinal",
            &[],
            TagContent::Text(data.consumer_type.as_deref().unwrap_or("0")),
        ),
        tag(
            "indPres",
            &[],
            TagContent::Text(data.buyer_presence.as_deref().unwrap_or("0")),
        ),
        tag(
            "indIntermed",
            &[],
            TagContent::Text(data.intermediary_indicator.as_deref().unwrap_or("0")),
        ),
        tag(
            "procEmi",
            &[],
            TagContent::Text(data.emission_process.as_deref().unwrap_or("0")),
        ),
        tag("verProc", &[], TagContent::Text("FinOpenPOS 1.0")),
    ];

    children.extend(ref_elements);
    tag("ide", &[], TagContent::Children(children))
}

/// Build referenced document elements (`<NFref>` inside `<ide>`).
fn build_references(references: Option<&[ReferenceDoc]>) -> Vec<String> {
    let Some(refs) = references else {
        return vec![];
    };

    refs.iter()
        .map(|r| match r {
            ReferenceDoc::Nfe { access_key } => tag(
                "NFref",
                &[],
                TagContent::Children(vec![tag("refNFe", &[], TagContent::Text(access_key))]),
            ),
            ReferenceDoc::Nf {
                state_code,
                year_month,
                tax_id,
                model,
                series,
                number,
            } => tag(
                "NFref",
                &[],
                TagContent::Children(vec![tag(
                    "refNF",
                    &[],
                    TagContent::Children(vec![
                        tag("cUF", &[], TagContent::Text(&state_code.0)),
                        tag("AAMM", &[], TagContent::Text(year_month)),
                        tag("CNPJ", &[], TagContent::Text(tax_id)),
                        tag("mod", &[], TagContent::Text(model)),
                        tag("serie", &[], TagContent::Text(series)),
                        tag("nNF", &[], TagContent::Text(number)),
                    ]),
                )]),
            ),
            ReferenceDoc::Nfp {
                state_code,
                year_month,
                tax_id,
                model,
                series,
                number,
            } => {
                let tid = TaxId::new(tax_id);
                tag(
                    "NFref",
                    &[],
                    TagContent::Children(vec![tag(
                        "refNFP",
                        &[],
                        TagContent::Children(vec![
                            tag("cUF", &[], TagContent::Text(&state_code.0)),
                            tag("AAMM", &[], TagContent::Text(year_month)),
                            tag(tid.tag_name(), &[], TagContent::Text(tax_id)),
                            tag("mod", &[], TagContent::Text(model)),
                            tag("serie", &[], TagContent::Text(series)),
                            tag("nNF", &[], TagContent::Text(number)),
                        ]),
                    )]),
                )
            }
            ReferenceDoc::Cte { access_key } => tag(
                "NFref",
                &[],
                TagContent::Children(vec![tag("refCTe", &[], TagContent::Text(access_key))]),
            ),
            ReferenceDoc::Ecf {
                model,
                ecf_number,
                coo_number,
            } => tag(
                "NFref",
                &[],
                TagContent::Children(vec![tag(
                    "refECF",
                    &[],
                    TagContent::Children(vec![
                        tag("mod", &[], TagContent::Text(model)),
                        tag("nECF", &[], TagContent::Text(ecf_number)),
                        tag("nCOO", &[], TagContent::Text(coo_number)),
                    ]),
                )]),
            ),
        })
        .collect()
}
