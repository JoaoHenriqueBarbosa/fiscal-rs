use super::*;

#[test]
fn minimal_ibs_cbs_xml() {
    let data = IbsCbsData::new("00", "12345678");
    let xml = build_ibs_cbs_xml(&data);
    assert!(xml.contains("<IBSCBS>"));
    assert!(xml.contains("<CST>00</CST>"));
    assert!(xml.contains("<cClassTrib>12345678</cClassTrib>"));
    assert!(xml.contains("</IBSCBS>"));
    assert!(!xml.contains("<indDoacao>"));
}

#[test]
fn ibs_cbs_with_ad_valorem() {
    let g_ibs_uf = GIbsUfData::new("18.0000", "180.00");
    let g_ibs_mun = GIbsMunData::new("5.0000", "50.00");
    let g_cbs = GCbsData::new("9.0000", "90.00");
    let g = GIbsCbsData::new("1000.00", g_ibs_uf, g_ibs_mun, g_cbs).v_ibs("230.00");
    let data = IbsCbsData::new("00", "12345678")
        .ind_doacao(true)
        .g_ibs_cbs(g);
    let xml = build_ibs_cbs_xml(&data);
    assert!(xml.contains("<indDoacao>1</indDoacao>"));
    assert!(xml.contains("<gIBSCBS>"));
    assert!(xml.contains("<vBC>1000.00</vBC>"));
    assert!(xml.contains("<pIBSUF>18.0000</pIBSUF>"));
    assert!(xml.contains("<vIBSUF>180.00</vIBSUF>"));
    assert!(xml.contains("<pIBSMun>5.0000</pIBSMun>"));
    assert!(xml.contains("<vIBSMun>50.00</vIBSMun>"));
    assert!(xml.contains("<vIBS>230.00</vIBS>"));
    assert!(xml.contains("<pCBS>9.0000</pCBS>"));
    assert!(xml.contains("<vCBS>90.00</vCBS>"));
    assert!(xml.contains("</gIBSCBS>"));
}

#[test]
fn ibs_cbs_with_diferimento() {
    let g_ibs_uf = GIbsUfData::new("18.0000", "162.00").g_dif(GDifData::new("10.0000", "18.00"));
    let g_ibs_mun = GIbsMunData::new("5.0000", "50.00");
    let g_cbs = GCbsData::new("9.0000", "90.00");
    let g = GIbsCbsData::new("1000.00", g_ibs_uf, g_ibs_mun, g_cbs).v_ibs("212.00");
    let data = IbsCbsData::new("00", "12345678").g_ibs_cbs(g);
    let xml = build_ibs_cbs_xml(&data);
    assert!(xml.contains("<gDif><pDif>10.0000</pDif><vDif>18.00</vDif></gDif>"));
}

#[test]
fn ibs_cbs_monofasico() {
    let mono = GIbsCbsMonoData::new("15.00", "10.00").g_mono_padrao(GMonoPadraoData::new(
        "100.0000", "0.1500", "0.1000", "15.00", "10.00",
    ));
    let data = IbsCbsData::new("02", "87654321").g_ibs_cbs_mono(mono);
    let xml = build_ibs_cbs_xml(&data);
    assert!(xml.contains("<gIBSCBSMono>"));
    assert!(xml.contains("<gMonoPadrao>"));
    assert!(xml.contains("<qBCMono>100.0000</qBCMono>"));
    assert!(xml.contains("<vTotIBSMonoItem>15.00</vTotIBSMonoItem>"));
    assert!(xml.contains("<vTotCBSMonoItem>10.00</vTotCBSMonoItem>"));
}

#[test]
fn ibs_cbs_transf_cred() {
    let data =
        IbsCbsData::new("03", "11111111").g_transf_cred(GTransfCredData::new("50.00", "30.00"));
    let xml = build_ibs_cbs_xml(&data);
    assert!(xml.contains("<gTransfCred>"));
    assert!(xml.contains("<vIBS>50.00</vIBS>"));
    assert!(xml.contains("<vCBS>30.00</vCBS>"));
}

