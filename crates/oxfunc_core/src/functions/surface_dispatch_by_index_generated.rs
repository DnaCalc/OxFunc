// Auto-generated from eval_surface_value_call_with_callable string arms.
// Do not edit by hand; regenerate from surface_dispatch.rs when the catalog arms change.
match dispatch_key.catalog_index {
    // FUNC.ACOS
    0 => eval_acos_surface(args, resolver).map_err(|e| map_acos_error_to_ws(&e)),
    // FUNC.ACOT
    1 => eval_acot_surface(args, resolver).map_err(|e| map_acot_error_to_ws(&e)),
    // FUNC.ACOSH
    2 => {
                eval_acosh_surface(args, resolver).map_err(|e| map_acosh_error_to_ws(&e))
            }
            3 => {
                eval_acoth_surface(args, resolver).map_err(|e| map_acoth_error_to_ws(&e))
            }
            4 => {
                eval_abs_scalar_value(args, resolver).map_err(|e| map_abs_error_to_ws(&e))
            }
            5 => {
                eval_accrint_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            6 => {
                eval_accrintm_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            21 => {
                crate::functions::subtotal_aggregate_family::eval_aggregate_surface(
                    args, resolver, host_info,
                )
                .map_err(|e| {
                    crate::functions::subtotal_aggregate_family::map_subtotal_aggregate_error_to_ws(
                        &e,
                    )
                })
            }
            9 => eval_atan_surface(args, resolver).map_err(|e| map_atan_error_to_ws(&e)),
    // FUNC.ASIN
    7 => eval_asin_surface(args, resolver).map_err(|e| map_asin_error_to_ws(&e)),
    // FUNC.ASINH
    8 => {
                eval_asinh_surface(args, resolver).map_err(|e| map_asinh_error_to_ws(&e))
            }
            10 => {
                eval_atan2_surface(args, resolver).map_err(|e| map_atan2_error_to_ws(&e))
            }
            11 => {
                eval_atanh_surface(args, resolver).map_err(|e| map_atanh_error_to_ws(&e))
            }
            12 => eval_and_surface(args, resolver).map_err(|e| map_and_error_to_ws(&e)),
    // FUNC.AMORDEGRC
    13 => eval_amordegrc_surface(args, resolver)
                .map_err(|e| map_amor_depreciation_error_to_ws(&e)),
    // FUNC.AMORLINC
    14 => eval_amorlinc_surface(args, resolver)
                .map_err(|e| map_amor_depreciation_error_to_ws(&e)),
    // FUNC.ARABIC
    15 => {
                eval_arabic_surface(args, resolver).map_err(|e| map_arabic_error_to_ws(&e))
            }
            50 => eval_call_surface(args, resolver, registered_external_provider)
                .map_err(|e| map_call_register_id_error_to_ws(&e)),
    // FUNC.ADDRESS
    16 => eval_address_surface(args, resolver)
                .map_err(|e| map_reference_metadata_error_to_ws(&e)),
    // FUNC.ARRAYTOTEXT
    17 => eval_arraytotext_surface(args, resolver)
                .map_err(|e| map_array_text_split_error_to_ws(&e)),
    // FUNC.ASC
    18 => eval_asc_surface(args, resolver, host_info)
                .map_err(|e| map_text_compat_locale_error_to_ws(&e)),
    // FUNC.AREAS
    19 => {
                eval_areas_surface(args).map_err(|e| map_reference_metadata_error_to_ws(&e))
            }
            20 => {
                eval_avedev_surface(args, resolver).map_err(|e| map_avedev_error_to_ws(&e))
            }
            22 => {
                eval_average_surface(args, resolver).map_err(|e| map_average_error_to_ws(&e))
            }
            23 => {
                eval_averageif_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            24 => {
                eval_averageifs_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            25 => {
                eval_averagea_surface(args, resolver).map_err(|e| map_averagea_error_to_ws(&e))
            }
            26 => eval_bahttext_surface(args, resolver)
                .map_err(|e| map_misc_conversion_error_to_ws(&e)),
    // FUNC.BASE
    27 => eval_base_surface(args, resolver).map_err(|e| map_base_error_to_ws(&e)),
    // FUNC.BETA.DIST
    28 => eval_beta_dist_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
    // FUNC.BETA.INV
    29 => eval_beta_inv_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
    // FUNC.BETADIST
    30 => eval_betadist_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
    // FUNC.BETAINV
    31 => eval_betainv_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
    // FUNC.BESSELI
    32 => {
                eval_besseli_surface(args, resolver).map_err(|e| map_bessel_convert_error_to_ws(&e))
            }
            33 => {
                eval_besselj_surface(args, resolver).map_err(|e| map_bessel_convert_error_to_ws(&e))
            }
            34 => {
                eval_besselk_surface(args, resolver).map_err(|e| map_bessel_convert_error_to_ws(&e))
            }
            35 => {
                eval_bessely_surface(args, resolver).map_err(|e| map_bessel_convert_error_to_ws(&e))
            }
            36 => eval_binom_dist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
    // FUNC.BINOM.DIST.RANGE
    37 => eval_binom_dist_range_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
    // FUNC.BINOM.INV
    38 => eval_binom_inv_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
    // FUNC.BINOMDIST
    39 => eval_binomdist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
    // FUNC.BIN2DEC
    40 => eval_bin2dec_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
    // FUNC.BIN2HEX
    41 => eval_bin2hex_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
    // FUNC.BIN2OCT
    42 => eval_bin2oct_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
    // FUNC.BITAND
    43 => {
                eval_bitand_surface(args, resolver).map_err(|e| map_bitand_error_to_ws(&e))
            }
            44 => {
                eval_bitlshift_surface(args, resolver).map_err(|e| map_bitlshift_error_to_ws(&e))
            }
            45 => {
                eval_bitor_surface(args, resolver).map_err(|e| map_bitor_error_to_ws(&e))
            }
            46 => {
                eval_bitrshift_surface(args, resolver).map_err(|e| map_bitrshift_error_to_ws(&e))
            }
            47 => {
                eval_bitxor_surface(args, resolver).map_err(|e| map_bitxor_error_to_ws(&e))
            }
            48 => eval_bycol_surface(args, resolver, callable_invoker)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
    // FUNC.BYROW
    49 => eval_byrow_surface(args, resolver, callable_invoker)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
    // FUNC.CELL
    51 => {
                eval_cell_surface(args, resolver, host_info).map_err(|e| map_cell_error_to_ws(&e))
            }
            52 => {
                eval_ceiling_surface(args, resolver).map_err(|e| map_ceiling_floor_error_to_ws(&e))
            }
            53 => eval_ceiling_math_surface(args, resolver)
                .map_err(|e| map_ceiling_floor_error_to_ws(&e)),
    // FUNC.CEILING.PRECISE
    54 => eval_ceiling_precise_surface(args, resolver)
                .map_err(|e| map_ceiling_floor_error_to_ws(&e)),
    // FUNC.CHIDIST
    55 => {
                eval_chidist_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            56 => {
                eval_chiinv_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            57 => {
                eval_choose_surface(args, resolver).map_err(|e| map_choose_ifs_error_to_ws(&e))
            }
            58 => eval_choosecols_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
    // FUNC.CHOOSEROWS
    59 => eval_chooserows_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
    // FUNC.CHISQ.DIST
    60 => {
                eval_chisq_dist_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            61 => {
                eval_chisq_dist_rt_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            62 => {
                eval_chisq_inv_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            63 => {
                eval_chisq_inv_rt_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            64 => eval_chisq_test_surface(args, resolver)
                .map_err(|e| map_statistical_tests_error_to_ws(&e)),
    // FUNC.CHITEST
    65 => eval_chitest_surface(args, resolver)
                .map_err(|e| map_statistical_tests_error_to_ws(&e)),
    // FUNC.CHAR
    66 => {
                eval_char_surface(args, resolver).map_err(|e| map_text_scalar_error_to_ws(&e))
            }
            69 => {
                eval_code_surface(args, resolver).map_err(|e| map_text_scalar_error_to_ws(&e))
            }
            70 => {
                eval_combin_surface(args, resolver).map_err(|e| map_combin_error_to_ws(&e))
            }
            71 => {
                eval_combina_surface(args, resolver).map_err(|e| map_combina_error_to_ws(&e))
            }
            72 => {
                eval_complex_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            75 => {
                eval_clean_surface(args, resolver).map_err(|e| map_clean_error_to_ws(&e))
            }
            73 => {
                eval_concat_surface(args, resolver).map_err(|e| map_concat_error_to_ws(&e))
            }
            74 => {
                eval_concatenate_surface(args, resolver).map_err(|e| map_concat_error_to_ws(&e))
            }
            67 => {
                eval_column_surface(args, resolver).map_err(|e| map_column_error_to_ws(&e))
            }
            68 => eval_columns_surface(args).map_err(|e| map_columns_error_to_ws(&e)),
    // FUNC.CORREL
    76 => {
                eval_correl_surface(args, resolver).map_err(|e| map_correl_error_to_ws(&e))
            }
            93 => eval_covariance_p_surface(args, resolver)
                .map_err(|e| map_covariance_p_error_to_ws(&e)),
    // FUNC.COS
    77 => eval_cos_surface(args, resolver).map_err(|e| map_cos_error_to_ws(&e)),
    // FUNC.COSH
    78 => eval_cosh_surface(args, resolver).map_err(|e| map_cosh_error_to_ws(&e)),
    // FUNC.COT
    79 => eval_cot_surface(args, resolver).map_err(|e| map_cot_error_to_ws(&e)),
    // FUNC.COTH
    80 => eval_coth_surface(args, resolver).map_err(|e| map_coth_error_to_ws(&e)),
    // FUNC.COUNT
    81 => {
                eval_count_surface(args, resolver).map_err(|e| map_count_error_to_ws(&e))
            }
            82 => {
                eval_countblank_surface(args, resolver).map_err(|e| map_countblank_error_to_ws(&e))
            }
            86 => {
                eval_coupdaybs_surface(args, resolver).map_err(|e| map_coupon_error_to_ws(&e))
            }
            87 => {
                eval_coupdays_surface(args, resolver).map_err(|e| map_coupon_error_to_ws(&e))
            }
            88 => {
                eval_coupdaysnc_surface(args, resolver).map_err(|e| map_coupon_error_to_ws(&e))
            }
            89 => {
                eval_coupncd_surface(args, resolver).map_err(|e| map_coupon_error_to_ws(&e))
            }
            90 => {
                eval_coupnum_surface(args, resolver).map_err(|e| map_coupon_error_to_ws(&e))
            }
            91 => {
                eval_couppcd_surface(args, resolver).map_err(|e| map_coupon_error_to_ws(&e))
            }
            83 => {
                eval_countif_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            84 => {
                eval_countifs_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            85 => {
                eval_counta_surface(args, resolver).map_err(|e| map_counta_error_to_ws(&e))
            }
            260 => eval_covar_surface(args, resolver)
                .map_err(|e| map_legacy_stats_alias_error_to_ws(&e)),
    // FUNC.CRITBINOM
    92 => eval_critbinom_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
    // FUNC.COVARIANCE.S
    94 => eval_covariance_s_surface(args, resolver)
                .map_err(|e| map_covariance_s_error_to_ws(&e)),
    // FUNC.CSC
    95 => eval_csc_surface(args, resolver).map_err(|e| map_csc_error_to_ws(&e)),
    // FUNC.CSCH
    96 => eval_csch_surface(args, resolver).map_err(|e| map_csch_error_to_ws(&e)),
    // FUNC.CUMIPMT
    97 => eval_cumipmt_surface(args, resolver)
                .map_err(|e| map_cumulative_finance_error_to_ws(&e)),
    // FUNC.CUMPRINC
    98 => eval_cumprinc_surface(args, resolver)
                .map_err(|e| map_cumulative_finance_error_to_ws(&e)),
    // FUNC.CONVERT
    99 => eval_convert_surface(args, resolver)
                .map_err(|e| map_misc_conversion_error_to_ws(&e)),
    // FUNC.DAVERAGE
    100 => {
                eval_daverage_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            114 => eval_date_surface(args, resolver).map_err(|e| map_date_error_to_ws(&e)),
    // FUNC.DGET
    104 => {
                eval_dget_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            105 => {
                eval_dmax_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            106 => {
                eval_dmin_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            107 => {
                eval_dproduct_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            108 => {
                eval_dstdev_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            109 => {
                eval_dstdevp_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            110 => {
                eval_dsum_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            111 => {
                eval_dvar_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            112 => {
                eval_dvarp_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            113 => eval_drop_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
    // FUNC.DATEDIF
    115 => eval_datedif_surface(args, resolver)
                .map_err(|e| map_date_value_family_error_to_ws(&e)),
    // FUNC.DAY
    116 => {
                eval_day_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            117 => {
                eval_days_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            118 => eval_days360_surface(args, resolver)
                .map_err(|e| map_date_value_family_error_to_ws(&e)),
    // FUNC.DBCS
    119 => eval_dbcs_surface(args, resolver, host_info)
                .map_err(|e| map_text_compat_locale_error_to_ws(&e)),
    // FUNC.DATEVALUE
    120 => eval_datevalue_surface(args, resolver)
                .map_err(|e| map_date_value_family_error_to_ws(&e)),
    // FUNC.DB
    121 => {
                eval_db_surface(args, resolver).map_err(|e| map_depreciation_error_to_ws(&e))
            }
            122 => eval_dec2bin_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
    // FUNC.DEC2HEX
    123 => eval_dec2hex_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
    // FUNC.DEC2OCT
    124 => eval_dec2oct_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
    // FUNC.FDIST
    126 => {
                eval_fdist_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            169 => {
                eval_finv_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            170 => eval_false_surface(args),
    // FUNC.EDATE
    127 => {
                eval_edate_surface(args, resolver).map_err(|e| map_date_week_error_to_ws(&e))
            }
            128 => {
                eval_eomonth_surface(args, resolver).map_err(|e| map_date_week_error_to_ws(&e))
            }
            142 => eval_effect_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
    // FUNC.HOUR
    129 => {
                eval_hour_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            198 => {
                eval_hstack_surface(args, resolver).map_err(|e| map_hstack_error_to_ws(&e))
            }
            417 => eval_sort_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
    // FUNC.ISOWEEKNUM
    130 => {
                eval_isoweeknum_surface(args, resolver).map_err(|e| map_date_week_error_to_ws(&e))
            }
            299 => eval_n_surface(args, resolver).map_err(|e| map_n_error_to_ws(&e)),
    // FUNC.MINUTE
    131 => {
                eval_minute_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            323 => eval_mode_surface(args, resolver)
                .map_err(|e| map_legacy_stats_alias_error_to_ws(&e)),
    // FUNC.SECOND
    132 => {
                eval_second_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            412 => eval_sec_surface(args, resolver).map_err(|e| map_sec_error_to_ws(&e)),
    // FUNC.DECIMAL
    134 => {
                eval_decimal_surface(args, resolver).map_err(|e| map_decimal_error_to_ws(&e))
            }
            144 => {
                eval_encodeurl_surface(args, resolver).map_err(|e| map_web_text_xml_error_to_ws(&e))
            }
            135 => {
                eval_ddb_surface(args, resolver).map_err(|e| map_depreciation_error_to_ws(&e))
            }
            101 => {
                eval_dcount_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            102 => {
                eval_dcounta_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            103 => eval_disc_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
    // FUNC.DEVSQ
    136 => {
                eval_devsq_surface(args, resolver).map_err(|e| map_devsq_error_to_ws(&e))
            }
            137 => {
                eval_degrees_surface(args, resolver).map_err(|e| map_degrees_error_to_ws(&e))
            }
            138 => {
                eval_delta_surface(args, resolver).map_err(|e| map_delta_error_to_ws(&e))
            }
            125 => {
                eval_duration_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            139 => {
                let ctx = locale_ctx.ok_or(WorksheetErrorCode::Value)?;
                eval_dollar_surface(args, resolver, ctx).map_err(|e| map_dollar_error_to_ws(&e))
            }
            140 => eval_dollarde_surface(args, resolver)
                .map_err(|e| map_dollar_fraction_error_to_ws(&e)),
    // FUNC.DOLLARFR
    141 => eval_dollarfr_surface(args, resolver)
                .map_err(|e| map_dollar_fraction_error_to_ws(&e)),
    // FUNC.EUROCONVERT
    143 => eval_euroconvert_surface(args, resolver)
                .map_err(|e| map_misc_conversion_error_to_ws(&e)),
    // FUNC.EXPAND
    145 => eval_expand_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
    // FUNC.EVEN
    146 => eval_even_surface(args, resolver).map_err(|e| map_even_error_to_ws(&e)),
    // FUNC.ERROR.TYPE
    147 => {
                eval_error_type_surface(args, resolver).map_err(|e| map_error_type_error_to_ws(&e))
            }
            148 => {
                eval_erf_surface(args, resolver).map_err(|e| map_special_dist_error_to_ws(&e))
            }
            149 => eval_erf_precise_surface(args, resolver)
                .map_err(|e| map_special_dist_error_to_ws(&e)),
    // FUNC.ERFC
    150 => {
                eval_erfc_surface(args, resolver).map_err(|e| map_special_dist_error_to_ws(&e))
            }
            151 => eval_erfc_precise_surface(args, resolver)
                .map_err(|e| map_special_dist_error_to_ws(&e)),
    // FUNC.FINDB
    153 => {
                eval_findb_surface(args, resolver).map_err(|e| map_text_b_compat_error_to_ws(&e))
            }
            154 => eval_filter_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
    // FUNC.FILTERXML
    155 => {
                eval_filterxml_surface(args, resolver).map_err(|e| map_web_text_xml_error_to_ws(&e))
            }
            177 => {
                let ctx = locale_ctx.ok_or(WorksheetErrorCode::Value)?;
                eval_fixed_surface(args, resolver, ctx).map_err(|e| map_fixed_error_to_ws(&e))
            }
            178 => {
                eval_floor_surface(args, resolver).map_err(|e| map_ceiling_floor_error_to_ws(&e))
            }
            179 => eval_floor_math_surface(args, resolver)
                .map_err(|e| map_ceiling_floor_error_to_ws(&e)),
    // FUNC.EXACT
    156 => {
                eval_exact_surface(args, resolver).map_err(|e| map_exact_error_to_ws(&e))
            }
            158 => eval_expon_dist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
    // FUNC.EXP
    157 => eval_exp_surface(args, resolver).map_err(|e| map_exp_error_to_ws(&e)),
    // FUNC.EXPONDIST
    159 => eval_expondist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
    // FUNC.FACT
    160 => eval_fact_surface(args, resolver).map_err(|e| map_fact_error_to_ws(&e)),
    // FUNC.FACTDOUBLE
    161 => {
                eval_factdouble_surface(args, resolver).map_err(|e| map_factdouble_error_to_ws(&e))
            }
            164 => {
                eval_f_dist_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            165 => {
                eval_f_dist_rt_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            166 => {
                eval_f_inv_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            167 => {
                eval_f_inv_rt_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            168 => eval_f_test_surface(args, resolver)
                .map_err(|e| map_statistical_tests_error_to_ws(&e)),
    // FUNC.FORECAST
    162 => eval_forecast_surface(args, resolver)
                .map_err(|e| map_regression_forecast_error_to_ws(&e)),
    // FUNC.FORECAST.LINEAR
    163 => eval_forecast_linear_surface(args, resolver)
                .map_err(|e| map_regression_forecast_error_to_ws(&e)),
    // FUNC.FTEST
    171 => eval_ftest_surface(args, resolver)
                .map_err(|e| map_statistical_tests_error_to_ws(&e)),
    // FUNC.FREQUENCY
    172 => eval_frequency_surface(args, resolver)
                .map_err(|e| map_lookup_prob_frequency_error_to_ws(&e)),
    // FUNC.FV
    173 => eval_fv_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
    // FUNC.FVSCHEDULE
    174 => eval_fvschedule_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
    // FUNC.FISHER
    175 => {
                eval_fisher_surface(args, resolver).map_err(|e| map_fisher_error_to_ws(&e))
            }
            176 => {
                eval_fisherinv_surface(args, resolver).map_err(|e| map_fisherinv_error_to_ws(&e))
            }
            152 => eval_find_surface(args, resolver)
                .map_err(|e| map_text_search_replace_error_to_ws(&e)),
    // FUNC.FLOOR.PRECISE
    180 => eval_floor_precise_surface(args, resolver)
                .map_err(|e| map_ceiling_floor_error_to_ws(&e)),
    // FUNC.FORMULATEXT
    181 => eval_formulatext_surface(args, host_info)
                .map_err(|e| map_reference_metadata_error_to_ws(&e)),
    // FUNC.GAUSS
    182 => {
                eval_gauss_surface(args, resolver).map_err(|e| map_gauss_error_to_ws(&e))
            }
            183 => {
                eval_gamma_surface(args, resolver).map_err(|e| map_special_dist_error_to_ws(&e))
            }
            184 => eval_gamma_dist_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
    // FUNC.GAMMA.INV
    185 => eval_gamma_inv_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
    // FUNC.GAMMADIST
    186 => eval_gammadist_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
    // FUNC.GAMMAINV
    187 => eval_gammainv_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
    // FUNC.GAMMALN
    188 => {
                eval_gammaln_surface(args, resolver).map_err(|e| map_special_dist_error_to_ws(&e))
            }
            189 => eval_gammaln_precise_surface(args, resolver)
                .map_err(|e| map_special_dist_error_to_ws(&e)),
    // FUNC.GCD
    190 => eval_gcd_surface(args, resolver).map_err(|e| map_gcd_error_to_ws(&e)),
    // FUNC.GEOMEAN
    191 => {
                eval_geomean_surface(args, resolver).map_err(|e| map_geomean_error_to_ws(&e))
            }
            192 => {
                eval_gestep_surface(args, resolver).map_err(|e| map_gestep_error_to_ws(&e))
            }
            193 => eval_groupby_surface(args, resolver, callable_invoker)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
    // FUNC.GROWTH
    194 => eval_growth_surface(args, resolver)
                .map_err(|e| map_regression_forecast_error_to_ws(&e)),
    // FUNC.HARMEAN
    195 => {
                eval_harmean_surface(args, resolver).map_err(|e| map_harmean_error_to_ws(&e))
            }
            196 => {
                eval_hyperlink_surface(args, resolver).map_err(|e| map_hyperlink_error_to_ws(&e))
            }
            197 => {
                eval_image_surface(args, resolver, host_info).map_err(|e| map_image_error_to_ws(&e))
            }
            199 => eval_hypgeom_dist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
    // FUNC.HYPGEOMDIST
    200 => eval_hypgeomdist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
    // FUNC.INFO
    201 => {
                eval_info_surface(args, resolver, host_info).map_err(|e| map_info_error_to_ws(&e))
            }
            202 => eval_isomitted_surface(args, resolver)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
    // FUNC.IRR
    203 => {
                eval_irr_surface(args, resolver).map_err(|e| map_cashflow_rate_error_to_ws(&e))
            }
            214 => {
                eval_imabs_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            215 => eval_imaginary_surface(args, resolver)
                .map_err(|e| map_complex_family_error_to_ws(&e)),
    // FUNC.INTRATE
    205 => eval_intrate_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
    // FUNC.ISPMT
    206 => eval_ispmt_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
    // FUNC.JIS
    207 => eval_jis_surface(args, resolver, host_info)
                .map_err(|e| map_text_compat_locale_error_to_ws(&e)),
    // FUNC.IFERROR
    209 => {
                eval_iferror_surface(args, resolver).map_err(|e| map_iferror_error_to_ws(&e))
            }
            210 => eval_ifna_surface(args, resolver).map_err(|e| map_ifna_error_to_ws(&e)),
    // FUNC.IFS
    211 => {
                eval_ifs_surface(args, resolver).map_err(|e| map_choose_ifs_error_to_ws(&e))
            }
            212 => {
                eval_index_surface(args, resolver).map_err(|e| map_index_error_to_ws(&e))
            }
            204 => eval_ipmt_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
    // FUNC.IMARGUMENT
    216 => eval_imargument_surface(args, resolver)
                .map_err(|e| map_complex_family_error_to_ws(&e)),
    // FUNC.IMCONJUGATE
    217 => eval_imconjugate_surface(args, resolver)
                .map_err(|e| map_complex_family_error_to_ws(&e)),
    // FUNC.IMCOS
    218 => {
                eval_imcos_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            219 => {
                eval_imcosh_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            220 => {
                eval_imcot_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            221 => {
                eval_imcsc_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            222 => {
                eval_imcsch_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            223 => {
                eval_imdiv_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            224 => {
                eval_imexp_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            225 => {
                eval_imln_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            226 => {
                eval_imlog10_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            227 => {
                eval_imlog2_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            228 => {
                eval_impower_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            229 => eval_improduct_surface(args, resolver)
                .map_err(|e| map_complex_family_error_to_ws(&e)),
    // FUNC.IMREAL
    230 => {
                eval_imreal_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            231 => {
                eval_imsec_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            232 => {
                eval_imsech_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            233 => {
                eval_imsin_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            234 => {
                eval_imsinh_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            235 => {
                eval_imsqrt_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            236 => {
                eval_imsub_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            237 => {
                eval_imsum_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            238 => {
                eval_imtan_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            281 => eval_isformula_surface(args, host_info)
                .map_err(|e| map_misc_switch_info_error_to_ws(&e)),
    // FUNC.ISERR
    241 => eval_iserr_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
    // FUNC.ISERROR
    242 => eval_iserror_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
    // FUNC.ISLOGICAL
    243 => eval_islogical_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
    // FUNC.ISNA
    244 => eval_isna_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
    // FUNC.ISNONTEXT
    245 => eval_isnontext_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
    // FUNC.ISODD
    246 => eval_isodd_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
    // FUNC.ISNUMBER
    247 => {
                eval_isnumber_surface(args, resolver).map_err(|e| map_isnumber_error_to_ws(&e))
            }
            240 => eval_isblank_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
    // FUNC.ISREF
    248 => eval_isref_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
    // FUNC.ISTEXT
    249 => eval_istext_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
    // FUNC.ISO.CEILING
    250 => eval_iso_ceiling_surface(args, resolver)
                .map_err(|e| map_ceiling_floor_error_to_ws(&e)),
    // FUNC.ISEVEN
    252 => {
                eval_iseven_surface(args, resolver).map_err(|e| map_iseven_error_to_ws(&e))
            }
            253 => {
                eval_kurt_surface(args, resolver).map_err(|e| map_moment_stats_error_to_ws(&e))
            }
            257 => {
                eval_large_surface(args, resolver).map_err(|e| map_large_error_to_ws(&e))
            }
            258 => eval_lcm_surface(args, resolver).map_err(|e| map_lcm_error_to_ws(&e)),
    // FUNC.HEX2BIN
    254 => eval_hex2bin_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
    // FUNC.HEX2DEC
    255 => eval_hex2dec_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
    // FUNC.HEX2OCT
    256 => eval_hex2oct_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
    // FUNC.LINEST
    259 => eval_linest_surface(args, resolver)
                .map_err(|e| map_regression_forecast_error_to_ws(&e)),
    // FUNC.LOGINV
    261 => eval_loginv_surface(args, resolver)
                .map_err(|e| map_legacy_stats_alias_error_to_ws(&e)),
    // FUNC.LN
    262 => eval_ln_surface(args, resolver).map_err(|e| map_ln_error_to_ws(&e)),
    // FUNC.LOOKUP
    263 => eval_lookup_surface(args, resolver)
                .map_err(|e| map_lookup_prob_frequency_error_to_ws(&e)),
    // FUNC.LOG10
    265 => {
                eval_log10_surface(args, resolver).map_err(|e| map_log10_error_to_ws(&e))
            }
            267 => {
                eval_lower_surface(args, resolver).map_err(|e| map_text_scalar_error_to_ws(&e))
            }
            268 => {
                eval_left_surface(args, resolver).map_err(|e| map_text_slice_error_to_ws(&e))
            }
            269 => {
                eval_leftb_surface(args, resolver).map_err(|e| map_text_b_compat_error_to_ws(&e))
            }
            270 => {
                eval_len_surface(args, resolver).map_err(|e| map_text_slice_error_to_ws(&e))
            }
            271 => {
                eval_lenb_surface(args, resolver).map_err(|e| map_text_b_compat_error_to_ws(&e))
            }
            272 => {
                eval_mid_surface(args, resolver).map_err(|e| map_text_slice_error_to_ws(&e))
            }
            273 => {
                eval_midb_surface(args, resolver).map_err(|e| map_text_b_compat_error_to_ws(&e))
            }
            274 => {
                eval_right_surface(args, resolver).map_err(|e| map_text_slice_error_to_ws(&e))
            }
            275 => {
                eval_rightb_surface(args, resolver).map_err(|e| map_text_b_compat_error_to_ws(&e))
            }
            276 => eval_max_surface(args, resolver).map_err(|e| map_max_error_to_ws(&e)),
    // FUNC.LOGEST
    266 => eval_logest_surface(args, resolver)
                .map_err(|e| map_regression_forecast_error_to_ws(&e)),
    // FUNC.MAXA
    277 => eval_maxa_surface(args, resolver).map_err(|e| map_maxa_error_to_ws(&e)),
    // FUNC.MAXIFS
    278 => {
                eval_maxifs_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            279 => {
                eval_median_surface(args, resolver).map_err(|e| map_median_error_to_ws(&e))
            }
            282 => {
                if args.len() < 2 {
                    return Err(WorksheetErrorCode::Value);
                }
                let lookup_array = singleton_arg_slice(&args[1]);
                let match_type = args.get(2);
                eval_match_surface(&args[0], &lookup_array, match_type, resolver)
                    .map_err(|e| map_match_error_to_ws(&e))
            }
            283 => eval_makearray_surface(args, resolver, callable_invoker)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
    // FUNC.MAP
    284 => eval_map_surface(args, resolver, callable_invoker)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
    // FUNC.MDETERM
    285 => {
                eval_mdeterm_surface(args, resolver).map_err(|e| map_matrix_error_to_ws(&e))
            }
            286 => {
                eval_mduration_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            287 => {
                eval_minverse_surface(args, resolver).map_err(|e| map_matrix_error_to_ws(&e))
            }
            288 => {
                eval_mmult_surface(args, resolver).map_err(|e| map_matrix_error_to_ws(&e))
            }
            289 => {
                eval_munit_surface(args, resolver).map_err(|e| map_matrix_error_to_ws(&e))
            }
            293 => eval_mod_surface(args, resolver).map_err(|e| map_mod_error_to_ws(&e)),
    // FUNC.MIN
    290 => eval_min_surface(args, resolver).map_err(|e| map_min_error_to_ws(&e)),
    // FUNC.MINA
    291 => eval_mina_surface(args, resolver).map_err(|e| map_mina_error_to_ws(&e)),
    // FUNC.MINIFS
    292 => {
                eval_minifs_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            280 => eval_mirr_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
    // FUNC.MODE.MULT
    294 => eval_mode_mult_surface(args, resolver)
                .map_err(|e| map_lookup_prob_frequency_error_to_ws(&e)),
    // FUNC.MODE.SNGL
    295 => {
                eval_mode_sngl_surface(args, resolver).map_err(|e| map_mode_sngl_error_to_ws(&e))
            }
            296 => {
                eval_month_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            297 => {
                eval_mround_surface(args, resolver).map_err(|e| map_mround_error_to_ws(&e))
            }
            298 => eval_multinomial_surface(args, resolver)
                .map_err(|e| map_multinomial_error_to_ws(&e)),
    // FUNC.NA
    300 => eval_na_surface(args),
    // FUNC.NEGBINOM.DIST
    301 => eval_negbinom_dist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
    // FUNC.NEGBINOMDIST
    302 => eval_negbinomdist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
    // FUNC.NOMINAL
    303 => eval_nominal_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
    // FUNC.NPER
    304 => eval_nper_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
    // FUNC.NPV
    305 => eval_npv_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
    // FUNC.NUMBERVALUE
    306 => eval_numbervalue_surface(args, resolver, locale_ctx)
                .map_err(|e| map_number_regex_translate_error_to_ws(&e)),
    // FUNC.CONFIDENCE
    307 => {
                eval_confidence_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            308 => eval_confidence_t_surface(args, resolver)
                .map_err(|e| map_confidence_test_error_to_ws(&e)),
    // FUNC.CONFIDENCE.NORM
    309 => eval_confidence_norm_surface(args, resolver)
                .map_err(|e| map_normal_log_error_to_ws(&e)),
    // FUNC.LOGNORM.DIST
    310 => eval_lognorm_dist_surface(args, resolver)
                .map_err(|e| map_normal_log_error_to_ws(&e)),
    // FUNC.LOGNORM.INV
    311 => {
                eval_lognorm_inv_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            312 => {
                eval_lognormdist_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            313 => {
                eval_norm_dist_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            314 => {
                eval_norm_inv_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            315 => {
                eval_norm_s_dist_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            316 => {
                eval_norm_s_inv_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            317 => {
                eval_normdist_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            318 => {
                eval_norminv_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            319 => {
                eval_normsdist_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            320 => {
                eval_normsinv_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            321 => eval_networkdays_surface(args, resolver)
                .map_err(|e| map_workday_networkdays_error_to_ws(&e)),
    // FUNC.NETWORKDAYS.INTL
    322 => eval_networkdays_intl_surface(args, resolver)
                .map_err(|e| map_workday_networkdays_error_to_ws(&e)),
    // FUNC.NOT
    324 => eval_not_surface(args, resolver).map_err(|e| map_not_error_to_ws(&e)),
    // FUNC.NOW
    325 => {
                let serial = now_serial.ok_or(WorksheetErrorCode::Value)?;
                let provider = FixedNowProvider { serial };
                eval_now_surface(args, &provider).map_err(|e| map_now_error_to_ws(&e))
            }
            326 => eval_oct2bin_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
    // FUNC.OCT2DEC
    327 => eval_oct2dec_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
    // FUNC.OCT2HEX
    328 => eval_oct2hex_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
    // FUNC.ODDFPRICE
    330 => {
                eval_oddfprice_surface(args, resolver).map_err(|e| map_odd_bond_error_to_ws(&e))
            }
            331 => {
                eval_oddfyield_surface(args, resolver).map_err(|e| map_odd_bond_error_to_ws(&e))
            }
            332 => {
                eval_oddlprice_surface(args, resolver).map_err(|e| map_odd_bond_error_to_ws(&e))
            }
            333 => {
                eval_oddlyield_surface(args, resolver).map_err(|e| map_odd_bond_error_to_ws(&e))
            }
            334 => eval_or_surface(args, resolver).map_err(|e| map_or_error_to_ws(&e)),
    // FUNC.PHI
    336 => eval_phi_surface(args, resolver).map_err(|e| map_phi_error_to_ws(&e)),
    // FUNC.PERCENTILE.EXC
    337 => eval_percentile_exc_surface(args, resolver)
                .map_err(|e| map_percentile_exc_error_to_ws(&e)),
    // FUNC.PERCENTILE.INC
    338 => eval_percentile_inc_surface(args, resolver)
                .map_err(|e| map_percentile_inc_error_to_ws(&e)),
    // FUNC.PERCENTILE
    339 => eval_percentile_surface(args, resolver)
                .map_err(|e| map_legacy_stats_alias_error_to_ws(&e)),
    // FUNC.PERCENTRANK.EXC
    340 => eval_percentrank_exc_surface(args, resolver)
                .map_err(|e| map_percentrank_exc_error_to_ws(&e)),
    // FUNC.PERCENTRANK.INC
    341 => eval_percentrank_inc_surface(args, resolver)
                .map_err(|e| map_percentrank_inc_error_to_ws(&e)),
    // FUNC.PERCENTRANK
    342 => eval_percentrank_surface(args, resolver)
                .map_err(|e| map_legacy_stats_alias_error_to_ws(&e)),
    // FUNC.POISSON
    344 => {
                eval_poisson_surface(args, resolver).map_err(|e| map_discrete_dist_error_to_ws(&e))
            }
            345 => eval_poisson_dist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
    // FUNC.OFFSET
    346 => {
                eval_offset_surface(args, resolver).map_err(|e| map_offset_error_to_ws(&e))
            }
            335 => {
                eval_pearson_surface(args, resolver).map_err(|e| map_pearson_error_to_ws(&e))
            }
            343 => eval_pduration_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
    // FUNC.OP_ADD
    347 => {
                eval_op_add_surface(args, resolver).map_err(|e| map_op_add_error_to_ws(&e))
            }
            348 => eval_op_concat_surface(args, resolver)
                .map_err(|e| map_operator_compare_concat_error_to_ws(&e)),
    // FUNC.OP_DIVIDE
    349 => eval_op_divide_surface(args, resolver)
                .map_err(|e| map_operator_binary_error_to_ws(&e)),
    // FUNC.OP_EQUAL
    350 => eval_op_equal_surface(args, resolver)
                .map_err(|e| map_operator_compare_concat_error_to_ws(&e)),
    // FUNC.OP_GREATER_EQUAL
    351 => eval_op_greater_equal_surface(args, resolver)
                .map_err(|e| map_operator_compare_concat_error_to_ws(&e)),
    // FUNC.OP_GREATER_THAN
    352 => eval_op_greater_than_surface(args, resolver)
                .map_err(|e| map_operator_compare_concat_error_to_ws(&e)),
    // FUNC.OP_LESS_EQUAL
    354 => eval_op_less_equal_surface(args, resolver)
                .map_err(|e| map_operator_compare_concat_error_to_ws(&e)),
    // FUNC.OP_LESS_THAN
    355 => eval_op_less_than_surface(args, resolver)
                .map_err(|e| map_operator_compare_concat_error_to_ws(&e)),
    // FUNC.OP_MULTIPLY
    356 => eval_op_multiply_surface(args, resolver)
                .map_err(|e| map_operator_binary_error_to_ws(&e)),
    // FUNC.OP_NEGATE
    357 => eval_op_negate_surface(args, resolver)
                .map_err(|e| map_operator_unary_error_to_ws(&e)),
    // FUNC.OP_NOT_EQUAL
    358 => eval_op_not_equal_surface(args, resolver)
                .map_err(|e| map_operator_compare_concat_error_to_ws(&e)),
    // FUNC.OP_PERCENT
    359 => eval_op_percent_surface(args, resolver)
                .map_err(|e| map_operator_unary_error_to_ws(&e)),
    // FUNC.OP_POWER
    360 => eval_op_power_surface(args, resolver)
                .map_err(|e| map_operator_binary_error_to_ws(&e)),
    // FUNC.OP_RANGE_REF
    361 => eval_op_range_ref_surface(args, resolver)
                .map_err(|e| map_operator_reference_error_to_ws(&e)),
    // FUNC.OP_SPILL_REF
    362 => eval_op_spill_ref_surface(args, resolver)
                .map_err(|e| map_op_spill_ref_error_to_ws(&e)),
    // FUNC.OP_SUBTRACT
    363 => eval_op_subtract_surface(args, resolver)
                .map_err(|e| map_operator_binary_error_to_ws(&e)),
    // FUNC.OP_TRIM_REF_BOTH
    364 => eval_op_trim_ref_both_surface(args, resolver)
                .map_err(|e| map_operator_reference_error_to_ws(&e)),
    // FUNC.OP_TRIM_REF_LEADING
    365 => eval_op_trim_ref_leading_surface(args, resolver)
                .map_err(|e| map_operator_reference_error_to_ws(&e)),
    // FUNC.OP_TRIM_REF_TRAILING
    366 => eval_op_trim_ref_trailing_surface(args, resolver)
                .map_err(|e| map_operator_reference_error_to_ws(&e)),
    // FUNC.OP_UNARY_PLUS
    367 => eval_op_unary_plus_surface(args, resolver)
                .map_err(|e| map_operator_unary_error_to_ws(&e)),
    // FUNC.OP_UNION_REF
    368 => eval_op_union_ref_surface(args, resolver)
                .map_err(|e| map_operator_reference_error_to_ws(&e)),
    // FUNC.PERMUT
    369 => {
                eval_permut_surface(args, resolver).map_err(|e| map_permut_error_to_ws(&e))
            }
            370 => eval_permutationa_surface(args, resolver)
                .map_err(|e| map_permutationa_error_to_ws(&e)),
    // FUNC.PI
    371 => {
                if !args.is_empty() {
                    return Err(WorksheetErrorCode::Value);
                }
                let pi_args: Vec<Value> = Vec::new();
                match eval_pi(&pi_args) {
                    Ok(Value::Number(n)) => Ok(EvalValue::Number(n)),
                    Ok(Value::Error(_)) => Err(WorksheetErrorCode::Value),
                    Err(e) => Err(map_eval_error_to_ws(&e)),
                }
            }
            372 => eval_pivotby_surface(args, resolver, callable_invoker)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
    // FUNC.PMT
    373 => eval_pmt_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
    // FUNC.PPMT
    374 => eval_ppmt_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
    // FUNC.PERCENTOF
    375 => eval_percentof_surface(args, resolver)
                .map_err(|e| map_misc_conversion_error_to_ws(&e)),
    // FUNC.PRICE
    376 => {
                eval_price_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            377 => eval_pricedisc_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
    // FUNC.PRICEMAT
    378 => {
                eval_pricemat_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            379 => eval_prob_surface(args, resolver)
                .map_err(|e| map_lookup_prob_frequency_error_to_ws(&e)),
    // FUNC.PRODUCT
    380 => {
                eval_product_surface(args, resolver).map_err(|e| map_product_error_to_ws(&e))
            }
            436 => crate::functions::subtotal_aggregate_family::eval_subtotal_surface(
                args, resolver, host_info,
            )
            .map_err(|e| {
                crate::functions::subtotal_aggregate_family::map_subtotal_aggregate_error_to_ws(&e)
            }),
    // FUNC.POWER
    381 => {
                eval_power_surface(args, resolver).map_err(|e| map_power_error_to_ws(&e))
            }
            384 => {
                eval_quotient_surface(args, resolver).map_err(|e| map_quotient_error_to_ws(&e))
            }
    // FUNC.PROPER
    382 => eval_proper_surface(args, resolver)
                .map_err(|e| map_text_search_replace_error_to_ws(&e)),
    // FUNC.PV
    383 => eval_pv_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
    // FUNC.QUARTILE.INC
    386 => eval_quartile_inc_surface(args, resolver)
                .map_err(|e| map_quartile_inc_error_to_ws(&e)),
    // FUNC.QUARTILE
    387 => eval_quartile_surface(args, resolver)
                .map_err(|e| map_legacy_stats_alias_error_to_ws(&e)),
    // FUNC.RADIANS
    388 => {
                eval_radians_surface(args, resolver).map_err(|e| map_radians_error_to_ws(&e))
            }
            264 => eval_log_surface(args, resolver).map_err(|e| map_log_error_to_ws(&e)),
    // FUNC.RANK
    389 => eval_rank_surface(args, resolver).map_err(|e| map_rank_error_to_ws(&e)),
    // FUNC.RANK.AVG
    390 => {
                eval_rank_avg_surface(args, resolver).map_err(|e| map_rank_avg_error_to_ws(&e))
            }
            391 => {
                eval_rank_eq_surface(args, resolver).map_err(|e| map_rank_eq_error_to_ws(&e))
            }
            385 => eval_quartile_exc_surface(args, resolver)
                .map_err(|e| map_quartile_exc_error_to_ws(&e)),
    // FUNC.RAND
    392 => {
                let value = random_value.ok_or(WorksheetErrorCode::Value)?;
                let provider = FixedRandomProvider { value };
                eval_rand_surface(args, &provider).map_err(|e| map_rand_error_to_ws(&e))
            }
            393 => {
                let value = random_value.ok_or(WorksheetErrorCode::Value)?;
                let provider = FixedRandomProvider { value };
                eval_randbetween_surface(args, resolver, &provider)
                    .map_err(|e| map_randbetween_error_to_ws(&e))
            }
            395 => eval_rate_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
    // FUNC.RANDARRAY
    394 => {
                let value = random_value.ok_or(WorksheetErrorCode::Value)?;
                let provider = FixedRandomProvider { value };
                eval_randarray_surface(args, resolver, &provider)
                    .map_err(|e| map_misc_conversion_error_to_ws(&e))
            }
            396 => eval_reduce_surface(args, resolver, callable_invoker)
                .map(|value| match value {
                    crate::functions::adapters::PreparedArgValue::Eval(v) => v,
                    crate::functions::adapters::PreparedArgValue::MissingArg => {
                        EvalValue::Text(crate::value::ExcelText::from_utf16_code_units(Vec::new()))
                    }
                    crate::functions::adapters::PreparedArgValue::EmptyCell => {
                        EvalValue::Array(crate::value::EvalArray::from_scalar(
                            crate::value::ArrayCellValue::EmptyCell,
                        ))
                    }
                })
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
    // FUNC.REGISTER.ID
    397 => {
                eval_register_id_surface(args, resolver, registered_external_provider)
                    .map_err(|e| map_call_register_id_error_to_ws(&e))
            }
            404 => {
                eval_roundup_surface(args, resolver).map_err(|e| map_roundup_error_to_ws(&e))
            }
            398 => {
                eval_roman_surface(args, resolver).map_err(|e| map_roman_error_to_ws(&e))
            }
            401 => eval_rsq_surface(args, resolver).map_err(|e| map_rsq_error_to_ws(&e)),
    // FUNC.ROW
    399 => eval_row_surface(args, resolver).map_err(|e| map_row_error_to_ws(&e)),
    // FUNC.ROWS
    400 => eval_rows_surface(args).map_err(|e| map_rows_error_to_ws(&e)),
    // FUNC.REPLACEB
    406 => {
                eval_replaceb_surface(args, resolver).map_err(|e| map_text_b_compat_error_to_ws(&e))
            }
            407 => eval_received_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
    // FUNC.REGEXEXTRACT
    408 => eval_regexextract_surface(args, resolver)
                .map_err(|e| map_number_regex_translate_error_to_ws(&e)),
    // FUNC.REGEXREPLACE
    409 => eval_regexreplace_surface(args, resolver)
                .map_err(|e| map_number_regex_translate_error_to_ws(&e)),
    // FUNC.REGEXTEST
    410 => eval_regextest_surface(args, resolver)
                .map_err(|e| map_number_regex_translate_error_to_ws(&e)),
    // FUNC.SERIESSUM
    413 => {
                eval_seriessum_surface(args, resolver).map_err(|e| map_sumproduct_error_to_ws(&e))
            }
            329 => eval_odd_surface(args, resolver).map_err(|e| map_odd_error_to_ws(&e)),
    // FUNC.SEQUENCE
    414 => {
                eval_sequence_surface(args, resolver).map_err(|e| map_sequence_error_to_ws(&e))
            }
            411 => eval_scan_surface(args, resolver, callable_invoker)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
    // FUNC.SECH
    415 => eval_sech_surface(args, resolver).map_err(|e| map_sech_error_to_ws(&e)),
    // FUNC.SIGN
    416 => eval_sign_surface(args, resolver).map_err(|e| map_sign_error_to_ws(&e)),
    // FUNC.SORTBY
    418 => eval_sortby_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
    // FUNC.SIN
    419 => eval_sin_surface(args, resolver).map_err(|e| map_sin_error_to_ws(&e)),
    // FUNC.SINH
    420 => eval_sinh_surface(args, resolver).map_err(|e| map_sinh_error_to_ws(&e)),
    // FUNC.SKEW
    421 => {
                eval_skew_surface(args, resolver).map_err(|e| map_moment_stats_error_to_ws(&e))
            }
            422 => {
                eval_skew_p_surface(args, resolver).map_err(|e| map_moment_stats_error_to_ws(&e))
            }
            425 => {
                eval_sln_surface(args, resolver).map_err(|e| map_depreciation_error_to_ws(&e))
            }
            426 => {
                eval_small_surface(args, resolver).map_err(|e| map_small_error_to_ws(&e))
            }
            427 => eval_sqrt_surface(args, resolver).map_err(|e| map_sqrt_error_to_ws(&e)),
    // FUNC.SQRTPI
    428 => {
                eval_sqrtpi_surface(args, resolver).map_err(|e| map_sqrtpi_error_to_ws(&e))
            }
            424 => {
                eval_slope_surface(args, resolver).map_err(|e| map_slope_error_to_ws(&e))
            }
            429 => {
                eval_stdev_surface(args, resolver).map_err(|e| map_stdev_error_to_ws(&e))
            }
            430 => {
                eval_stdev_p_surface(args, resolver).map_err(|e| map_stdev_p_error_to_ws(&e))
            }
            431 => {
                eval_stdev_s_surface(args, resolver).map_err(|e| map_stdev_s_error_to_ws(&e))
            }
            432 => {
                eval_stdevp_surface(args, resolver).map_err(|e| map_stdevp_error_to_ws(&e))
            }
            433 => {
                eval_stdeva_surface(args, resolver).map_err(|e| map_stdeva_error_to_ws(&e))
            }
            434 => {
                eval_stdevpa_surface(args, resolver).map_err(|e| map_stdevpa_error_to_ws(&e))
            }
            423 => {
                eval_steyx_surface(args, resolver).map_err(|e| map_moment_stats_error_to_ws(&e))
            }
            435 => eval_standardize_surface(args, resolver)
                .map_err(|e| map_standardize_error_to_ws(&e)),
    // FUNC.SUM
    437 => eval_sum_surface(args, resolver).map_err(|e| map_sum_error_to_ws(&e)),
    // FUNC.SUMIF
    438 => {
                eval_sumif_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            439 => {
                eval_sumifs_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            441 => {
                eval_sumproduct_surface(args, resolver).map_err(|e| map_sumproduct_error_to_ws(&e))
            }
            442 => {
                eval_sumx2my2_surface(args, resolver).map_err(|e| map_sumproduct_error_to_ws(&e))
            }
            443 => {
                eval_sumx2py2_surface(args, resolver).map_err(|e| map_sumproduct_error_to_ws(&e))
            }
            444 => {
                eval_sumxmy2_surface(args, resolver).map_err(|e| map_sumproduct_error_to_ws(&e))
            }
            440 => {
                eval_sumsq_surface(args, resolver).map_err(|e| map_sumsq_error_to_ws(&e))
            }
            445 => eval_switch_surface(args, resolver)
                .map_err(|e| map_misc_switch_info_error_to_ws(&e)),
    // FUNC.T
    446 => eval_t_surface(args, resolver).map_err(|e| map_t_error_to_ws(&e)),
    // FUNC.T.DIST
    447 => {
                eval_t_dist_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            448 => {
                eval_t_dist_2t_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            449 => {
                eval_t_dist_rt_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            450 => {
                eval_t_inv_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            451 => {
                eval_t_inv_2t_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            452 => eval_t_test_surface(args, resolver)
                .map_err(|e| map_statistical_tests_error_to_ws(&e)),
    // FUNC.TAN
    453 => eval_tan_surface(args, resolver).map_err(|e| map_tan_error_to_ws(&e)),
    // FUNC.TANH
    454 => eval_tanh_surface(args, resolver).map_err(|e| map_tanh_error_to_ws(&e)),
    // FUNC.TAKE
    455 => eval_take_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
    // FUNC.TBILLEQ
    456 => eval_tbilleq_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
    // FUNC.TBILLPRICE
    457 => eval_tbillprice_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
    // FUNC.TBILLYIELD
    458 => eval_tbillyield_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
    // FUNC.TOCOL
    459 => eval_tocol_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
    // FUNC.TOROW
    460 => eval_torow_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
    // FUNC.TDIST
    462 => {
                eval_tdist_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            482 => {
                eval_tinv_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            518 => {
                eval_syd_surface(args, resolver).map_err(|e| map_depreciation_error_to_ws(&e))
            }
            208 => eval_if_surface(args, resolver).map_err(|e| map_if_error_to_ws(&e)),
    // FUNC.SEARCH
    467 => eval_search_surface(args, resolver)
                .map_err(|e| map_text_search_replace_error_to_ws(&e)),
    // FUNC.SEARCHB
    468 => {
                eval_searchb_surface(args, resolver).map_err(|e| map_text_b_compat_error_to_ws(&e))
            }
            466 => {
                let ctx = locale_ctx.ok_or(WorksheetErrorCode::Value)?;
                eval_text_surface(args, resolver, ctx).map_err(|e| map_text_error_to_ws(&e))
            }
            463 => {
                eval_textafter_surface(args, resolver).map_err(|e| map_text_delim_error_to_ws(&e))
            }
            464 => {
                eval_textbefore_surface(args, resolver).map_err(|e| map_text_delim_error_to_ws(&e))
            }
            465 => eval_textsplit_surface(args, resolver)
                .map_err(|e| map_array_text_split_error_to_ws(&e)),
    // FUNC.SHEET
    469 => eval_sheet_surface(args, resolver, host_info)
                .map_err(|e| map_reference_metadata_error_to_ws(&e)),
    // FUNC.SHEETS
    470 => eval_sheets_surface(args, host_info)
                .map_err(|e| map_reference_metadata_error_to_ws(&e)),
    // FUNC.REPT
    471 => {
                eval_rept_surface(args, resolver).map_err(|e| map_text_scalar_error_to_ws(&e))
            }
            472 => eval_substitute_surface(args, resolver)
                .map_err(|e| map_text_search_replace_error_to_ws(&e)),
    // FUNC.TEXTJOIN
    473 => {
                eval_textjoin_surface(args, resolver).map_err(|e| map_textjoin_error_to_ws(&e))
            }
            474 => {
                let serial = now_serial.ok_or(WorksheetErrorCode::Value)?;
                let provider = FixedNowProvider { serial };
                eval_today_surface(args, &provider).map_err(|e| map_today_error_to_ws(&e))
            }
            133 => {
                eval_time_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            475 => eval_timevalue_surface(args, resolver)
                .map_err(|e| map_date_value_family_error_to_ws(&e)),
    // FUNC.TRANSLATE
    476 => eval_translate_surface(args, resolver, host_info)
                .map_err(|e| map_number_regex_translate_error_to_ws(&e)),
    // FUNC.TRIMMEAN
    478 => {
                eval_trimmean_surface(args, resolver).map_err(|e| map_moment_stats_error_to_ws(&e))
            }
            477 => eval_transpose_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
    // FUNC.TRUE
    479 => eval_true_surface(args),
    // FUNC.TREND
    481 => eval_trend_surface(args, resolver)
                .map_err(|e| map_regression_forecast_error_to_ws(&e)),
    // FUNC.TRUNC
    483 => {
                eval_trunc_surface(args, resolver).map_err(|e| map_trunc_error_to_ws(&e))
            }
            480 => {
                eval_trimrange_surface(args, resolver).map_err(|e| map_trimrange_error_to_ws(&e))
            }
            484 => {
                eval_trim_surface(args, resolver).map_err(|e| map_text_scalar_error_to_ws(&e))
            }
            485 => eval_ttest_surface(args, resolver)
                .map_err(|e| map_statistical_tests_error_to_ws(&e)),
    // FUNC.TYPE
    486 => eval_type_surface(args, resolver).map_err(|e| map_type_error_to_ws(&e)),
    // FUNC.UNIQUE
    487 => eval_unique_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
    // FUNC.UNICHAR
    488 => {
                eval_unichar_surface(args, resolver).map_err(|e| map_text_unicode_error_to_ws(&e))
            }
            489 => {
                eval_unicode_surface(args, resolver).map_err(|e| map_text_unicode_error_to_ws(&e))
            }
            490 => {
                eval_upper_surface(args, resolver).map_err(|e| map_text_scalar_error_to_ws(&e))
            }
            491 => {
                let ctx = locale_ctx.ok_or(WorksheetErrorCode::Value)?;
                eval_value_surface(args, resolver, ctx).map_err(|e| map_value_error_to_ws(&e))
            }
            492 => eval_valuetotext_surface(args, resolver)
                .map_err(|e| map_valuetotext_error_to_ws(&e)),
    // FUNC.VAR
    493 => eval_var_surface(args, resolver).map_err(|e| map_var_error_to_ws(&e)),
    // FUNC.VAR.P
    494 => {
                eval_var_p_surface(args, resolver).map_err(|e| map_var_p_error_to_ws(&e))
            }
            495 => {
                eval_var_s_surface(args, resolver).map_err(|e| map_var_s_error_to_ws(&e))
            }
            496 => eval_vara_surface(args, resolver).map_err(|e| map_vara_error_to_ws(&e)),
    // FUNC.VARP
    497 => eval_varp_surface(args, resolver).map_err(|e| map_varp_error_to_ws(&e)),
    // FUNC.VARPA
    498 => {
                eval_varpa_surface(args, resolver).map_err(|e| map_varpa_error_to_ws(&e))
            }
            501 => {
                eval_vdb_surface(args, resolver).map_err(|e| map_depreciation_error_to_ws(&e))
            }
            516 => eval_wrapcols_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
    // FUNC.RRI
    499 => eval_rri_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
    // FUNC.RTD
    500 => {
                eval_rtd_surface(args, resolver, rtd_provider).map_err(|e| map_rtd_error_to_ws(&e))
            }
            402 => {
                eval_round_surface(args, resolver).map_err(|e| map_round_error_to_ws(&e))
            }
            403 => {
                eval_rounddown_surface(args, resolver).map_err(|e| map_rounddown_error_to_ws(&e))
            }
            405 => eval_replace_surface(args, resolver)
                .map_err(|e| map_text_search_replace_error_to_ws(&e)),
    // FUNC.VSTACK
    502 => eval_vstack_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
    // FUNC.HLOOKUP
    503 => {
                eval_hlookup_surface(args, resolver).map_err(|e| map_vhlookup_error_to_ws(&e))
            }
            504 => {
                eval_vlookup_surface(args, resolver).map_err(|e| map_vhlookup_error_to_ws(&e))
            }
            514 => {
                eval_weibull_surface(args, resolver).map_err(|e| map_special_dist_error_to_ws(&e))
            }
            515 => eval_weibull_dist_surface(args, resolver)
                .map_err(|e| map_special_dist_error_to_ws(&e)),
    // FUNC.WORKDAY.INTL
    506 => eval_workday_intl_surface(args, resolver)
                .map_err(|e| map_workday_networkdays_error_to_ws(&e)),
    // FUNC.XLOOKUP
    507 => {
                if args.len() < 3 {
                    return Err(WorksheetErrorCode::Value);
                }
                let lookup_array = singleton_arg_slice(&args[1]);
                let return_array = singleton_arg_slice(&args[2]);
                eval_xlookup_surface(
                    &args[0],
                    &lookup_array,
                    &return_array,
                    args.get(3),
                    args.get(4),
                    args.get(5),
                    resolver,
                )
                .map_err(|e| map_xlookup_error_to_ws(&e))
            }
            213 => {
                eval_indirect_surface(args, resolver).map_err(|e| map_indirect_error_to_ws(&e))
            }
            239 => {
                eval_intercept_surface(args, resolver).map_err(|e| map_intercept_error_to_ws(&e))
            }
            251 => eval_int_surface(args, resolver).map_err(|e| map_int_error_to_ws(&e)),
    // FUNC.XIRR
    508 => {
                eval_xirr_surface(args, resolver).map_err(|e| map_cashflow_rate_error_to_ws(&e))
            }
            509 => {
                eval_xnpv_surface(args, resolver).map_err(|e| map_cashflow_rate_error_to_ws(&e))
            }
            511 => eval_xor_surface(args, resolver).map_err(|e| map_xor_error_to_ws(&e)),
    // FUNC.WEEKDAY
    512 => {
                eval_weekday_surface(args, resolver).map_err(|e| map_date_week_error_to_ws(&e))
            }
            513 => {
                eval_weeknum_surface(args, resolver).map_err(|e| map_date_week_error_to_ws(&e))
            }
            505 => eval_workday_surface(args, resolver)
                .map_err(|e| map_workday_networkdays_error_to_ws(&e)),
    // FUNC.WRAPROWS
    517 => eval_wraprows_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
    // FUNC.YIELD
    519 => {
                eval_yield_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            520 => {
                eval_yielddisc_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            521 => {
                eval_yieldmat_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            510 => {
                if args.len() < 2 {
                    return Err(WorksheetErrorCode::Value);
                }
                let lookup_array = singleton_arg_slice(&args[1]);
                eval_xmatch_surface_value(
                    &args[0],
                    &lookup_array,
                    args.get(2),
                    args.get(3),
                    resolver,
                )
                .map_err(|e| map_xmatch_error_to_ws(&e))
            }
            524 => {
                eval_z_test_surface(args, resolver).map_err(|e| map_confidence_test_error_to_ws(&e))
            }
            461 => {
                eval_ztest_surface(args, resolver).map_err(|e| map_test_alias_error_to_ws(&e))
            }
            522 => {
                eval_year_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            523 => eval_yearfrac_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
    // FUNC.OP_IMPLICIT_INTERSECTION
    525 => {
                eval_op_implicit_intersection_surface(args, resolver)
                    .map_err(|e| map_op_implicit_intersection_error_to_ws(&e))
            }
            353 => eval_op_intersection_ref_surface(args, resolver)
                .map_err(|e| map_operator_reference_error_to_ws(&e)),
    _ => Err(WorksheetErrorCode::Value),
}
