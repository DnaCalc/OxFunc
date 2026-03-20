use crate::function::{
    ArgPreparationProfile, CoercionLiftProfile, DeterminismClass, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, VolatilityClass,
};
use crate::functions::{
    abs::ABS_META,
    acos::ACOS_META,
    acosh::ACOSH_META,
    acot::ACOT_META,
    acoth::ACOTH_META,
    amor_depreciation_family::{AMORDEGRC_META, AMORLINC_META},
    and_fn::AND_META,
    arabic_fn::ARABIC_META,
    array_text_split_family::{ARRAYTOTEXT_META, TEXTSPLIT_META},
    asin::ASIN_META,
    asinh::ASINH_META,
    atan::ATAN_META,
    atan2::ATAN2_META,
    atanh::ATANH_META,
    avedev_fn::AVEDEV_META,
    average::AVERAGE_META,
    averagea_fn::AVERAGEA_META,
    base_fn::BASE_META,
    bessel_convert_family::{BESSELI_META, BESSELJ_META, BESSELK_META, BESSELY_META},
    beta_gamma_stats_family::{
        BETA_DIST_META, BETA_INV_META, BETADIST_META, BETAINV_META, GAMMA_DIST_META,
        GAMMA_INV_META, GAMMADIST_META, GAMMAINV_META,
    },
    bitand_fn::BITAND_META,
    bitlshift_fn::BITLSHIFT_META,
    bitor_fn::BITOR_META,
    bitrshift_fn::BITRSHIFT_META,
    bitxor_fn::BITXOR_META,
    bond_core_family::{
        ACCRINT_META, ACCRINTM_META, DURATION_META, MDURATION_META, PRICE_META, PRICEMAT_META,
        YIELD_META, YIELDDISC_META, YIELDMAT_META,
    },
    callable_helpers::{
        BYCOL_META, BYROW_META, ISOMITTED_META, MAKEARRAY_META, MAP_META, REDUCE_META, SCAN_META,
    },
    cashflow_rate_family::{IRR_META, XIRR_META, XNPV_META},
    ceiling_floor_family::{
        CEILING_MATH_META, CEILING_META, CEILING_PRECISE_META, FLOOR_MATH_META, FLOOR_META,
        FLOOR_PRECISE_META, ISO_CEILING_META,
    },
    cell::CELL_META,
    chi_f_t_family::{
        CHIDIST_META, CHIINV_META, CHISQ_DIST_META, CHISQ_DIST_RT_META, CHISQ_INV_META,
        CHISQ_INV_RT_META, F_DIST_META, F_DIST_RT_META, F_INV_META, F_INV_RT_META, FDIST_META,
        FINV_META, T_DIST_2T_META, T_DIST_META, T_DIST_RT_META, T_INV_2T_META, T_INV_META,
        TDIST_META, TINV_META,
    },
    choose_ifs_family::{CHOOSE_META, IFS_META},
    clean_fn::CLEAN_META,
    column_fn::COLUMN_META,
    combin::COMBIN_META,
    combina::COMBINA_META,
    complex_family::{
        COMPLEX_META, IMABS_META, IMAGINARY_META, IMARGUMENT_META, IMCONJUGATE_META, IMCOS_META,
        IMCOSH_META, IMCOT_META, IMCSC_META, IMCSCH_META, IMDIV_META, IMEXP_META, IMLN_META,
        IMLOG2_META, IMLOG10_META, IMPOWER_META, IMPRODUCT_META, IMREAL_META, IMSEC_META,
        IMSECH_META, IMSIN_META, IMSINH_META, IMSQRT_META, IMSUB_META, IMSUM_META, IMTAN_META,
    },
    concat_family::{CONCAT_META, CONCATENATE_META},
    confidence_test_family::{CONFIDENCE_T_META, Z_TEST_META},
    correl_fn::CORREL_META,
    cos::COS_META,
    cosh::COSH_META,
    cot::COT_META,
    coth::COTH_META,
    count::COUNT_META,
    counta::COUNTA_META,
    countblank_fn::COUNTBLANK_META,
    coupon_family::{
        COUPDAYBS_META, COUPDAYS_META, COUPDAYSNC_META, COUPNCD_META, COUPNUM_META, COUPPCD_META,
    },
    covariance_p_fn::COVARIANCE_P_META,
    covariance_s_fn::COVARIANCE_S_META,
    criteria_family::{
        AVERAGEIF_META, AVERAGEIFS_META, COUNTIF_META, COUNTIFS_META, MAXIFS_META, MINIFS_META,
        SUMIFS_META,
    },
    csc::CSC_META,
    csch::CSCH_META,
    cumulative_finance_family::{CUMIPMT_META, CUMPRINC_META},
    database_family::{
        DAVERAGE_META, DCOUNT_META, DCOUNTA_META, DGET_META, DMAX_META, DMIN_META, DPRODUCT_META,
        DSTDEV_META, DSTDEVP_META, DSUM_META, DVAR_META, DVARP_META,
    },
    date_fn::DATE_META,
    date_parts_family::{
        DAY_META, DAYS_META, HOUR_META, MINUTE_META, MONTH_META, SECOND_META, TIME_META, YEAR_META,
    },
    date_value_family::{DATEDIF_META, DATEVALUE_META, DAYS360_META, TIMEVALUE_META},
    date_week_family::{EDATE_META, EOMONTH_META, ISOWEEKNUM_META, WEEKDAY_META, WEEKNUM_META},
    decimal_fn::DECIMAL_META,
    degrees::DEGREES_META,
    delta_fn::DELTA_META,
    depreciation_family::{DB_META, DDB_META, SLN_META, SYD_META, VDB_META},
    devsq_fn::DEVSQ_META,
    discount_bill_yearfrac_family::{
        DISC_META, INTRATE_META, PRICEDISC_META, RECEIVED_META, TBILLEQ_META, TBILLPRICE_META,
        TBILLYIELD_META, YEARFRAC_META,
    },
    discrete_dist_family::{
        BINOM_DIST_META, BINOM_DIST_RANGE_META, BINOM_INV_META, BINOMDIST_META, CRITBINOM_META,
        EXPON_DIST_META, EXPONDIST_META, HYPGEOM_DIST_META, HYPGEOMDIST_META, NEGBINOM_DIST_META,
        NEGBINOMDIST_META, POISSON_DIST_META, POISSON_META,
    },
    dollar_fn::DOLLAR_META,
    dollar_fraction_family::{DOLLARDE_META, DOLLARFR_META},
    engineering_radix_family::{
        BIN2DEC_META, BIN2HEX_META, BIN2OCT_META, DEC2BIN_META, DEC2HEX_META, DEC2OCT_META,
        HEX2BIN_META, HEX2DEC_META, HEX2OCT_META, OCT2BIN_META, OCT2DEC_META, OCT2HEX_META,
    },
    error_type_fn::ERROR_TYPE_META,
    even_fn::EVEN_META,
    exact_fn::EXACT_META,
    exp_fn::EXP_META,
    fact::FACT_META,
    factdouble::FACTDOUBLE_META,
    false_fn::FALSE_META,
    financial_time_value_family::{
        EFFECT_META, FV_META, FVSCHEDULE_META, IPMT_META, ISPMT_META, MIRR_META, NOMINAL_META,
        NPER_META, NPV_META, PDURATION_META, PMT_META, PPMT_META, PV_META, RATE_META, RRI_META,
    },
    fisher_fn::FISHER_META,
    fisherinv_fn::FISHERINV_META,
    fixed_fn::FIXED_META,
    gauss_fn::GAUSS_META,
    gcd_fn::GCD_META,
    geomean_fn::GEOMEAN_META,
    gestep_fn::GESTEP_META,
    harmean_fn::HARMEAN_META,
    hstack::HSTACK_META,
    if_fn::IF_META,
    iferror::IFERROR_META,
    ifna_fn::IFNA_META,
    index::INDEX_META,
    indirect::INDIRECT_META,
    info_fn::INFO_META,
    int_fn::INT_META,
    intercept_fn::INTERCEPT_META,
    is_predicates_family::{
        ISBLANK_META, ISERR_META, ISERROR_META, ISLOGICAL_META, ISNA_META, ISNONTEXT_META,
        ISODD_META, ISREF_META, ISTEXT_META,
    },
    iseven_fn::ISEVEN_META,
    isnumber::ISNUMBER_META,
    large_fn::LARGE_META,
    lcm_fn::LCM_META,
    legacy_stats_alias_family::{
        COVAR_META, LOGINV_META, MODE_META, PERCENTILE_META, PERCENTRANK_META, QUARTILE_META,
    },
    ln_fn::LN_META,
    log_fn::LOG_META,
    log10_fn::LOG10_META,
    lookup_prob_frequency_family::{FREQUENCY_META, LOOKUP_META, MODE_MULT_META, PROB_META},
    match_fn::MATCH_META,
    matrix_family::{MDETERM_META, MINVERSE_META, MMULT_META, MUNIT_META},
    max_fn::MAX_META,
    maxa_fn::MAXA_META,
    median_fn::MEDIAN_META,
    min_fn::MIN_META,
    mina_fn::MINA_META,
    misc_conversion_family::{
        BAHTTEXT_META, CONVERT_META, EUROCONVERT_META, PERCENTOF_META, RANDARRAY_META,
    },
    misc_switch_info_family::{ISFORMULA_META, SWITCH_META},
    mod_fn::MOD_META,
    mode_sngl_fn::MODE_SNGL_META,
    moment_stats_family::{KURT_META, SKEW_META, SKEW_P_META, STEYX_META, TRIMMEAN_META},
    mround::MROUND_META,
    multinomial::MULTINOMIAL_META,
    n_fn::N_META,
    na_fn::NA_META,
    normal_log_family::{
        CONFIDENCE_META, CONFIDENCE_NORM_META, LOGNORM_DIST_META, LOGNORM_INV_META,
        LOGNORMDIST_META, NORM_DIST_META, NORM_INV_META, NORM_S_DIST_META, NORM_S_INV_META,
        NORMDIST_META, NORMINV_META, NORMSDIST_META, NORMSINV_META,
    },
    not_fn::NOT_META,
    now_fn::NOW_META,
    number_regex_translate_family::{
        NUMBERVALUE_META, REGEXEXTRACT_META, REGEXREPLACE_META, REGEXTEST_META, TRANSLATE_META,
    },
    odd_bond_family::{ODDFPRICE_META, ODDFYIELD_META, ODDLPRICE_META, ODDLYIELD_META},
    odd_fn::ODD_META,
    offset::OFFSET_META,
    op_add::OP_ADD_META,
    or_fn::OR_META,
    pearson_fn::PEARSON_META,
    percentile_exc_fn::PERCENTILE_EXC_META,
    percentile_inc_fn::PERCENTILE_INC_META,
    percentrank_exc_fn::PERCENTRANK_EXC_META,
    percentrank_inc_fn::PERCENTRANK_INC_META,
    permut_fn::PERMUT_META,
    permutationa_fn::PERMUTATIONA_META,
    phi_fn::PHI_META,
    pi::PI_META,
    power_fn::POWER_META,
    product::PRODUCT_META,
    quartile_exc_fn::QUARTILE_EXC_META,
    quartile_inc_fn::QUARTILE_INC_META,
    quotient_fn::QUOTIENT_META,
    radians::RADIANS_META,
    rand_fn::RAND_META,
    rank_avg_fn::RANK_AVG_META,
    rank_eq_fn::RANK_EQ_META,
    rank_fn::RANK_META,
    regression_forecast_family::{
        FORECAST_LINEAR_META, FORECAST_META, GROWTH_META, LINEST_META, LOGEST_META, TREND_META,
    },
    roman_fn::ROMAN_META,
    round_fn::ROUND_META,
    rounddown_fn::ROUNDDOWN_META,
    roundup_fn::ROUNDUP_META,
    row_fn::ROW_META,
    rsq_fn::RSQ_META,
    rtd_fn::RTD_META,
    sec::SEC_META,
    sech::SECH_META,
    sequence::SEQUENCE_META,
    sign_fn::SIGN_META,
    sin::SIN_META,
    sinh::SINH_META,
    slope_fn::SLOPE_META,
    small_fn::SMALL_META,
    special_dist_family::{
        ERF_META, ERF_PRECISE_META, ERFC_META, ERFC_PRECISE_META, GAMMA_META, GAMMALN_META,
        GAMMALN_PRECISE_META, WEIBULL_DIST_META, WEIBULL_META,
    },
    sqrt_fn::SQRT_META,
    sqrtpi::SQRTPI_META,
    standardize_fn::STANDARDIZE_META,
    statistical_tests_family::{
        CHISQ_TEST_META, CHITEST_META, F_TEST_META, FTEST_META, T_TEST_META, TTEST_META,
    },
    stdev_fn::STDEV_META,
    stdev_p_fn::STDEV_P_META,
    stdev_s_fn::STDEV_S_META,
    stdeva_fn::STDEVA_META,
    stdevp_fn::STDEVP_META,
    stdevpa_fn::STDEVPA_META,
    sum::SUM_META,
    sumproduct_family::{
        SERIESSUM_META, SUMPRODUCT_META, SUMX2MY2_META, SUMX2PY2_META, SUMXMY2_META,
    },
    sumsq::SUMSQ_META,
    t_fn::T_META,
    tan::TAN_META,
    tanh::TANH_META,
    test_alias_family::ZTEST_META,
    text_b_compat_family::{
        FINDB_META, LEFTB_META, LENB_META, MIDB_META, REPLACEB_META, RIGHTB_META, SEARCHB_META,
    },
    text_compat_locale_family::{ASC_META, DBCS_META, JIS_META},
    text_delim_family::{TEXTAFTER_META, TEXTBEFORE_META},
    text_fn::TEXT_META,
    text_scalar_misc::{CHAR_META, CODE_META, LOWER_META, REPT_META, TRIM_META, UPPER_META},
    text_search_replace_family::{
        FIND_META, PROPER_META, REPLACE_META, SEARCH_META, SUBSTITUTE_META,
    },
    text_slice_family::{LEFT_META, LEN_META, MID_META, RIGHT_META},
    text_unicode_fn::{UNICHAR_META, UNICODE_META},
    textjoin::TEXTJOIN_META,
    today_fn::TODAY_META,
    true_fn::TRUE_META,
    trunc_fn::TRUNC_META,
    type_fn::TYPE_META,
    value_fn::VALUE_META,
    var_fn::VAR_META,
    var_p_fn::VAR_P_META,
    var_s_fn::VAR_S_META,
    vara_fn::VARA_META,
    varp_fn::VARP_META,
    varpa_fn::VARPA_META,
    vhlookup_family::{HLOOKUP_META, VLOOKUP_META},
    workday_networkdays_family::{
        NETWORKDAYS_INTL_META, NETWORKDAYS_META, WORKDAY_INTL_META, WORKDAY_META,
    },
    xlookup::XLOOKUP_META,
    xmatch::XMATCH_META,
    xor_fn::XOR_META,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XllEntryKind {
    UArity(usize),
    QUnaryNumber,
    QBinaryNumber,
    QNullaryNumber,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XllULiftPolicy {
    ScalarOnly,
    UnaryScalarOrArrayElementwise,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XllExportSpec {
    pub export_name: String,
    pub worksheet_name: String,
    pub type_text: String,
    pub arg_names: String,
    pub function_id: &'static str,
    pub min_arity: usize,
    pub entry_kind: XllEntryKind,
    pub u_lift_policy: Option<XllULiftPolicy>,
    pub preserve_refs: bool,
}

const FUNCTION_CATALOG: &[FunctionMeta] = &[
    ACOS_META,
    ACOT_META,
    ACOSH_META,
    ACOTH_META,
    ABS_META,
    ACCRINT_META,
    ACCRINTM_META,
    ASIN_META,
    ASINH_META,
    ATAN_META,
    ATAN2_META,
    ATANH_META,
    AND_META,
    AMORDEGRC_META,
    AMORLINC_META,
    ARABIC_META,
    ARRAYTOTEXT_META,
    ASC_META,
    AVEDEV_META,
    AVERAGE_META,
    AVERAGEIF_META,
    AVERAGEIFS_META,
    AVERAGEA_META,
    BAHTTEXT_META,
    BASE_META,
    BETA_DIST_META,
    BETA_INV_META,
    BETADIST_META,
    BETAINV_META,
    BESSELI_META,
    BESSELJ_META,
    BESSELK_META,
    BESSELY_META,
    BINOM_DIST_META,
    BINOM_DIST_RANGE_META,
    BINOM_INV_META,
    BINOMDIST_META,
    BIN2DEC_META,
    BIN2HEX_META,
    BIN2OCT_META,
    BITAND_META,
    BITLSHIFT_META,
    BITOR_META,
    BITRSHIFT_META,
    BITXOR_META,
    BYCOL_META,
    BYROW_META,
    CELL_META,
    CEILING_META,
    CEILING_MATH_META,
    CEILING_PRECISE_META,
    CHIDIST_META,
    CHIINV_META,
    CHOOSE_META,
    CHISQ_DIST_META,
    CHISQ_DIST_RT_META,
    CHISQ_INV_META,
    CHISQ_INV_RT_META,
    CHISQ_TEST_META,
    CHITEST_META,
    CHAR_META,
    COLUMN_META,
    CODE_META,
    COMBIN_META,
    COMBINA_META,
    COMPLEX_META,
    CONCAT_META,
    CONCATENATE_META,
    CLEAN_META,
    CORREL_META,
    COS_META,
    COSH_META,
    COT_META,
    COTH_META,
    COUNT_META,
    COUNTBLANK_META,
    COUNTIF_META,
    COUNTIFS_META,
    COUNTA_META,
    COUPDAYBS_META,
    COUPDAYS_META,
    COUPDAYSNC_META,
    COUPNCD_META,
    COUPNUM_META,
    COUPPCD_META,
    CRITBINOM_META,
    COVARIANCE_P_META,
    COVARIANCE_S_META,
    CSC_META,
    CSCH_META,
    CUMIPMT_META,
    CUMPRINC_META,
    CONVERT_META,
    DAVERAGE_META,
    DCOUNT_META,
    DCOUNTA_META,
    DISC_META,
    DGET_META,
    DMAX_META,
    DMIN_META,
    DPRODUCT_META,
    DSTDEV_META,
    DSTDEVP_META,
    DSUM_META,
    DVAR_META,
    DVARP_META,
    DATE_META,
    DATEDIF_META,
    DAY_META,
    DAYS_META,
    DAYS360_META,
    DBCS_META,
    DATEVALUE_META,
    DB_META,
    DEC2BIN_META,
    DEC2HEX_META,
    DEC2OCT_META,
    DURATION_META,
    FDIST_META,
    EDATE_META,
    EOMONTH_META,
    HOUR_META,
    ISOWEEKNUM_META,
    MINUTE_META,
    SECOND_META,
    TIME_META,
    DECIMAL_META,
    DDB_META,
    DEVSQ_META,
    DEGREES_META,
    DELTA_META,
    DOLLAR_META,
    DOLLARDE_META,
    DOLLARFR_META,
    EFFECT_META,
    EUROCONVERT_META,
    EVEN_META,
    ERROR_TYPE_META,
    ERF_META,
    ERF_PRECISE_META,
    ERFC_META,
    ERFC_PRECISE_META,
    FIND_META,
    FINDB_META,
    EXACT_META,
    EXP_META,
    EXPON_DIST_META,
    EXPONDIST_META,
    FACT_META,
    FACTDOUBLE_META,
    FORECAST_META,
    FORECAST_LINEAR_META,
    F_DIST_META,
    F_DIST_RT_META,
    F_INV_META,
    F_INV_RT_META,
    F_TEST_META,
    FINV_META,
    FALSE_META,
    FTEST_META,
    FREQUENCY_META,
    FV_META,
    FVSCHEDULE_META,
    FISHER_META,
    FISHERINV_META,
    FIXED_META,
    FLOOR_META,
    FLOOR_MATH_META,
    FLOOR_PRECISE_META,
    GAUSS_META,
    GAMMA_META,
    GAMMA_DIST_META,
    GAMMA_INV_META,
    GAMMADIST_META,
    GAMMAINV_META,
    GAMMALN_META,
    GAMMALN_PRECISE_META,
    GCD_META,
    GEOMEAN_META,
    GESTEP_META,
    GROWTH_META,
    HARMEAN_META,
    HSTACK_META,
    HYPGEOM_DIST_META,
    HYPGEOMDIST_META,
    INFO_META,
    ISOMITTED_META,
    IRR_META,
    IPMT_META,
    INTRATE_META,
    ISPMT_META,
    JIS_META,
    IF_META,
    IFERROR_META,
    IFNA_META,
    IFS_META,
    INDEX_META,
    INDIRECT_META,
    IMABS_META,
    IMAGINARY_META,
    IMARGUMENT_META,
    IMCONJUGATE_META,
    IMCOS_META,
    IMCOSH_META,
    IMCOT_META,
    IMCSC_META,
    IMCSCH_META,
    IMDIV_META,
    IMEXP_META,
    IMLN_META,
    IMLOG10_META,
    IMLOG2_META,
    IMPOWER_META,
    IMPRODUCT_META,
    IMREAL_META,
    IMSEC_META,
    IMSECH_META,
    IMSIN_META,
    IMSINH_META,
    IMSQRT_META,
    IMSUB_META,
    IMSUM_META,
    IMTAN_META,
    INTERCEPT_META,
    ISBLANK_META,
    ISERR_META,
    ISERROR_META,
    ISLOGICAL_META,
    ISNA_META,
    ISNONTEXT_META,
    ISODD_META,
    ISNUMBER_META,
    ISREF_META,
    ISTEXT_META,
    ISO_CEILING_META,
    INT_META,
    ISEVEN_META,
    KURT_META,
    HEX2BIN_META,
    HEX2DEC_META,
    HEX2OCT_META,
    LARGE_META,
    LCM_META,
    LINEST_META,
    COVAR_META,
    LOGINV_META,
    LN_META,
    LOOKUP_META,
    LOG_META,
    LOG10_META,
    LOGEST_META,
    LOWER_META,
    LEFT_META,
    LEFTB_META,
    LEN_META,
    LENB_META,
    MID_META,
    MIDB_META,
    RIGHT_META,
    RIGHTB_META,
    MAX_META,
    MAXA_META,
    MAXIFS_META,
    MEDIAN_META,
    MIRR_META,
    ISFORMULA_META,
    MATCH_META,
    MAKEARRAY_META,
    MAP_META,
    MDETERM_META,
    MDURATION_META,
    MINVERSE_META,
    MMULT_META,
    MUNIT_META,
    MIN_META,
    MINA_META,
    MINIFS_META,
    MOD_META,
    MODE_MULT_META,
    MODE_SNGL_META,
    MONTH_META,
    MROUND_META,
    MULTINOMIAL_META,
    N_META,
    NA_META,
    NEGBINOM_DIST_META,
    NEGBINOMDIST_META,
    NOMINAL_META,
    NPER_META,
    NPV_META,
    NUMBERVALUE_META,
    CONFIDENCE_META,
    CONFIDENCE_T_META,
    CONFIDENCE_NORM_META,
    LOGNORM_DIST_META,
    LOGNORM_INV_META,
    LOGNORMDIST_META,
    NORM_DIST_META,
    NORM_INV_META,
    NORM_S_DIST_META,
    NORM_S_INV_META,
    NORMDIST_META,
    NORMINV_META,
    NORMSDIST_META,
    NORMSINV_META,
    NETWORKDAYS_META,
    NETWORKDAYS_INTL_META,
    MODE_META,
    NOT_META,
    NOW_META,
    OCT2BIN_META,
    OCT2DEC_META,
    OCT2HEX_META,
    ODD_META,
    ODDFPRICE_META,
    ODDFYIELD_META,
    ODDLPRICE_META,
    ODDLYIELD_META,
    OR_META,
    PEARSON_META,
    PHI_META,
    PERCENTILE_EXC_META,
    PERCENTILE_INC_META,
    PERCENTILE_META,
    PERCENTRANK_EXC_META,
    PERCENTRANK_INC_META,
    PERCENTRANK_META,
    PDURATION_META,
    POISSON_META,
    POISSON_DIST_META,
    OFFSET_META,
    OP_ADD_META,
    PERMUT_META,
    PERMUTATIONA_META,
    PI_META,
    PMT_META,
    PPMT_META,
    PERCENTOF_META,
    PRICE_META,
    PRICEDISC_META,
    PRICEMAT_META,
    PROB_META,
    PRODUCT_META,
    POWER_META,
    PROPER_META,
    PV_META,
    QUOTIENT_META,
    QUARTILE_EXC_META,
    QUARTILE_INC_META,
    QUARTILE_META,
    RADIANS_META,
    RANK_META,
    RANK_AVG_META,
    RANK_EQ_META,
    RAND_META,
    RANDARRAY_META,
    RATE_META,
    REDUCE_META,
    ROMAN_META,
    ROW_META,
    RSQ_META,
    ROUND_META,
    ROUNDDOWN_META,
    ROUNDUP_META,
    REPLACE_META,
    REPLACEB_META,
    RECEIVED_META,
    REGEXEXTRACT_META,
    REGEXREPLACE_META,
    REGEXTEST_META,
    SCAN_META,
    SEC_META,
    SERIESSUM_META,
    SEQUENCE_META,
    SECH_META,
    SIGN_META,
    SIN_META,
    SINH_META,
    SKEW_META,
    SKEW_P_META,
    STEYX_META,
    SLOPE_META,
    SLN_META,
    SMALL_META,
    SQRT_META,
    SQRTPI_META,
    STDEV_META,
    STDEV_P_META,
    STDEV_S_META,
    STDEVP_META,
    STDEVA_META,
    STDEVPA_META,
    STANDARDIZE_META,
    SUM_META,
    SUMIFS_META,
    SUMSQ_META,
    SUMPRODUCT_META,
    SUMX2MY2_META,
    SUMX2PY2_META,
    SUMXMY2_META,
    SWITCH_META,
    T_META,
    T_DIST_META,
    T_DIST_2T_META,
    T_DIST_RT_META,
    T_INV_META,
    T_INV_2T_META,
    T_TEST_META,
    TAN_META,
    TANH_META,
    TBILLEQ_META,
    TBILLPRICE_META,
    TBILLYIELD_META,
    ZTEST_META,
    TDIST_META,
    TEXTAFTER_META,
    TEXTBEFORE_META,
    TEXTSPLIT_META,
    TEXT_META,
    SEARCH_META,
    SEARCHB_META,
    REPT_META,
    SUBSTITUTE_META,
    TEXTJOIN_META,
    TODAY_META,
    TIMEVALUE_META,
    TRANSLATE_META,
    TRIMMEAN_META,
    TRUE_META,
    TREND_META,
    TINV_META,
    TRUNC_META,
    TRIM_META,
    TTEST_META,
    TYPE_META,
    UNICHAR_META,
    UNICODE_META,
    UPPER_META,
    VALUE_META,
    VAR_META,
    VAR_P_META,
    VAR_S_META,
    VARA_META,
    VARP_META,
    VARPA_META,
    RRI_META,
    RTD_META,
    VDB_META,
    HLOOKUP_META,
    VLOOKUP_META,
    WORKDAY_META,
    WORKDAY_INTL_META,
    XLOOKUP_META,
    XIRR_META,
    XNPV_META,
    XMATCH_META,
    XOR_META,
    WEEKDAY_META,
    WEEKNUM_META,
    WEIBULL_META,
    WEIBULL_DIST_META,
    SYD_META,
    YIELD_META,
    YIELDDISC_META,
    YIELDMAT_META,
    YEAR_META,
    YEARFRAC_META,
    Z_TEST_META,
];

fn function_suffix(function_id: &str) -> String {
    function_id
        .strip_prefix("FUNC.")
        .unwrap_or(function_id)
        .replace('.', "_")
}

fn csv_escape(field: &str) -> String {
    let needs_quotes = field.contains(',') || field.contains('"') || field.contains('\n');
    if !needs_quotes {
        return field.to_string();
    }

    let escaped = field.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

fn arg_names_for_count(count: usize) -> String {
    if count == 0 {
        return String::new();
    }
    (1..=count)
        .map(|i| format!("arg{i}"))
        .collect::<Vec<_>>()
        .join(",")
}

const MAX_XLL_ARG_NAMES_LEN: usize = 255;

fn capped_arg_names_for_count(count: usize) -> String {
    let arg_names = arg_names_for_count(count);
    if arg_names.len() > MAX_XLL_ARG_NAMES_LEN {
        String::new()
    } else {
        arg_names
    }
}

fn type_text_for_u_arity(count: usize) -> String {
    let mut out = String::from("Q");
    out.push_str(&"U".repeat(count));
    out
}

const MAX_XLL_TYPE_TEXT_LEN: usize = 255;

fn registration_suffix_len(meta: &FunctionMeta) -> usize {
    let mut len = 0;
    if meta.volatility == VolatilityClass::VolatileFull {
        len += 1;
    }
    len
}

fn capped_u_arity(meta: &FunctionMeta) -> usize {
    let max_u_arity = MAX_XLL_TYPE_TEXT_LEN.saturating_sub(1 + registration_suffix_len(meta));
    meta.arity.max.min(max_u_arity)
}

fn apply_registration_suffixes(meta: &FunctionMeta, base_type_text: String) -> String {
    let mut out = base_type_text;
    if meta.volatility == VolatilityClass::VolatileFull {
        out.push('!');
    }
    out
}

fn q_entry_kind_from_profile(meta: &FunctionMeta) -> Option<XllEntryKind> {
    let exact_arity = meta.arity.min == meta.arity.max;
    if !exact_arity {
        return None;
    }

    let profile_allows_q = meta.determinism == DeterminismClass::Deterministic
        && meta.volatility == VolatilityClass::NonVolatile
        && meta.host_interaction == HostInteractionClass::None
        && meta.arg_preparation_profile == ArgPreparationProfile::ValuesOnlyPreAdapter;

    if !profile_allows_q {
        return None;
    }

    match (meta.arity.min, meta.kernel_signature_class) {
        (0, KernelSignatureClass::NullaryConst) => Some(XllEntryKind::QNullaryNumber),
        (1, KernelSignatureClass::NumToNum) => Some(XllEntryKind::QUnaryNumber),
        (2, KernelSignatureClass::NumsToNum) => Some(XllEntryKind::QBinaryNumber),
        _ => None,
    }
}

fn u_lift_policy_from_profile(meta: &FunctionMeta) -> XllULiftPolicy {
    match meta.coercion_lift_profile {
        CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise
            if meta.arity.min == 1 && meta.arity.max == 1 =>
        {
            XllULiftPolicy::UnaryScalarOrArrayElementwise
        }
        _ => XllULiftPolicy::ScalarOnly,
    }
}

pub fn xll_export_specs() -> Vec<XllExportSpec> {
    let mut specs = Vec::new();

    for meta in FUNCTION_CATALOG {
        let suffix = function_suffix(meta.function_id);
        let export_base = format!("OX_{suffix}");
        let worksheet_base = format!("ox_{suffix}");
        let q_kind = q_entry_kind_from_profile(meta);
        let emit_u = meta.arity.max > 0 || q_kind.is_none();
        let u_arity = capped_u_arity(meta);

        if emit_u {
            specs.push(XllExportSpec {
                export_name: export_base.clone(),
                worksheet_name: worksheet_base.clone(),
                type_text: apply_registration_suffixes(meta, type_text_for_u_arity(u_arity)),
                arg_names: capped_arg_names_for_count(u_arity),
                function_id: meta.function_id,
                min_arity: meta.arity.min,
                entry_kind: XllEntryKind::UArity(u_arity),
                u_lift_policy: Some(u_lift_policy_from_profile(meta)),
                preserve_refs: meta.arg_preparation_profile
                    == ArgPreparationProfile::RefsVisibleInAdapter,
            });
        }

        if let Some(kind) = q_kind {
            let (type_text, arg_names) = match kind {
                XllEntryKind::QNullaryNumber => ("B".to_string(), String::new()),
                XllEntryKind::QUnaryNumber => ("BB".to_string(), "value".to_string()),
                XllEntryKind::QBinaryNumber => ("BBB".to_string(), "lhs,rhs".to_string()),
                XllEntryKind::UArity(_) => unreachable!(),
            };

            let export_name = if emit_u {
                format!("{export_base}_Q")
            } else {
                export_base.clone()
            };
            let worksheet_name = if emit_u {
                format!("{worksheet_base}_Q")
            } else {
                worksheet_base.clone()
            };

            specs.push(XllExportSpec {
                export_name,
                worksheet_name,
                type_text,
                arg_names,
                function_id: meta.function_id,
                min_arity: meta.arity.min,
                entry_kind: kind,
                u_lift_policy: None,
                preserve_refs: false,
            });
        }
    }

    specs.sort_by(|a, b| a.export_name.cmp(&b.export_name));
    specs
}

pub fn render_export_specs_csv() -> String {
    let mut out = String::from(
        "export_name,worksheet_name,type_text,arg_names,function_id,min_arity,entry_kind,u_lift_policy,preserve_refs\n",
    );
    for spec in xll_export_specs() {
        let entry_kind = match spec.entry_kind {
            XllEntryKind::UArity(n) => format!("u_arity_{n}"),
            XllEntryKind::QUnaryNumber => "q_unary_number".to_string(),
            XllEntryKind::QBinaryNumber => "q_binary_number".to_string(),
            XllEntryKind::QNullaryNumber => "q_nullary_number".to_string(),
        };
        let u_lift = match spec.u_lift_policy {
            Some(XllULiftPolicy::ScalarOnly) => "scalar_only",
            Some(XllULiftPolicy::UnaryScalarOrArrayElementwise) => {
                "unary_scalar_or_array_elementwise"
            }
            None => "",
        };
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            csv_escape(&spec.export_name),
            csv_escape(&spec.worksheet_name),
            csv_escape(&spec.type_text),
            csv_escape(&spec.arg_names),
            csv_escape(spec.function_id),
            spec.min_arity,
            csv_escape(&entry_kind),
            csv_escape(u_lift),
            if spec.preserve_refs { "true" } else { "false" }
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_specs_have_unique_export_and_worksheet_names() {
        let mut exports = std::collections::BTreeSet::new();
        let mut worksheets = std::collections::BTreeSet::new();
        for spec in xll_export_specs() {
            assert!(exports.insert(spec.export_name));
            assert!(worksheets.insert(spec.worksheet_name));
        }
    }

    #[test]
    fn all_catalog_functions_have_at_least_one_export() {
        let specs = xll_export_specs();
        let mut ids = std::collections::BTreeSet::new();
        for spec in specs {
            ids.insert(spec.function_id);
        }
        for meta in FUNCTION_CATALOG {
            assert!(ids.contains(meta.function_id));
        }
    }

    #[test]
    fn csv_header_and_row_count_are_stable() {
        let csv = render_export_specs_csv();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(
            lines.first().copied(),
            Some(
                "export_name,worksheet_name,type_text,arg_names,function_id,min_arity,entry_kind,u_lift_policy,preserve_refs"
            )
        );
        assert_eq!(lines.len(), xll_export_specs().len() + 1);
    }

    #[test]
    fn volatile_full_u_exports_receive_bang_suffix() {
        let specs = xll_export_specs();
        for function_id in ["FUNC.NOW", "FUNC.TODAY", "FUNC.RAND"] {
            let spec = specs
                .iter()
                .find(|s| {
                    s.function_id == function_id && matches!(s.entry_kind, XllEntryKind::UArity(_))
                })
                .unwrap_or_else(|| panic!("missing U export for {function_id}"));
            assert!(
                spec.type_text.ends_with('!'),
                "expected volatile U export for {function_id} to end with !, got {}",
                spec.type_text
            );
        }
    }

    #[test]
    fn nonvolatile_u_export_does_not_receive_bang_suffix() {
        let specs = xll_export_specs();
        let spec = specs
            .iter()
            .find(|s| {
                s.function_id == "FUNC.ABS" && matches!(s.entry_kind, XllEntryKind::UArity(_))
            })
            .expect("missing U export for FUNC.ABS");
        assert!(
            !spec.type_text.ends_with('!'),
            "expected nonvolatile U export for FUNC.ABS to omit !, got {}",
            spec.type_text
        );
    }

    #[test]
    fn u_exports_stay_within_type_text_limit() {
        let specs = xll_export_specs();
        for spec in specs {
            if matches!(spec.entry_kind, XllEntryKind::UArity(_)) {
                assert!(
                    spec.type_text.len() <= MAX_XLL_TYPE_TEXT_LEN,
                    "expected {} to stay within {MAX_XLL_TYPE_TEXT_LEN}, got {}",
                    spec.export_name,
                    spec.type_text.len()
                );
            }
        }
    }

    #[test]
    fn roman_has_only_u_export_with_optional_second_argument_shape() {
        let specs = xll_export_specs();
        let roman_specs = specs
            .iter()
            .filter(|s| s.function_id == "FUNC.ROMAN")
            .collect::<Vec<_>>();
        assert_eq!(roman_specs.len(), 1);
        let spec = roman_specs[0];
        assert_eq!(spec.entry_kind, XllEntryKind::UArity(2));
        assert_eq!(spec.type_text, "QUU");
        assert_eq!(spec.arg_names, "arg1,arg2");
    }

    #[test]
    fn sum_u_export_is_capped_to_callable_arity_limit() {
        let specs = xll_export_specs();
        let spec = specs
            .iter()
            .find(|s| {
                s.function_id == "FUNC.SUM" && matches!(s.entry_kind, XllEntryKind::UArity(_))
            })
            .expect("missing U export for FUNC.SUM");
        assert_eq!(spec.entry_kind, XllEntryKind::UArity(254));
        assert_eq!(spec.type_text.len(), MAX_XLL_TYPE_TEXT_LEN);
        assert!(spec.arg_names.is_empty());
    }
}