#[test]
fn ibs_cbs_estorno_cred() {
    let data =
        IbsCbsData::new("04", "22222222").g_estorno_cred(GEstornoCredData::new("10.00", "5.00"));
    let xml = build_ibs_cbs_xml(&data);
    assert!(xml.contains("<gEstornoCred>"));
    assert!(xml.contains("<vIBSEstCred>10.00</vIBSEstCred>"));
    assert!(xml.contains("<vCBSEstCred>5.00</vCBSEstCred>"));
}

#[test]
fn ibs_cbs_cred_pres_oper() {
    let data = IbsCbsData::new("05", "33333333").g_cred_pres_oper(
        GCredPresOperData::new("500.00", "01")
            .g_ibs_cred_pres(GIbsCredPresData::with_cred_pres("2.5000", "12.50"))
            .g_cbs_cred_pres(GCbsCredPresData::with_cred_pres_cond_sus("1.5000", "7.50")),
    );
    let xml = build_ibs_cbs_xml(&data);
    assert!(xml.contains("<gCredPresOper>"));
    assert!(xml.contains("<vBCCredPres>500.00</vBCCredPres>"));
    assert!(xml.contains("<gIBSCredPres>"));
    assert!(xml.contains("<vCredPres>12.50</vCredPres>"));
    assert!(xml.contains("<gCBSCredPres>"));
    assert!(xml.contains("<vCredPresCondSus>7.50</vCredPresCondSus>"));
}

#[test]
fn is_tot_xml() {
    let data = IsTotData::new("15.00");
    let xml = build_is_tot_xml(&data);
    assert_eq!(xml, "<ISTot><vIS>15.00</vIS></ISTot>");
}

#[test]
fn ibs_cbs_tot_minimal() {
    let data = IbsCbsTotData::new("1000.00");
    let xml = build_ibs_cbs_tot_xml(&data);
    assert!(xml.contains("<IBSCBSTot>"));
    assert!(xml.contains("<vBCIBSCBS>1000.00</vBCIBSCBS>"));
    // No gIBS/gCBS/gMono without data
    assert!(!xml.contains("<gIBS>"));
    assert!(!xml.contains("<gCBS>"));
    assert!(!xml.contains("<gMono>"));
}

#[test]
fn ibs_cbs_tot_with_gibs_and_gcbs() {
    let mut data = IbsCbsTotData::new("1000.00");
    data.g_ibs_v_ibs = Some("230.00".into());
    data.g_ibs_uf_v_ibs_uf = Some("180.00".into());
    data.g_ibs_mun_v_ibs_mun = Some("50.00".into());
    data.g_cbs_v_cbs = Some("90.00".into());
    let xml = build_ibs_cbs_tot_xml(&data);
    assert!(xml.contains("<gIBS>"));
    assert!(xml.contains("<vIBSUF>180.00</vIBSUF>"));
    assert!(xml.contains("<vIBSMun>50.00</vIBSMun>"));
    assert!(xml.contains("<vIBS>230.00</vIBS>"));
    assert!(xml.contains("<gCBS>"));
    assert!(xml.contains("<vCBS>90.00</vCBS>"));
}

#[test]
fn trib_regular_xml() {
    let data = IbsCbsData::new("00", "12345678").g_trib_regular(GTribRegularData::new(
        "01", "99999999", "18.0000", "180.00", "5.0000", "50.00", "9.0000", "90.00",
    ));
    let xml = build_ibs_cbs_xml(&data);
    assert!(xml.contains("<gTribRegular>"));
    assert!(xml.contains("<CSTReg>01</CSTReg>"));
    assert!(xml.contains("<pAliqEfetRegCBS>9.0000</pAliqEfetRegCBS>"));
    assert!(xml.contains("<vTribRegCBS>90.00</vTribRegCBS>"));
}

#[test]
fn trib_compra_gov_xml() {
    let data = IbsCbsData::new("00", "12345678").g_trib_compra_gov(GTribCompraGovData::new(
        "18.0000", "180.00", "5.0000", "50.00", "9.0000", "90.00",
    ));
    let xml = build_ibs_cbs_xml(&data);
    assert!(xml.contains("<gTribCompraGov>"));
    assert!(xml.contains("<pAliqIBSUF>18.0000</pAliqIBSUF>"));
    assert!(xml.contains("<vTribCBS>90.00</vTribCBS>"));
}

#[test]
fn cred_pres_ibs_zfm_xml() {
    let data = IbsCbsData::new("00", "12345678")
        .g_cred_pres_ibs_zfm(GCredPresIbsZfmData::new("1", "100.00").compet_apur("2025-06"));
    let xml = build_ibs_cbs_xml(&data);
    assert!(xml.contains("<gCredPresIBSZFM>"));
    assert!(xml.contains("<competApur>2025-06</competApur>"));
    assert!(xml.contains("<tpCredPresIBSZFM>1</tpCredPresIBSZFM>"));
    assert!(xml.contains("<vCredPresIBSZFM>100.00</vCredPresIBSZFM>"));
}

#[test]
fn ajuste_compet_xml() {
    let data = IbsCbsData::new("00", "12345678")
        .g_ajuste_compet(GAjusteCompetData::new("2025-06", "50.00", "30.00"));
    let xml = build_ibs_cbs_xml(&data);
    assert!(xml.contains("<gAjusteCompet>"));
    assert!(xml.contains("<competApur>2025-06</competApur>"));
    assert!(xml.contains("<vIBS>50.00</vIBS>"));
    assert!(xml.contains("<vCBS>30.00</vCBS>"));
}

// ── Coverage: GDevTribData::new, GRedData::new constructors ─────────

#[test]
fn g_dev_trib_data_new() {
    let d = GDevTribData::new("3.00");
    assert_eq!(d.v_dev_trib, "3.00");
}

#[test]
fn g_red_data_new() {
    let d = GRedData::new("20.0000", "15.0000");
    assert_eq!(d.p_red_aliq, "20.0000");
    assert_eq!(d.p_aliq_efet, "15.0000");
}

// ── Coverage: GIbsUfData builder setters g_dev_trib, g_red ──────────

#[test]
fn g_ibs_uf_with_dev_trib_and_red() {
    let uf = GIbsUfData::new("18.0000", "180.00")
        .g_dev_trib(GDevTribData::new("3.00"))
        .g_red(GRedData::new("20.0000", "15.0000"));
    assert!(uf.g_dev_trib.is_some());
    assert!(uf.g_red.is_some());
}

// ── Coverage: GIbsMunData builder setters g_dif, g_dev_trib, g_red ──

#[test]
fn g_ibs_mun_with_all_sub_groups() {
    let mun = GIbsMunData::new("5.0000", "50.00")
        .g_dif(GDifData::new("10.0000", "5.00"))
        .g_dev_trib(GDevTribData::new("2.00"))
        .g_red(GRedData::new("15.0000", "4.2500"));
    assert!(mun.g_dif.is_some());
    assert!(mun.g_dev_trib.is_some());
    assert!(mun.g_red.is_some());
}

// ── Coverage: GCbsData builder setters g_dif, g_dev_trib, g_red ─────

#[test]
fn g_cbs_with_all_sub_groups() {
    let cbs = GCbsData::new("9.0000", "90.00")
        .g_dif(GDifData::new("5.0000", "4.50"))
        .g_dev_trib(GDevTribData::new("1.50"))
        .g_red(GRedData::new("10.0000", "8.1000"));
    assert!(cbs.g_dif.is_some());
    assert!(cbs.g_dev_trib.is_some());
    assert!(cbs.g_red.is_some());
}

// ── Coverage: GMonoRetenData, GMonoRetData, GMonoDifData constructors ──

#[test]
fn g_mono_reten_data_new() {
    let d = GMonoRetenData::new("200.00", "0.2000", "40.00", "0.1000", "20.00");
    assert_eq!(d.q_bc_mono_reten, "200.00");
    assert_eq!(d.ad_rem_ibs_reten, "0.2000");
    assert_eq!(d.v_ibs_mono_reten, "40.00");
    assert_eq!(d.ad_rem_cbs_reten, "0.1000");
    assert_eq!(d.v_cbs_mono_reten, "20.00");
}

#[test]
fn g_mono_ret_data_new() {
    let d = GMonoRetData::new("150.00", "0.1500", "22.50", "0.0800", "12.00");
    assert_eq!(d.q_bc_mono_ret, "150.00");
    assert_eq!(d.ad_rem_ibs_ret, "0.1500");
    assert_eq!(d.v_ibs_mono_ret, "22.50");
    assert_eq!(d.ad_rem_cbs_ret, "0.0800");
    assert_eq!(d.v_cbs_mono_ret, "12.00");
}

#[test]
fn g_mono_dif_data_new() {
    let d = GMonoDifData::new("10.0000", "1.50", "5.0000", "0.50");
    assert_eq!(d.p_dif_ibs, "10.0000");
    assert_eq!(d.v_ibs_mono_dif, "1.50");
    assert_eq!(d.p_dif_cbs, "5.0000");
    assert_eq!(d.v_cbs_mono_dif, "0.50");
}

// ── Coverage: GIbsCbsMonoData builder setters g_mono_reten, g_mono_ret, g_mono_dif ──

#[test]
fn g_ibs_cbs_mono_data_all_setters() {
    let mono = GIbsCbsMonoData::new("15.00", "10.00")
        .g_mono_reten(GMonoRetenData::new(
            "200.00", "0.20", "40.00", "0.10", "20.00",
        ))
        .g_mono_ret(GMonoRetData::new(
            "150.00", "0.15", "22.50", "0.08", "12.00",
        ))
        .g_mono_dif(GMonoDifData::new("10.00", "1.50", "5.00", "0.50"));
    assert!(mono.g_mono_reten.is_some());
    assert!(mono.g_mono_ret.is_some());
    assert!(mono.g_mono_dif.is_some());
}

// ── Coverage: XML building for dev_trib and red sub-elements in gIBSUF/gIBSMun/gCBS ──

#[test]
fn ibs_cbs_with_dev_trib_and_red_in_all_groups() {
    let g_ibs_uf = GIbsUfData::new("18.0000", "180.00")
        .g_dev_trib(GDevTribData::new("3.00"))
        .g_red(GRedData::new("20.0000", "14.4000"));
    let g_ibs_mun = GIbsMunData::new("5.0000", "50.00")
        .g_dif(GDifData::new("10.0000", "5.00"))
        .g_dev_trib(GDevTribData::new("2.00"))
        .g_red(GRedData::new("15.0000", "4.2500"));
    let g_cbs = GCbsData::new("9.0000", "90.00")
        .g_dif(GDifData::new("5.0000", "4.50"))
        .g_dev_trib(GDevTribData::new("1.50"))
        .g_red(GRedData::new("10.0000", "8.1000"));
    let g = GIbsCbsData::new("1000.00", g_ibs_uf, g_ibs_mun, g_cbs);
    let data = IbsCbsData::new("00", "12345678").g_ibs_cbs(g);
    let xml = build_ibs_cbs_xml(&data);

    // gIBSUF: dev_trib and red
    assert!(xml.contains("<gIBSUF>"));
    assert!(xml.contains("<gDevTrib><vDevTrib>3.00</vDevTrib></gDevTrib>"));
    assert!(
        xml.contains("<gRed><pRedAliq>20.0000</pRedAliq><pAliqEfet>14.4000</pAliqEfet></gRed>")
    );

    // gIBSMun: dif, dev_trib and red
    assert!(xml.contains("<gIBSMun>"));
    assert!(
        xml.contains("<gDif><pDif>10.0000</pDif><vDif>5.00</vDif></gDif>"),
        "gIBSMun should contain gDif"
    );
    assert!(xml.contains("<gDevTrib><vDevTrib>2.00</vDevTrib></gDevTrib>"));
    assert!(xml.contains("<gRed><pRedAliq>15.0000</pRedAliq><pAliqEfet>4.2500</pAliqEfet></gRed>"));

    // gCBS: dif, dev_trib and red
    assert!(xml.contains("<gCBS>"));
    assert!(xml.contains("<gDif><pDif>5.0000</pDif><vDif>4.50</vDif></gDif>"));
    assert!(xml.contains("<gDevTrib><vDevTrib>1.50</vDevTrib></gDevTrib>"));
    assert!(xml.contains("<gRed><pRedAliq>10.0000</pRedAliq><pAliqEfet>8.1000</pAliqEfet></gRed>"));
}

// ── Coverage: Monofasico XML with reten, ret and dif sub-groups ─────

#[test]
fn ibs_cbs_mono_with_reten_ret_dif_xml() {
    let mono = GIbsCbsMonoData::new("15.00", "10.00")
        .g_mono_padrao(GMonoPadraoData::new(
            "100.0000", "0.1500", "0.1000", "15.00", "10.00",
        ))
        .g_mono_reten(GMonoRetenData::new(
            "200.00", "0.2000", "40.00", "0.1000", "20.00",
        ))
        .g_mono_ret(GMonoRetData::new(
            "150.00", "0.1500", "22.50", "0.0800", "12.00",
        ))
        .g_mono_dif(GMonoDifData::new("10.0000", "1.50", "5.0000", "0.50"));
    let data = IbsCbsData::new("02", "87654321").g_ibs_cbs_mono(mono);
    let xml = build_ibs_cbs_xml(&data);

    // gMonoReten
    assert!(xml.contains("<gMonoReten>"));
    assert!(xml.contains("<qBCMonoReten>200.00</qBCMonoReten>"));
    assert!(xml.contains("<adRemIBSReten>0.2000</adRemIBSReten>"));
    assert!(xml.contains("<vIBSMonoReten>40.00</vIBSMonoReten>"));
    assert!(xml.contains("<adRemCBSReten>0.1000</adRemCBSReten>"));
    assert!(xml.contains("<vCBSMonoReten>20.00</vCBSMonoReten>"));

    // gMonoRet
    assert!(xml.contains("<gMonoRet>"));
    assert!(xml.contains("<qBCMonoRet>150.00</qBCMonoRet>"));
    assert!(xml.contains("<adRemIBSRet>0.1500</adRemIBSRet>"));
    assert!(xml.contains("<vIBSMonoRet>22.50</vIBSMonoRet>"));
    assert!(xml.contains("<adRemCBSRet>0.0800</adRemCBSRet>"));
    assert!(xml.contains("<vCBSMonoRet>12.00</vCBSMonoRet>"));

    // gMonoDif
    assert!(xml.contains("<gMonoDif>"));
    assert!(xml.contains("<pDifIBS>10.0000</pDifIBS>"));
    assert!(xml.contains("<vIBSMonoDif>1.50</vIBSMonoDif>"));
    assert!(xml.contains("<pDifCBS>5.0000</pDifCBS>"));
    assert!(xml.contains("<vCBSMonoDif>0.50</vCBSMonoDif>"));
}

// ── Coverage: IBSCBSTot with gMono block ────────────────────────────

#[test]
fn ibs_cbs_tot_with_g_mono() {
    let mut data = IbsCbsTotData::new("2000.00");
    data.g_mono_v_ibs_mono = Some("15.00".into());
    data.g_mono_v_cbs_mono = Some("10.00".into());
    data.g_mono_v_ibs_mono_reten = Some("40.00".into());
    data.g_mono_v_cbs_mono_reten = Some("20.00".into());
    data.g_mono_v_ibs_mono_ret = Some("22.50".into());
    data.g_mono_v_cbs_mono_ret = Some("12.00".into());
    let xml = build_ibs_cbs_tot_xml(&data);

    assert!(xml.contains("<gMono>"));
    assert!(xml.contains("<vIBSMono>15.00</vIBSMono>"));
    assert!(xml.contains("<vCBSMono>10.00</vCBSMono>"));
    assert!(xml.contains("<vIBSMonoReten>40.00</vIBSMonoReten>"));
    assert!(xml.contains("<vCBSMonoReten>20.00</vCBSMonoReten>"));
    assert!(xml.contains("<vIBSMonoRet>22.50</vIBSMonoRet>"));
    assert!(xml.contains("<vCBSMonoRet>12.00</vCBSMonoRet>"));
    assert!(xml.contains("</gMono>"));
}

// ── Coverage: IBSCBSTot with gMono using defaults ───────────────────

#[test]
fn ibs_cbs_tot_g_mono_defaults() {
    let mut data = IbsCbsTotData::new("500.00");
    // Only set v_ibs_mono to trigger gMono; other fields default to "0.00"
    data.g_mono_v_ibs_mono = Some("5.00".into());
    let xml = build_ibs_cbs_tot_xml(&data);

    assert!(xml.contains("<gMono>"));
    assert!(xml.contains("<vIBSMono>5.00</vIBSMono>"));
    assert!(xml.contains("<vCBSMono>0.00</vCBSMono>"));
    assert!(xml.contains("<vIBSMonoReten>0.00</vIBSMonoReten>"));
    assert!(xml.contains("<vCBSMonoReten>0.00</vCBSMonoReten>"));
    assert!(xml.contains("<vIBSMonoRet>0.00</vIBSMonoRet>"));
    assert!(xml.contains("<vCBSMonoRet>0.00</vCBSMonoRet>"));
}

// ── Coverage: IBSCBSTot with gEstornoCred block ─────────────────────

#[test]
fn ibs_cbs_tot_with_g_estorno_cred() {
    let mut data = IbsCbsTotData::new("1000.00");
    data.g_estorno_cred_v_ibs_est_cred = Some("10.00".into());
    data.g_estorno_cred_v_cbs_est_cred = Some("5.00".into());
    let xml = build_ibs_cbs_tot_xml(&data);

    assert!(xml.contains("<gEstornoCred>"));
    assert!(xml.contains("<vIBSEstCred>10.00</vIBSEstCred>"));
    assert!(xml.contains("<vCBSEstCred>5.00</vCBSEstCred>"));
    assert!(xml.contains("</gEstornoCred>"));
}

// ── Coverage: gEstornoCred with only one side set ───────────────────

#[test]
fn ibs_cbs_tot_g_estorno_cred_only_ibs() {
    let mut data = IbsCbsTotData::new("1000.00");
    data.g_estorno_cred_v_ibs_est_cred = Some("10.00".into());
    // CBS side not set -- should still emit block with default "0.00" for CBS
    let xml = build_ibs_cbs_tot_xml(&data);

    assert!(xml.contains("<gEstornoCred>"));
    assert!(xml.contains("<vIBSEstCred>10.00</vIBSEstCred>"));
    assert!(xml.contains("<vCBSEstCred>0.00</vCBSEstCred>"));
}

#[test]
fn ibs_cbs_tot_g_estorno_cred_only_cbs() {
    let mut data = IbsCbsTotData::new("1000.00");
    data.g_estorno_cred_v_cbs_est_cred = Some("5.00".into());
    let xml = build_ibs_cbs_tot_xml(&data);

    assert!(xml.contains("<gEstornoCred>"));
    assert!(xml.contains("<vIBSEstCred>0.00</vIBSEstCred>"));
    assert!(xml.contains("<vCBSEstCred>5.00</vCBSEstCred>"));
}

// ── Coverage: gEstornoCred NOT emitted when values are empty strings ─

#[test]
fn ibs_cbs_tot_g_estorno_cred_empty_strings_no_emit() {
    let mut data = IbsCbsTotData::new("1000.00");
    data.g_estorno_cred_v_ibs_est_cred = Some("".into());
    data.g_estorno_cred_v_cbs_est_cred = Some("".into());
    let xml = build_ibs_cbs_tot_xml(&data);
    assert!(!xml.contains("<gEstornoCred>"));
}
