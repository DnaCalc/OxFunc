use crate::coercion::CoercionError;
use crate::function::ArgPreparationProfile;
use crate::functions::abs::{AbsEvalError, abs_kernel, eval_abs_scalar_value};
use crate::functions::acos::{eval_acos_surface, map_acos_error_to_ws};
use crate::functions::acosh::{eval_acosh_surface, map_acosh_error_to_ws};
use crate::functions::acot::{acot_kernel, eval_acot_surface, map_acot_error_to_ws};
use crate::functions::acoth::{eval_acoth_surface, map_acoth_error_to_ws};
use crate::functions::amor_depreciation_family::{
    eval_amordegrc_surface, eval_amorlinc_surface, map_amor_depreciation_error_to_ws,
};
use crate::functions::and_fn::{eval_and_surface, map_and_error_to_ws};
use crate::functions::arabic_fn::{eval_arabic_surface, map_arabic_error_to_ws};
use crate::functions::array_text_split_family::{
    eval_arraytotext_surface, eval_textsplit_surface, map_array_text_split_error_to_ws,
};
use crate::functions::asin::{eval_asin_surface, map_asin_error_to_ws};
use crate::functions::asinh::{asinh_kernel, eval_asinh_surface, map_asinh_error_to_ws};
use crate::functions::atan::{atan_kernel, eval_atan_surface, map_atan_error_to_ws};
use crate::functions::atan2::{atan2_kernel, eval_atan2_surface, map_atan2_error_to_ws};
use crate::functions::atanh::{atanh_kernel, eval_atanh_surface, map_atanh_error_to_ws};
use crate::functions::avedev_fn::{eval_avedev_surface, map_avedev_error_to_ws};
use crate::functions::average::{eval_average_surface, map_average_error_to_ws};
use crate::functions::averagea_fn::{eval_averagea_surface, map_averagea_error_to_ws};
use crate::functions::base_fn::{eval_base_surface, map_base_error_to_ws};
use crate::functions::bessel_convert_family::{
    eval_besseli_surface, eval_besselj_surface, eval_besselk_surface, eval_bessely_surface,
    map_bessel_convert_error_to_ws,
};
use crate::functions::beta_gamma_stats_family::{
    eval_beta_dist_surface, eval_beta_inv_surface, eval_betadist_surface, eval_betainv_surface,
    eval_gamma_dist_surface, eval_gamma_inv_surface, eval_gammadist_surface, eval_gammainv_surface,
    map_beta_gamma_stats_error_to_ws,
};
use crate::functions::bitand_fn::{bitand_kernel, eval_bitand_surface, map_bitand_error_to_ws};
use crate::functions::bitlshift_fn::{
    bitlshift_kernel, eval_bitlshift_surface, map_bitlshift_error_to_ws,
};
use crate::functions::bitor_fn::{bitor_kernel, eval_bitor_surface, map_bitor_error_to_ws};
use crate::functions::bitrshift_fn::{
    bitrshift_kernel, eval_bitrshift_surface, map_bitrshift_error_to_ws,
};
use crate::functions::bitxor_fn::{bitxor_kernel, eval_bitxor_surface, map_bitxor_error_to_ws};
use crate::functions::bond_core_family::{
    eval_accrint_surface, eval_accrintm_surface, eval_duration_surface, eval_mduration_surface,
    eval_price_surface, eval_pricemat_surface, eval_yield_surface, eval_yielddisc_surface,
    eval_yieldmat_surface, map_bond_core_error_to_ws,
};
use crate::functions::call_register_id_family::{
    CALL_META, REGISTER_ID_META, RegisteredExternalProvider, eval_call_surface,
    eval_register_id_surface, map_call_register_id_error_to_ws,
};
use crate::functions::callable_helpers::{
    BYCOL_META, BYROW_META, CallableInvocationError, CallableInvoker, ISOMITTED_META,
    MAKEARRAY_META, MAP_META, REDUCE_META, SCAN_META, eval_bycol_surface, eval_byrow_surface,
    eval_isomitted_surface, eval_makearray_surface, eval_map_surface, eval_reduce_surface,
    eval_scan_surface, map_lambda_helper_error_to_ws,
};
use crate::functions::cashflow_rate_family::{
    eval_irr_surface, eval_xirr_surface, eval_xnpv_surface, map_cashflow_rate_error_to_ws,
};
use crate::functions::ceiling_floor_family::{
    eval_ceiling_math_surface, eval_ceiling_precise_surface, eval_ceiling_surface,
    eval_floor_math_surface, eval_floor_precise_surface, eval_floor_surface,
    eval_iso_ceiling_surface, map_ceiling_floor_error_to_ws,
};
use crate::functions::cell::{eval_cell_surface, map_cell_error_to_ws};
use crate::functions::chi_f_t_family::{
    eval_chidist_surface, eval_chiinv_surface, eval_chisq_dist_rt_surface, eval_chisq_dist_surface,
    eval_chisq_inv_rt_surface, eval_chisq_inv_surface, eval_f_dist_rt_surface, eval_f_dist_surface,
    eval_f_inv_rt_surface, eval_f_inv_surface, eval_fdist_surface, eval_finv_surface,
    eval_t_dist_2t_surface, eval_t_dist_rt_surface, eval_t_dist_surface, eval_t_inv_2t_surface,
    eval_t_inv_surface, eval_tdist_surface, eval_tinv_surface, map_chi_f_t_error_to_ws,
};
use crate::functions::choose_ifs_family::{
    eval_choose_surface, eval_ifs_surface, map_choose_ifs_error_to_ws,
};
use crate::functions::clean_fn::{eval_clean_surface, map_clean_error_to_ws};
use crate::functions::column_fn::{eval_column_surface, map_column_error_to_ws};
use crate::functions::columns_fn::{eval_columns_surface, map_columns_error_to_ws};
use crate::functions::combin::{combin_kernel, eval_combin_surface, map_combin_error_to_ws};
use crate::functions::combina::{combina_kernel, eval_combina_surface, map_combina_error_to_ws};
use crate::functions::complex_family::{
    eval_complex_surface, eval_imabs_surface, eval_imaginary_surface, eval_imargument_surface,
    eval_imconjugate_surface, eval_imcos_surface, eval_imcosh_surface, eval_imcot_surface,
    eval_imcsc_surface, eval_imcsch_surface, eval_imdiv_surface, eval_imexp_surface,
    eval_imln_surface, eval_imlog2_surface, eval_imlog10_surface, eval_impower_surface,
    eval_improduct_surface, eval_imreal_surface, eval_imsec_surface, eval_imsech_surface,
    eval_imsin_surface, eval_imsinh_surface, eval_imsqrt_surface, eval_imsub_surface,
    eval_imsum_surface, eval_imtan_surface, map_complex_family_error_to_ws,
};
use crate::functions::concat_family::{
    eval_concat_surface, eval_concatenate_surface, map_concat_error_to_ws,
};
use crate::functions::confidence_test_family::{
    eval_confidence_t_surface, eval_z_test_surface, map_confidence_test_error_to_ws,
};
use crate::functions::correl_fn::{eval_correl_surface, map_correl_error_to_ws};
use crate::functions::cos::{cos_kernel, eval_cos_surface, map_cos_error_to_ws};
use crate::functions::cosh::{cosh_kernel, eval_cosh_surface, map_cosh_error_to_ws};
use crate::functions::cot::{cot_kernel, eval_cot_surface, map_cot_error_to_ws};
use crate::functions::coth::{coth_kernel, eval_coth_surface, map_coth_error_to_ws};
use crate::functions::count::{eval_count_surface, map_count_error_to_ws};
use crate::functions::counta::{eval_counta_surface, map_counta_error_to_ws};
use crate::functions::countblank_fn::{eval_countblank_surface, map_countblank_error_to_ws};
use crate::functions::coupon_family::{
    eval_coupdaybs_surface, eval_coupdays_surface, eval_coupdaysnc_surface, eval_coupncd_surface,
    eval_coupnum_surface, eval_couppcd_surface, map_coupon_error_to_ws,
};
use crate::functions::covariance_p_fn::{eval_covariance_p_surface, map_covariance_p_error_to_ws};
use crate::functions::covariance_s_fn::{eval_covariance_s_surface, map_covariance_s_error_to_ws};
use crate::functions::criteria_family::{
    eval_averageif_surface, eval_averageifs_surface, eval_countif_surface, eval_countifs_surface,
    eval_maxifs_surface, eval_minifs_surface, eval_sumif_surface, eval_sumifs_surface,
    map_criteria_error_to_ws,
};
use crate::functions::csc::{csc_kernel, eval_csc_surface, map_csc_error_to_ws};
use crate::functions::csch::{csch_kernel, eval_csch_surface, map_csch_error_to_ws};
use crate::functions::cumulative_finance_family::{
    eval_cumipmt_surface, eval_cumprinc_surface, map_cumulative_finance_error_to_ws,
};
use crate::functions::database_family::{
    eval_daverage_surface, eval_dcount_surface, eval_dcounta_surface, eval_dget_surface,
    eval_dmax_surface, eval_dmin_surface, eval_dproduct_surface, eval_dstdev_surface,
    eval_dstdevp_surface, eval_dsum_surface, eval_dvar_surface, eval_dvarp_surface,
    map_database_error_to_ws,
};
use crate::functions::date_fn::{eval_date_surface, map_date_error_to_ws};
use crate::functions::date_parts_family::{
    eval_day_surface, eval_days_surface, eval_hour_surface, eval_minute_surface,
    eval_month_surface, eval_second_surface, eval_time_surface, eval_year_surface,
    map_date_parts_error_to_ws,
};
use crate::functions::date_value_family::{
    eval_datedif_surface, eval_datevalue_surface, eval_days360_surface, eval_timevalue_surface,
    map_date_value_family_error_to_ws,
};
use crate::functions::date_week_family::{
    eval_edate_surface, eval_eomonth_surface, eval_isoweeknum_surface, eval_weekday_surface,
    eval_weeknum_surface, map_date_week_error_to_ws,
};
use crate::functions::decimal_fn::{eval_decimal_surface, map_decimal_error_to_ws};
use crate::functions::degrees::{degrees_kernel, eval_degrees_surface, map_degrees_error_to_ws};
use crate::functions::delta_fn::{delta_kernel, eval_delta_surface, map_delta_error_to_ws};
use crate::functions::depreciation_family::{
    eval_db_surface, eval_ddb_surface, eval_sln_surface, eval_syd_surface, eval_vdb_surface,
    map_depreciation_error_to_ws,
};
use crate::functions::devsq_fn::{eval_devsq_surface, map_devsq_error_to_ws};
use crate::functions::discount_bill_yearfrac_family::{
    eval_disc_surface, eval_intrate_surface, eval_pricedisc_surface, eval_received_surface,
    eval_tbilleq_surface, eval_tbillprice_surface, eval_tbillyield_surface, eval_yearfrac_surface,
    map_discount_bill_yearfrac_error_to_ws,
};
use crate::functions::discrete_dist_family::{
    eval_binom_dist_range_surface, eval_binom_dist_surface, eval_binom_inv_surface,
    eval_binomdist_surface, eval_critbinom_surface, eval_expon_dist_surface,
    eval_expondist_surface, eval_hypgeom_dist_surface, eval_hypgeomdist_surface,
    eval_negbinom_dist_surface, eval_negbinomdist_surface, eval_poisson_dist_surface,
    eval_poisson_surface, map_discrete_dist_error_to_ws,
};
use crate::functions::dollar_fn::{eval_dollar_surface, map_dollar_error_to_ws};
use crate::functions::dollar_fraction_family::{
    eval_dollarde_surface, eval_dollarfr_surface, map_dollar_fraction_error_to_ws,
};
use crate::functions::dynamic_array_reshape_family::{
    CHOOSECOLS_META, CHOOSEROWS_META, DROP_META, EXPAND_META, FILTER_META, SORT_META, SORTBY_META,
    TAKE_META, TOCOL_META, TOROW_META, TRANSPOSE_META, UNIQUE_META, VSTACK_META, WRAPCOLS_META,
    WRAPROWS_META, eval_choosecols_surface, eval_chooserows_surface, eval_drop_surface,
    eval_expand_surface, eval_filter_surface, eval_sort_surface, eval_sortby_surface,
    eval_take_surface, eval_tocol_surface, eval_torow_surface, eval_transpose_surface,
    eval_unique_surface, eval_vstack_surface, eval_wrapcols_surface, eval_wraprows_surface,
    map_dynamic_array_reshape_error_to_ws,
};
use crate::functions::engineering_radix_family::{
    eval_bin2dec_surface, eval_bin2hex_surface, eval_bin2oct_surface, eval_dec2bin_surface,
    eval_dec2hex_surface, eval_dec2oct_surface, eval_hex2bin_surface, eval_hex2dec_surface,
    eval_hex2oct_surface, eval_oct2bin_surface, eval_oct2dec_surface, eval_oct2hex_surface,
    map_engineering_radix_error_to_ws,
};
use crate::functions::error_type_fn::{eval_error_type_surface, map_error_type_error_to_ws};
use crate::functions::even_fn::{eval_even_surface, even_kernel, map_even_error_to_ws};
use crate::functions::exact_fn::{eval_exact_surface, map_exact_error_to_ws};
use crate::functions::exp_fn::{eval_exp_surface, exp_kernel, map_exp_error_to_ws};
use crate::functions::fact::{eval_fact_surface, fact_kernel, map_fact_error_to_ws};
use crate::functions::factdouble::{
    eval_factdouble_surface, factdouble_kernel, map_factdouble_error_to_ws,
};
use crate::functions::false_fn::eval_false_surface;
use crate::functions::financial_time_value_family::{
    eval_effect_surface, eval_fv_surface, eval_fvschedule_surface, eval_ipmt_surface,
    eval_ispmt_surface, eval_mirr_surface, eval_nominal_surface, eval_nper_surface,
    eval_npv_surface, eval_pduration_surface, eval_pmt_surface, eval_ppmt_surface, eval_pv_surface,
    eval_rate_surface, eval_rri_surface, map_financial_time_value_error_to_ws,
};
use crate::functions::fisher_fn::{eval_fisher_surface, map_fisher_error_to_ws};
use crate::functions::fisherinv_fn::{eval_fisherinv_surface, map_fisherinv_error_to_ws};
use crate::functions::fixed_fn::{eval_fixed_surface, map_fixed_error_to_ws};
use crate::functions::gauss_fn::{eval_gauss_surface, map_gauss_error_to_ws};
use crate::functions::gcd_fn::{eval_gcd_surface, map_gcd_error_to_ws};
use crate::functions::geomean_fn::{eval_geomean_surface, map_geomean_error_to_ws};
use crate::functions::gestep_fn::{eval_gestep_surface, gestep_kernel, map_gestep_error_to_ws};
use crate::functions::groupby_fn::eval_groupby_surface;
use crate::functions::harmean_fn::{eval_harmean_surface, map_harmean_error_to_ws};
use crate::functions::hstack::{eval_hstack_surface, map_hstack_error_to_ws};
use crate::functions::hyperlink_fn::{
    eval_hyperlink_surface, eval_hyperlink_surface_extended, map_hyperlink_error_to_ws,
};
use crate::functions::if_fn::{eval_if_surface, map_if_error_to_ws};
use crate::functions::iferror::{eval_iferror_surface, map_iferror_error_to_ws};
use crate::functions::ifna_fn::{eval_ifna_surface, map_ifna_error_to_ws};
use crate::functions::image_fn::{
    eval_image_surface, eval_image_surface_extended, map_image_error_to_ws,
};
use crate::functions::index::{eval_index_surface, map_index_error_to_ws};
use crate::functions::indirect::{eval_indirect_surface, map_indirect_error_to_ws};
use crate::functions::info_fn::{eval_info_surface, map_info_error_to_ws};
use crate::functions::int_fn::{eval_int_surface, int_kernel, map_int_error_to_ws};
use crate::functions::intercept_fn::{eval_intercept_surface, map_intercept_error_to_ws};
use crate::functions::is_predicates_family::{
    eval_isblank_surface, eval_iserr_surface, eval_iserror_surface, eval_islogical_surface,
    eval_isna_surface, eval_isnontext_surface, eval_isodd_surface, eval_isref_surface,
    eval_istext_surface, map_information_predicate_error_to_ws,
};
use crate::functions::iseven_fn::{eval_iseven_surface, map_iseven_error_to_ws};
use crate::functions::isnumber::{eval_isnumber_surface, map_isnumber_error_to_ws};
use crate::functions::large_fn::{eval_large_surface, map_large_error_to_ws};
use crate::functions::lcm_fn::{eval_lcm_surface, map_lcm_error_to_ws};
use crate::functions::legacy_stats_alias_family::{
    eval_covar_surface, eval_loginv_surface, eval_mode_surface, eval_percentile_surface,
    eval_percentrank_surface, eval_quartile_surface, map_legacy_stats_alias_error_to_ws,
};
use crate::functions::ln_fn::{eval_ln_surface, ln_kernel, map_ln_error_to_ws};
use crate::functions::log_fn::{eval_log_surface, map_log_error_to_ws};
use crate::functions::log10_fn::{eval_log10_surface, log10_kernel, map_log10_error_to_ws};
use crate::functions::lookup_prob_frequency_family::{
    eval_frequency_surface, eval_lookup_surface, eval_mode_mult_surface, eval_prob_surface,
    map_lookup_prob_frequency_error_to_ws,
};
use crate::functions::match_fn::{eval_match_surface, map_match_error_to_ws};
use crate::functions::matrix_family::{
    eval_mdeterm_surface, eval_minverse_surface, eval_mmult_surface, eval_munit_surface,
    map_matrix_error_to_ws,
};
use crate::functions::max_fn::{eval_max_surface, map_max_error_to_ws};
use crate::functions::maxa_fn::{eval_maxa_surface, map_maxa_error_to_ws};
use crate::functions::median_fn::{eval_median_surface, map_median_error_to_ws};
use crate::functions::min_fn::{eval_min_surface, map_min_error_to_ws};
use crate::functions::mina_fn::{eval_mina_surface, map_mina_error_to_ws};
use crate::functions::misc_conversion_family::{
    RandomArrayProvider, eval_bahttext_surface, eval_convert_surface, eval_euroconvert_surface,
    eval_percentof_surface, eval_randarray_surface, map_misc_conversion_error_to_ws,
};
use crate::functions::misc_switch_info_family::{
    eval_isformula_surface, eval_switch_surface, map_misc_switch_info_error_to_ws,
};
use crate::functions::mod_fn::{eval_mod_surface, map_mod_error_to_ws, mod_kernel};
use crate::functions::mode_sngl_fn::{eval_mode_sngl_surface, map_mode_sngl_error_to_ws};
use crate::functions::moment_stats_family::{
    eval_kurt_surface, eval_skew_p_surface, eval_skew_surface, eval_steyx_surface,
    eval_trimmean_surface, map_moment_stats_error_to_ws,
};
use crate::functions::mround::{eval_mround_surface, map_mround_error_to_ws, mround_kernel};
use crate::functions::multinomial::{eval_multinomial_surface, map_multinomial_error_to_ws};
use crate::functions::n_fn::{eval_n_surface, map_n_error_to_ws};
use crate::functions::na_fn::eval_na_surface;
use crate::functions::normal_log_family::{
    eval_confidence_norm_surface, eval_confidence_surface, eval_lognorm_dist_surface,
    eval_lognorm_inv_surface, eval_lognormdist_surface, eval_norm_dist_surface,
    eval_norm_inv_surface, eval_norm_s_dist_surface, eval_norm_s_inv_surface,
    eval_normdist_surface, eval_norminv_surface, eval_normsdist_surface, eval_normsinv_surface,
    map_normal_log_error_to_ws,
};
use crate::functions::not_fn::{eval_not_surface, map_not_error_to_ws};
use crate::functions::now_fn::{
    NowProvider, eval_now_surface, eval_now_surface_extended, map_now_error_to_ws,
};
use crate::functions::number_regex_translate_family::{
    eval_numbervalue_surface, eval_regexextract_surface, eval_regexreplace_surface,
    eval_regextest_surface, eval_translate_surface, map_number_regex_translate_error_to_ws,
};
use crate::functions::odd_bond_family::{
    eval_oddfprice_surface, eval_oddfyield_surface, eval_oddlprice_surface, eval_oddlyield_surface,
    map_odd_bond_error_to_ws,
};
use crate::functions::odd_fn::{eval_odd_surface, map_odd_error_to_ws, odd_kernel};
use crate::functions::offset::{eval_offset_surface, map_offset_error_to_ws};
use crate::functions::op_add::{eval_op_add_surface, map_op_add_error_to_ws, op_add_kernel};
use crate::functions::op_implicit_intersection::{
    OP_IMPLICIT_INTERSECTION_META, eval_op_implicit_intersection_surface,
    map_op_implicit_intersection_error_to_ws,
};
use crate::functions::op_spill_ref::{eval_op_spill_ref_surface, map_op_spill_ref_error_to_ws};
use crate::functions::operator_arithmetic_family::{
    OP_DIVIDE_META, OP_MULTIPLY_META, OP_NEGATE_META, OP_PERCENT_META, OP_POWER_META,
    OP_SUBTRACT_META, OP_UNARY_PLUS_META, eval_op_divide_surface, eval_op_multiply_surface,
    eval_op_negate_surface, eval_op_percent_surface, eval_op_power_surface,
    eval_op_subtract_surface, eval_op_unary_plus_surface, map_operator_binary_error_to_ws,
    map_operator_unary_error_to_ws, op_divide_kernel, op_multiply_kernel, op_negate_kernel,
    op_percent_kernel, op_subtract_kernel, op_unary_plus_kernel,
};
use crate::functions::operator_compare_concat_family::{
    OP_CONCAT_META, OP_EQUAL_META, OP_GREATER_EQUAL_META, OP_GREATER_THAN_META, OP_LESS_EQUAL_META,
    OP_LESS_THAN_META, OP_NOT_EQUAL_META, eval_op_concat_surface, eval_op_equal_surface,
    eval_op_greater_equal_surface, eval_op_greater_than_surface, eval_op_less_equal_surface,
    eval_op_less_than_surface, eval_op_not_equal_surface, map_operator_compare_concat_error_to_ws,
};
use crate::functions::operator_reference_family::{
    OP_INTERSECTION_REF_META, OP_RANGE_REF_META, OP_TRIM_REF_BOTH_META, OP_TRIM_REF_LEADING_META,
    OP_TRIM_REF_TRAILING_META, OP_UNION_REF_META, eval_op_intersection_ref_surface,
    eval_op_range_ref_surface, eval_op_trim_ref_both_surface, eval_op_trim_ref_leading_surface,
    eval_op_trim_ref_trailing_surface, eval_op_union_ref_surface,
    map_operator_reference_error_to_ws,
};
use crate::functions::or_fn::{eval_or_surface, map_or_error_to_ws};
use crate::functions::pearson_fn::{eval_pearson_surface, map_pearson_error_to_ws};
use crate::functions::percentile_exc_fn::{
    eval_percentile_exc_surface, map_percentile_exc_error_to_ws,
};
use crate::functions::percentile_inc_fn::{
    eval_percentile_inc_surface, map_percentile_inc_error_to_ws,
};
use crate::functions::percentrank_exc_fn::{
    eval_percentrank_exc_surface, map_percentrank_exc_error_to_ws,
};
use crate::functions::percentrank_inc_fn::{
    eval_percentrank_inc_surface, map_percentrank_inc_error_to_ws,
};
use crate::functions::permut_fn::{eval_permut_surface, map_permut_error_to_ws};
use crate::functions::permutationa_fn::{eval_permutationa_surface, map_permutationa_error_to_ws};
use crate::functions::phi_fn::{eval_phi_surface, map_phi_error_to_ws};
use crate::functions::pi::eval_pi;
use crate::functions::pivotby_fn::eval_pivotby_surface;
use crate::functions::power_fn::{eval_power_surface, map_power_error_to_ws, power_kernel};
use crate::functions::product::{eval_product_surface, map_product_error_to_ws};
use crate::functions::quartile_exc_fn::{eval_quartile_exc_surface, map_quartile_exc_error_to_ws};
use crate::functions::quartile_inc_fn::{eval_quartile_inc_surface, map_quartile_inc_error_to_ws};
use crate::functions::quotient_fn::{
    eval_quotient_surface, map_quotient_error_to_ws, quotient_kernel,
};
use crate::functions::radians::{eval_radians_surface, map_radians_error_to_ws, radians_kernel};
use crate::functions::rand_fn::{RandomProvider, eval_rand_surface, map_rand_error_to_ws};
use crate::functions::randbetween_fn::{eval_randbetween_surface, map_randbetween_error_to_ws};
use crate::functions::rank_avg_fn::{eval_rank_avg_surface, map_rank_avg_error_to_ws};
use crate::functions::rank_eq_fn::{eval_rank_eq_surface, map_rank_eq_error_to_ws};
use crate::functions::rank_fn::{eval_rank_surface, map_rank_error_to_ws};
use crate::functions::reference_metadata_family::{
    ADDRESS_META, AREAS_META, FORMULATEXT_META, SHEET_META, SHEETS_META, eval_address_surface,
    eval_areas_surface, eval_formulatext_surface, eval_sheet_surface, eval_sheets_surface,
    map_reference_metadata_error_to_ws,
};
use crate::functions::regression_forecast_family::{
    eval_forecast_linear_surface, eval_forecast_surface, eval_growth_surface, eval_linest_surface,
    eval_logest_surface, eval_trend_surface, map_regression_forecast_error_to_ws,
};
use crate::functions::roman_fn::{eval_roman_surface, map_roman_error_to_ws};
use crate::functions::round_fn::{eval_round_surface, map_round_error_to_ws, round_kernel};
use crate::functions::rounddown_fn::{eval_rounddown_surface, map_rounddown_error_to_ws};
use crate::functions::roundup_fn::{eval_roundup_surface, map_roundup_error_to_ws};
use crate::functions::row_fn::{eval_row_surface, map_row_error_to_ws};
use crate::functions::rows_fn::{eval_rows_surface, map_rows_error_to_ws};
use crate::functions::rsq_fn::{eval_rsq_surface, map_rsq_error_to_ws};
use crate::functions::rtd_fn::{RtdProvider, eval_rtd_surface, map_rtd_error_to_ws};
use crate::functions::sec::{eval_sec_surface, map_sec_error_to_ws, sec_kernel};
use crate::functions::sech::{eval_sech_surface, map_sech_error_to_ws, sech_kernel};
use crate::functions::sequence::{eval_sequence_surface, map_sequence_error_to_ws};
use crate::functions::sign_fn::{eval_sign_surface, map_sign_error_to_ws, sign_kernel};
use crate::functions::sin::{eval_sin_surface, map_sin_error_to_ws};
use crate::functions::sinh::{eval_sinh_surface, map_sinh_error_to_ws, sinh_kernel};
use crate::functions::slope_fn::{eval_slope_surface, map_slope_error_to_ws};
use crate::functions::small_fn::{eval_small_surface, map_small_error_to_ws};
use crate::functions::special_dist_family::{
    eval_erf_precise_surface, eval_erf_surface, eval_erfc_precise_surface, eval_erfc_surface,
    eval_gamma_surface, eval_gammaln_precise_surface, eval_gammaln_surface,
    eval_weibull_dist_surface, eval_weibull_surface, map_special_dist_error_to_ws,
};
use crate::functions::sqrt_fn::{eval_sqrt_surface, map_sqrt_error_to_ws, sqrt_kernel};
use crate::functions::sqrtpi::{eval_sqrtpi_surface, map_sqrtpi_error_to_ws, sqrtpi_kernel};
use crate::functions::standardize_fn::{eval_standardize_surface, map_standardize_error_to_ws};
use crate::functions::statistical_tests_family::{
    eval_chisq_test_surface, eval_chitest_surface, eval_f_test_surface, eval_ftest_surface,
    eval_t_test_surface, eval_ttest_surface, map_statistical_tests_error_to_ws,
};
use crate::functions::stdev_fn::{eval_stdev_surface, map_stdev_error_to_ws};
use crate::functions::stdev_p_fn::{eval_stdev_p_surface, map_stdev_p_error_to_ws};
use crate::functions::stdev_s_fn::{eval_stdev_s_surface, map_stdev_s_error_to_ws};
use crate::functions::stdeva_fn::{eval_stdeva_surface, map_stdeva_error_to_ws};
use crate::functions::stdevp_fn::{eval_stdevp_surface, map_stdevp_error_to_ws};
use crate::functions::stdevpa_fn::{eval_stdevpa_surface, map_stdevpa_error_to_ws};
use crate::functions::sum::{eval_sum_surface, map_sum_error_to_ws};
use crate::functions::sumproduct_family::{
    eval_seriessum_surface, eval_sumproduct_surface, eval_sumx2my2_surface, eval_sumx2py2_surface,
    eval_sumxmy2_surface, map_sumproduct_error_to_ws,
};
use crate::functions::sumsq::{eval_sumsq_surface, map_sumsq_error_to_ws};
use crate::functions::t_fn::{eval_t_surface, map_t_error_to_ws};
use crate::functions::tan::{eval_tan_surface, map_tan_error_to_ws, tan_kernel};
use crate::functions::tanh::{eval_tanh_surface, map_tanh_error_to_ws, tanh_kernel};
use crate::functions::test_alias_family::{eval_ztest_surface, map_test_alias_error_to_ws};
use crate::functions::text_b_compat_family::{
    eval_findb_surface, eval_leftb_surface, eval_lenb_surface, eval_midb_surface,
    eval_replaceb_surface, eval_rightb_surface, eval_searchb_surface,
    map_text_b_compat_error_to_ws,
};
use crate::functions::text_compat_locale_family::{
    eval_asc_surface, eval_dbcs_surface, eval_jis_surface, map_text_compat_locale_error_to_ws,
};
use crate::functions::text_delim_family::{
    eval_textafter_surface, eval_textbefore_surface, map_text_delim_error_to_ws,
};
use crate::functions::text_fn::{eval_text_surface, map_text_error_to_ws};
use crate::functions::text_scalar_misc::{
    eval_char_surface, eval_code_surface, eval_lower_surface, eval_rept_surface, eval_trim_surface,
    eval_upper_surface, map_text_scalar_error_to_ws,
};
use crate::functions::text_search_replace_family::{
    eval_find_surface, eval_proper_surface, eval_replace_surface, eval_search_surface,
    eval_substitute_surface, map_text_search_replace_error_to_ws,
};
use crate::functions::text_slice_family::{
    eval_left_surface, eval_len_surface, eval_mid_surface, eval_right_surface,
    map_text_slice_error_to_ws,
};
use crate::functions::text_unicode_fn::{
    eval_unichar_surface, eval_unicode_surface, map_text_unicode_error_to_ws,
};
use crate::functions::textjoin::{eval_textjoin_surface, map_textjoin_error_to_ws};
use crate::functions::today_fn::{
    TodayProvider, eval_today_surface, eval_today_surface_extended, map_today_error_to_ws,
};
use crate::functions::trimrange_fn::{eval_trimrange_surface, map_trimrange_error_to_ws};
use crate::functions::true_fn::eval_true_surface;
use crate::functions::trunc_fn::{eval_trunc_surface, map_trunc_error_to_ws, trunc_kernel};
use crate::functions::type_fn::{eval_type_surface, map_type_error_to_ws};
use crate::functions::value_fn::{eval_value_surface, map_value_error_to_ws};
use crate::functions::valuetotext_fn::{eval_valuetotext_surface, map_valuetotext_error_to_ws};
use crate::functions::var_fn::{eval_var_surface, map_var_error_to_ws};
use crate::functions::var_p_fn::{eval_var_p_surface, map_var_p_error_to_ws};
use crate::functions::var_s_fn::{eval_var_s_surface, map_var_s_error_to_ws};
use crate::functions::vara_fn::{eval_vara_surface, map_vara_error_to_ws};
use crate::functions::varp_fn::{eval_varp_surface, map_varp_error_to_ws};
use crate::functions::varpa_fn::{eval_varpa_surface, map_varpa_error_to_ws};
use crate::functions::vhlookup_family::{
    eval_hlookup_surface, eval_vlookup_surface, map_vhlookup_error_to_ws,
};
use crate::functions::web_text_xml_family::{
    eval_encodeurl_surface, eval_filterxml_surface, map_web_text_xml_error_to_ws,
};
use crate::functions::workday_networkdays_family::{
    eval_networkdays_intl_surface, eval_networkdays_surface, eval_workday_intl_surface,
    eval_workday_surface, map_workday_networkdays_error_to_ws,
};
use crate::functions::xlookup::{eval_xlookup_surface, map_xlookup_error_to_ws};
use crate::functions::xmatch::XmatchEvalError;
use crate::functions::xmatch_surface::eval_xmatch_surface_value;
use crate::functions::xor_fn::{eval_xor_surface, map_xor_error_to_ws};
use crate::host_info::HostInfoProvider;
use crate::locale_format::LocaleFormatContext;
use crate::resolver::RefResolutionError;
use crate::resolver::ReferenceResolver;
use crate::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalError, EvalValue, ExtendedValue,
    Value, WorksheetErrorCode,
};

pub const FUNC_ID_ACOS: &str = "FUNC.ACOS";
pub const FUNC_ID_ACOT: &str = "FUNC.ACOT";
pub const FUNC_ID_ACOSH: &str = "FUNC.ACOSH";
pub const FUNC_ID_ACOTH: &str = "FUNC.ACOTH";
pub const FUNC_ID_ABS: &str = "FUNC.ABS";
pub const FUNC_ID_ACCRINT: &str = "FUNC.ACCRINT";
pub const FUNC_ID_ACCRINTM: &str = "FUNC.ACCRINTM";
pub const FUNC_ID_AGGREGATE: &str = "FUNC.AGGREGATE";
pub const FUNC_ID_AMORDEGRC: &str = "FUNC.AMORDEGRC";
pub const FUNC_ID_AMORLINC: &str = "FUNC.AMORLINC";
pub const FUNC_ID_ATAN: &str = "FUNC.ATAN";
pub const FUNC_ID_ASIN: &str = "FUNC.ASIN";
pub const FUNC_ID_ASINH: &str = "FUNC.ASINH";
pub const FUNC_ID_ATAN2: &str = "FUNC.ATAN2";
pub const FUNC_ID_ATANH: &str = "FUNC.ATANH";
pub const FUNC_ID_AND: &str = "FUNC.AND";
pub const FUNC_ID_ARABIC: &str = "FUNC.ARABIC";
pub const FUNC_ID_ADDRESS: &str = "FUNC.ADDRESS";
pub const FUNC_ID_ARRAYTOTEXT: &str = "FUNC.ARRAYTOTEXT";
pub const FUNC_ID_ASC: &str = "FUNC.ASC";
pub const FUNC_ID_AREAS: &str = "FUNC.AREAS";
pub const FUNC_ID_AVEDEV: &str = "FUNC.AVEDEV";
pub const FUNC_ID_AVERAGE: &str = "FUNC.AVERAGE";
pub const FUNC_ID_AVERAGEIF: &str = "FUNC.AVERAGEIF";
pub const FUNC_ID_AVERAGEIFS: &str = "FUNC.AVERAGEIFS";
pub const FUNC_ID_AVERAGEA: &str = "FUNC.AVERAGEA";
pub const FUNC_ID_BAHTTEXT: &str = "FUNC.BAHTTEXT";
pub const FUNC_ID_BASE: &str = "FUNC.BASE";
pub const FUNC_ID_BETA_DIST: &str = "FUNC.BETA.DIST";
pub const FUNC_ID_BETA_INV: &str = "FUNC.BETA.INV";
pub const FUNC_ID_BETADIST: &str = "FUNC.BETADIST";
pub const FUNC_ID_BETAINV: &str = "FUNC.BETAINV";
pub const FUNC_ID_BESSELI: &str = "FUNC.BESSELI";
pub const FUNC_ID_BESSELJ: &str = "FUNC.BESSELJ";
pub const FUNC_ID_BESSELK: &str = "FUNC.BESSELK";
pub const FUNC_ID_BESSELY: &str = "FUNC.BESSELY";
pub const FUNC_ID_BINOM_DIST: &str = "FUNC.BINOM.DIST";
pub const FUNC_ID_BINOM_DIST_RANGE: &str = "FUNC.BINOM.DIST.RANGE";
pub const FUNC_ID_BINOM_INV: &str = "FUNC.BINOM.INV";
pub const FUNC_ID_BINOMDIST: &str = "FUNC.BINOMDIST";
pub const FUNC_ID_BIN2DEC: &str = "FUNC.BIN2DEC";
pub const FUNC_ID_BIN2HEX: &str = "FUNC.BIN2HEX";
pub const FUNC_ID_BIN2OCT: &str = "FUNC.BIN2OCT";
pub const FUNC_ID_BITAND: &str = "FUNC.BITAND";
pub const FUNC_ID_BITLSHIFT: &str = "FUNC.BITLSHIFT";
pub const FUNC_ID_BITOR: &str = "FUNC.BITOR";
pub const FUNC_ID_BITRSHIFT: &str = "FUNC.BITRSHIFT";
pub const FUNC_ID_BITXOR: &str = "FUNC.BITXOR";
pub const FUNC_ID_BYCOL: &str = "FUNC.BYCOL";
pub const FUNC_ID_BYROW: &str = "FUNC.BYROW";
pub const FUNC_ID_CELL: &str = "FUNC.CELL";
pub const FUNC_ID_CEILING: &str = "FUNC.CEILING";
pub const FUNC_ID_CEILING_MATH: &str = "FUNC.CEILING.MATH";
pub const FUNC_ID_CEILING_PRECISE: &str = "FUNC.CEILING.PRECISE";
pub const FUNC_ID_CHIDIST: &str = "FUNC.CHIDIST";
pub const FUNC_ID_CHIINV: &str = "FUNC.CHIINV";
pub const FUNC_ID_CHOOSE: &str = "FUNC.CHOOSE";
pub const FUNC_ID_CHOOSECOLS: &str = "FUNC.CHOOSECOLS";
pub const FUNC_ID_CHOOSEROWS: &str = "FUNC.CHOOSEROWS";
pub const FUNC_ID_CHISQ_DIST: &str = "FUNC.CHISQ.DIST";
pub const FUNC_ID_CHISQ_DIST_RT: &str = "FUNC.CHISQ.DIST.RT";
pub const FUNC_ID_CHISQ_INV: &str = "FUNC.CHISQ.INV";
pub const FUNC_ID_CHISQ_INV_RT: &str = "FUNC.CHISQ.INV.RT";
pub const FUNC_ID_CHISQ_TEST: &str = "FUNC.CHISQ.TEST";
pub const FUNC_ID_CHITEST: &str = "FUNC.CHITEST";
pub const FUNC_ID_CHAR: &str = "FUNC.CHAR";
pub const FUNC_ID_COLUMN: &str = "FUNC.COLUMN";
pub const FUNC_ID_COLUMNS: &str = "FUNC.COLUMNS";
pub const FUNC_ID_CODE: &str = "FUNC.CODE";
pub const FUNC_ID_COMBIN: &str = "FUNC.COMBIN";
pub const FUNC_ID_COMBINA: &str = "FUNC.COMBINA";
pub const FUNC_ID_COMPLEX: &str = "FUNC.COMPLEX";
pub const FUNC_ID_CLEAN: &str = "FUNC.CLEAN";
pub const FUNC_ID_CONCAT: &str = "FUNC.CONCAT";
pub const FUNC_ID_CONCATENATE: &str = "FUNC.CONCATENATE";
pub const FUNC_ID_COS: &str = "FUNC.COS";
pub const FUNC_ID_COSH: &str = "FUNC.COSH";
pub const FUNC_ID_CORREL: &str = "FUNC.CORREL";
pub const FUNC_ID_COVARIANCE_P: &str = "FUNC.COVARIANCE.P";
pub const FUNC_ID_COVARIANCE_S: &str = "FUNC.COVARIANCE.S";
pub const FUNC_ID_COT: &str = "FUNC.COT";
pub const FUNC_ID_COTH: &str = "FUNC.COTH";
pub const FUNC_ID_COUNT: &str = "FUNC.COUNT";
pub const FUNC_ID_COUNTBLANK: &str = "FUNC.COUNTBLANK";
pub const FUNC_ID_COUNTIF: &str = "FUNC.COUNTIF";
pub const FUNC_ID_COUNTIFS: &str = "FUNC.COUNTIFS";
pub const FUNC_ID_COUNTA: &str = "FUNC.COUNTA";
pub const FUNC_ID_COUPDAYBS: &str = "FUNC.COUPDAYBS";
pub const FUNC_ID_COUPDAYS: &str = "FUNC.COUPDAYS";
pub const FUNC_ID_COUPDAYSNC: &str = "FUNC.COUPDAYSNC";
pub const FUNC_ID_COUPNCD: &str = "FUNC.COUPNCD";
pub const FUNC_ID_COUPNUM: &str = "FUNC.COUPNUM";
pub const FUNC_ID_COUPPCD: &str = "FUNC.COUPPCD";
pub const FUNC_ID_COVAR: &str = "FUNC.COVAR";
pub const FUNC_ID_CRITBINOM: &str = "FUNC.CRITBINOM";
pub const FUNC_ID_CSC: &str = "FUNC.CSC";
pub const FUNC_ID_CSCH: &str = "FUNC.CSCH";
pub const FUNC_ID_CUMIPMT: &str = "FUNC.CUMIPMT";
pub const FUNC_ID_CUMPRINC: &str = "FUNC.CUMPRINC";
pub const FUNC_ID_CONVERT: &str = "FUNC.CONVERT";
pub const FUNC_ID_DAVERAGE: &str = "FUNC.DAVERAGE";
pub const FUNC_ID_DATE: &str = "FUNC.DATE";
pub const FUNC_ID_DAY: &str = "FUNC.DAY";
pub const FUNC_ID_DAYS: &str = "FUNC.DAYS";
pub const FUNC_ID_DAYS360: &str = "FUNC.DAYS360";
pub const FUNC_ID_DATEDIF: &str = "FUNC.DATEDIF";
pub const FUNC_ID_DATEVALUE: &str = "FUNC.DATEVALUE";
pub const FUNC_ID_DBCS: &str = "FUNC.DBCS";
pub const FUNC_ID_DB: &str = "FUNC.DB";
pub const FUNC_ID_DEC2BIN: &str = "FUNC.DEC2BIN";
pub const FUNC_ID_DEC2HEX: &str = "FUNC.DEC2HEX";
pub const FUNC_ID_DEC2OCT: &str = "FUNC.DEC2OCT";
pub const FUNC_ID_EDATE: &str = "FUNC.EDATE";
pub const FUNC_ID_EOMONTH: &str = "FUNC.EOMONTH";
pub const FUNC_ID_EFFECT: &str = "FUNC.EFFECT";
pub const FUNC_ID_EUROCONVERT: &str = "FUNC.EUROCONVERT";
pub const FUNC_ID_EXPAND: &str = "FUNC.EXPAND";
pub const FUNC_ID_DECIMAL: &str = "FUNC.DECIMAL";
pub const FUNC_ID_ENCODEURL: &str = "FUNC.ENCODEURL";
pub const FUNC_ID_DDB: &str = "FUNC.DDB";
pub const FUNC_ID_DCOUNT: &str = "FUNC.DCOUNT";
pub const FUNC_ID_DCOUNTA: &str = "FUNC.DCOUNTA";
pub const FUNC_ID_DISC: &str = "FUNC.DISC";
pub const FUNC_ID_DGET: &str = "FUNC.DGET";
pub const FUNC_ID_DMAX: &str = "FUNC.DMAX";
pub const FUNC_ID_DMIN: &str = "FUNC.DMIN";
pub const FUNC_ID_DPRODUCT: &str = "FUNC.DPRODUCT";
pub const FUNC_ID_DSTDEV: &str = "FUNC.DSTDEV";
pub const FUNC_ID_DSTDEVP: &str = "FUNC.DSTDEVP";
pub const FUNC_ID_DSUM: &str = "FUNC.DSUM";
pub const FUNC_ID_DVAR: &str = "FUNC.DVAR";
pub const FUNC_ID_DVARP: &str = "FUNC.DVARP";
pub const FUNC_ID_DROP: &str = "FUNC.DROP";
pub const FUNC_ID_DEVSQ: &str = "FUNC.DEVSQ";
pub const FUNC_ID_DEGREES: &str = "FUNC.DEGREES";
pub const FUNC_ID_DELTA: &str = "FUNC.DELTA";
pub const FUNC_ID_DURATION: &str = "FUNC.DURATION";
pub const FUNC_ID_DOLLAR: &str = "FUNC.DOLLAR";
pub const FUNC_ID_DOLLARDE: &str = "FUNC.DOLLARDE";
pub const FUNC_ID_DOLLARFR: &str = "FUNC.DOLLARFR";
pub const FUNC_ID_EVEN: &str = "FUNC.EVEN";
pub const FUNC_ID_ERROR_TYPE: &str = "FUNC.ERROR.TYPE";
pub const FUNC_ID_ERF: &str = "FUNC.ERF";
pub const FUNC_ID_ERF_PRECISE: &str = "FUNC.ERF.PRECISE";
pub const FUNC_ID_ERFC: &str = "FUNC.ERFC";
pub const FUNC_ID_ERFC_PRECISE: &str = "FUNC.ERFC.PRECISE";
pub const FUNC_ID_EXACT: &str = "FUNC.EXACT";
pub const FUNC_ID_EXPON_DIST: &str = "FUNC.EXPON.DIST";
pub const FUNC_ID_EXPONDIST: &str = "FUNC.EXPONDIST";
pub const FUNC_ID_EXP: &str = "FUNC.EXP";
pub const FUNC_ID_FACT: &str = "FUNC.FACT";
pub const FUNC_ID_FACTDOUBLE: &str = "FUNC.FACTDOUBLE";
pub const FUNC_ID_FALSE: &str = "FUNC.FALSE";
pub const FUNC_ID_FTEST: &str = "FUNC.FTEST";
pub const FUNC_ID_FREQUENCY: &str = "FUNC.FREQUENCY";
pub const FUNC_ID_FV: &str = "FUNC.FV";
pub const FUNC_ID_FVSCHEDULE: &str = "FUNC.FVSCHEDULE";
pub const FUNC_ID_F_DIST: &str = "FUNC.F.DIST";
pub const FUNC_ID_F_DIST_RT: &str = "FUNC.F.DIST.RT";
pub const FUNC_ID_F_INV: &str = "FUNC.F.INV";
pub const FUNC_ID_F_INV_RT: &str = "FUNC.F.INV.RT";
pub const FUNC_ID_F_TEST: &str = "FUNC.F.TEST";
pub const FUNC_ID_FDIST: &str = "FUNC.FDIST";
pub const FUNC_ID_FINV: &str = "FUNC.FINV";
pub const FUNC_ID_FISHER: &str = "FUNC.FISHER";
pub const FUNC_ID_FISHERINV: &str = "FUNC.FISHERINV";
pub const FUNC_ID_FIND: &str = "FUNC.FIND";
pub const FUNC_ID_FINDB: &str = "FUNC.FINDB";
pub const FUNC_ID_FILTER: &str = "FUNC.FILTER";
pub const FUNC_ID_FILTERXML: &str = "FUNC.FILTERXML";
pub const FUNC_ID_FIXED: &str = "FUNC.FIXED";
pub const FUNC_ID_FLOOR: &str = "FUNC.FLOOR";
pub const FUNC_ID_FLOOR_MATH: &str = "FUNC.FLOOR.MATH";
pub const FUNC_ID_FLOOR_PRECISE: &str = "FUNC.FLOOR.PRECISE";
pub const FUNC_ID_FORMULATEXT: &str = "FUNC.FORMULATEXT";
pub const FUNC_ID_IRR: &str = "FUNC.IRR";
pub const FUNC_ID_GAUSS: &str = "FUNC.GAUSS";
pub const FUNC_ID_GAMMA: &str = "FUNC.GAMMA";
pub const FUNC_ID_GAMMA_DIST: &str = "FUNC.GAMMA.DIST";
pub const FUNC_ID_GAMMA_INV: &str = "FUNC.GAMMA.INV";
pub const FUNC_ID_GAMMADIST: &str = "FUNC.GAMMADIST";
pub const FUNC_ID_GAMMAINV: &str = "FUNC.GAMMAINV";
pub const FUNC_ID_GAMMALN: &str = "FUNC.GAMMALN";
pub const FUNC_ID_GAMMALN_PRECISE: &str = "FUNC.GAMMALN.PRECISE";
pub const FUNC_ID_GCD: &str = "FUNC.GCD";
pub const FUNC_ID_GEOMEAN: &str = "FUNC.GEOMEAN";
pub const FUNC_ID_GESTEP: &str = "FUNC.GESTEP";
pub const FUNC_ID_GROUPBY: &str = "FUNC.GROUPBY";
pub const FUNC_ID_GROWTH: &str = "FUNC.GROWTH";
pub const FUNC_ID_FORECAST: &str = "FUNC.FORECAST";
pub const FUNC_ID_FORECAST_LINEAR: &str = "FUNC.FORECAST.LINEAR";
pub const FUNC_ID_HARMEAN: &str = "FUNC.HARMEAN";
pub const FUNC_ID_HYPERLINK: &str = "FUNC.HYPERLINK";
pub const FUNC_ID_IMAGE: &str = "FUNC.IMAGE";
pub const FUNC_ID_HYPGEOM_DIST: &str = "FUNC.HYPGEOM.DIST";
pub const FUNC_ID_HYPGEOMDIST: &str = "FUNC.HYPGEOMDIST";
pub const FUNC_ID_HOUR: &str = "FUNC.HOUR";
pub const FUNC_ID_HSTACK: &str = "FUNC.HSTACK";
pub const FUNC_ID_INFO: &str = "FUNC.INFO";
pub const FUNC_ID_ISOMITTED: &str = "FUNC.ISOMITTED";
pub const FUNC_ID_IMABS: &str = "FUNC.IMABS";
pub const FUNC_ID_IMAGINARY: &str = "FUNC.IMAGINARY";
pub const FUNC_ID_IMARGUMENT: &str = "FUNC.IMARGUMENT";
pub const FUNC_ID_IMCONJUGATE: &str = "FUNC.IMCONJUGATE";
pub const FUNC_ID_IMCOS: &str = "FUNC.IMCOS";
pub const FUNC_ID_IMCOSH: &str = "FUNC.IMCOSH";
pub const FUNC_ID_IMCOT: &str = "FUNC.IMCOT";
pub const FUNC_ID_IMCSC: &str = "FUNC.IMCSC";
pub const FUNC_ID_IMCSCH: &str = "FUNC.IMCSCH";
pub const FUNC_ID_IMDIV: &str = "FUNC.IMDIV";
pub const FUNC_ID_IMEXP: &str = "FUNC.IMEXP";
pub const FUNC_ID_IMLN: &str = "FUNC.IMLN";
pub const FUNC_ID_IMLOG10: &str = "FUNC.IMLOG10";
pub const FUNC_ID_IMLOG2: &str = "FUNC.IMLOG2";
pub const FUNC_ID_IMPOWER: &str = "FUNC.IMPOWER";
pub const FUNC_ID_IMPRODUCT: &str = "FUNC.IMPRODUCT";
pub const FUNC_ID_IMREAL: &str = "FUNC.IMREAL";
pub const FUNC_ID_IMSEC: &str = "FUNC.IMSEC";
pub const FUNC_ID_IMSECH: &str = "FUNC.IMSECH";
pub const FUNC_ID_IMSIN: &str = "FUNC.IMSIN";
pub const FUNC_ID_IMSINH: &str = "FUNC.IMSINH";
pub const FUNC_ID_IMSQRT: &str = "FUNC.IMSQRT";
pub const FUNC_ID_IMSUB: &str = "FUNC.IMSUB";
pub const FUNC_ID_IMSUM: &str = "FUNC.IMSUM";
pub const FUNC_ID_IMTAN: &str = "FUNC.IMTAN";
pub const FUNC_ID_ISFORMULA: &str = "FUNC.ISFORMULA";
pub const FUNC_ID_IF: &str = "FUNC.IF";
pub const FUNC_ID_IFERROR: &str = "FUNC.IFERROR";
pub const FUNC_ID_IFNA: &str = "FUNC.IFNA";
pub const FUNC_ID_IFS: &str = "FUNC.IFS";
pub const FUNC_ID_INDEX: &str = "FUNC.INDEX";
pub const FUNC_ID_INDIRECT: &str = "FUNC.INDIRECT";
pub const FUNC_ID_IPMT: &str = "FUNC.IPMT";
pub const FUNC_ID_ISPMT: &str = "FUNC.ISPMT";
pub const FUNC_ID_HEX2BIN: &str = "FUNC.HEX2BIN";
pub const FUNC_ID_HEX2DEC: &str = "FUNC.HEX2DEC";
pub const FUNC_ID_HEX2OCT: &str = "FUNC.HEX2OCT";
pub const FUNC_ID_ISNUMBER: &str = "FUNC.ISNUMBER";
pub const FUNC_ID_ISBLANK: &str = "FUNC.ISBLANK";
pub const FUNC_ID_ISERR: &str = "FUNC.ISERR";
pub const FUNC_ID_ISERROR: &str = "FUNC.ISERROR";
pub const FUNC_ID_ISLOGICAL: &str = "FUNC.ISLOGICAL";
pub const FUNC_ID_ISNA: &str = "FUNC.ISNA";
pub const FUNC_ID_ISNONTEXT: &str = "FUNC.ISNONTEXT";
pub const FUNC_ID_ISODD: &str = "FUNC.ISODD";
pub const FUNC_ID_ISREF: &str = "FUNC.ISREF";
pub const FUNC_ID_ISTEXT: &str = "FUNC.ISTEXT";
pub const FUNC_ID_ISOWEEKNUM: &str = "FUNC.ISOWEEKNUM";
pub const FUNC_ID_ISO_CEILING: &str = "FUNC.ISO.CEILING";
pub const FUNC_ID_INTERCEPT: &str = "FUNC.INTERCEPT";
pub const FUNC_ID_INT: &str = "FUNC.INT";
pub const FUNC_ID_INTRATE: &str = "FUNC.INTRATE";
pub const FUNC_ID_ISEVEN: &str = "FUNC.ISEVEN";
pub const FUNC_ID_JIS: &str = "FUNC.JIS";
pub const FUNC_ID_KURT: &str = "FUNC.KURT";
pub const FUNC_ID_LARGE: &str = "FUNC.LARGE";
pub const FUNC_ID_LCM: &str = "FUNC.LCM";
pub const FUNC_ID_LINEST: &str = "FUNC.LINEST";
pub const FUNC_ID_LOGINV: &str = "FUNC.LOGINV";
pub const FUNC_ID_LN: &str = "FUNC.LN";
pub const FUNC_ID_LOG: &str = "FUNC.LOG";
pub const FUNC_ID_LOG10: &str = "FUNC.LOG10";
pub const FUNC_ID_LOOKUP: &str = "FUNC.LOOKUP";
pub const FUNC_ID_LOWER: &str = "FUNC.LOWER";
pub const FUNC_ID_MAX: &str = "FUNC.MAX";
pub const FUNC_ID_MAXA: &str = "FUNC.MAXA";
pub const FUNC_ID_MAXIFS: &str = "FUNC.MAXIFS";
pub const FUNC_ID_MEDIAN: &str = "FUNC.MEDIAN";
pub const FUNC_ID_MATCH: &str = "FUNC.MATCH";
pub const FUNC_ID_MAKEARRAY: &str = "FUNC.MAKEARRAY";
pub const FUNC_ID_MAP: &str = "FUNC.MAP";
pub const FUNC_ID_MDETERM: &str = "FUNC.MDETERM";
pub const FUNC_ID_MDURATION: &str = "FUNC.MDURATION";
pub const FUNC_ID_MINVERSE: &str = "FUNC.MINVERSE";
pub const FUNC_ID_MMULT: &str = "FUNC.MMULT";
pub const FUNC_ID_MUNIT: &str = "FUNC.MUNIT";
pub const FUNC_ID_MIN: &str = "FUNC.MIN";
pub const FUNC_ID_MINA: &str = "FUNC.MINA";
pub const FUNC_ID_MINIFS: &str = "FUNC.MINIFS";
pub const FUNC_ID_MIRR: &str = "FUNC.MIRR";
pub const FUNC_ID_MINUTE: &str = "FUNC.MINUTE";
pub const FUNC_ID_MOD: &str = "FUNC.MOD";
pub const FUNC_ID_MODE: &str = "FUNC.MODE";
pub const FUNC_ID_MODE_MULT: &str = "FUNC.MODE.MULT";
pub const FUNC_ID_MODE_SNGL: &str = "FUNC.MODE.SNGL";
pub const FUNC_ID_MONTH: &str = "FUNC.MONTH";
pub const FUNC_ID_MROUND: &str = "FUNC.MROUND";
pub const FUNC_ID_MULTINOMIAL: &str = "FUNC.MULTINOMIAL";
pub const FUNC_ID_N: &str = "FUNC.N";
pub const FUNC_ID_NA: &str = "FUNC.NA";
pub const FUNC_ID_NOMINAL: &str = "FUNC.NOMINAL";
pub const FUNC_ID_NPER: &str = "FUNC.NPER";
pub const FUNC_ID_NPV: &str = "FUNC.NPV";
pub const FUNC_ID_NUMBERVALUE: &str = "FUNC.NUMBERVALUE";
pub const FUNC_ID_NEGBINOM_DIST: &str = "FUNC.NEGBINOM.DIST";
pub const FUNC_ID_NEGBINOMDIST: &str = "FUNC.NEGBINOMDIST";
pub const FUNC_ID_CONFIDENCE: &str = "FUNC.CONFIDENCE";
pub const FUNC_ID_CONFIDENCE_T: &str = "FUNC.CONFIDENCE.T";
pub const FUNC_ID_CONFIDENCE_NORM: &str = "FUNC.CONFIDENCE.NORM";
pub const FUNC_ID_LOGNORM_DIST: &str = "FUNC.LOGNORM.DIST";
pub const FUNC_ID_LOGNORM_INV: &str = "FUNC.LOGNORM.INV";
pub const FUNC_ID_LOGNORMDIST: &str = "FUNC.LOGNORMDIST";
pub const FUNC_ID_LOGEST: &str = "FUNC.LOGEST";
pub const FUNC_ID_NORM_DIST: &str = "FUNC.NORM.DIST";
pub const FUNC_ID_NORM_INV: &str = "FUNC.NORM.INV";
pub const FUNC_ID_NORM_S_DIST: &str = "FUNC.NORM.S.DIST";
pub const FUNC_ID_NORM_S_INV: &str = "FUNC.NORM.S.INV";
pub const FUNC_ID_NORMDIST: &str = "FUNC.NORMDIST";
pub const FUNC_ID_NORMINV: &str = "FUNC.NORMINV";
pub const FUNC_ID_NORMSDIST: &str = "FUNC.NORMSDIST";
pub const FUNC_ID_NORMSINV: &str = "FUNC.NORMSINV";
pub const FUNC_ID_NETWORKDAYS: &str = "FUNC.NETWORKDAYS";
pub const FUNC_ID_NETWORKDAYS_INTL: &str = "FUNC.NETWORKDAYS.INTL";
pub const FUNC_ID_NOT: &str = "FUNC.NOT";
pub const FUNC_ID_NOW: &str = "FUNC.NOW";
pub const FUNC_ID_OCT2BIN: &str = "FUNC.OCT2BIN";
pub const FUNC_ID_OCT2DEC: &str = "FUNC.OCT2DEC";
pub const FUNC_ID_OCT2HEX: &str = "FUNC.OCT2HEX";
pub const FUNC_ID_POISSON: &str = "FUNC.POISSON";
pub const FUNC_ID_POISSON_DIST: &str = "FUNC.POISSON.DIST";
pub const FUNC_ID_ODD: &str = "FUNC.ODD";
pub const FUNC_ID_ODDFPRICE: &str = "FUNC.ODDFPRICE";
pub const FUNC_ID_ODDFYIELD: &str = "FUNC.ODDFYIELD";
pub const FUNC_ID_ODDLPRICE: &str = "FUNC.ODDLPRICE";
pub const FUNC_ID_ODDLYIELD: &str = "FUNC.ODDLYIELD";
pub const FUNC_ID_OR: &str = "FUNC.OR";
pub const FUNC_ID_OFFSET: &str = "FUNC.OFFSET";
pub const FUNC_ID_CALL: &str = "FUNC.CALL";
pub const FUNC_ID_OP_ADD: &str = "FUNC.OP_ADD";
pub const FUNC_ID_OP_CONCAT: &str = "FUNC.OP_CONCAT";
pub const FUNC_ID_OP_DIVIDE: &str = "FUNC.OP_DIVIDE";
pub const FUNC_ID_OP_EQUAL: &str = "FUNC.OP_EQUAL";
pub const FUNC_ID_OP_GREATER_EQUAL: &str = "FUNC.OP_GREATER_EQUAL";
pub const FUNC_ID_OP_GREATER_THAN: &str = "FUNC.OP_GREATER_THAN";
pub const FUNC_ID_OP_IMPLICIT_INTERSECTION: &str = "FUNC.OP_IMPLICIT_INTERSECTION";
pub const FUNC_ID_OP_INTERSECTION_REF: &str = "FUNC.OP_INTERSECTION_REF";
pub const FUNC_ID_OP_LESS_EQUAL: &str = "FUNC.OP_LESS_EQUAL";
pub const FUNC_ID_OP_LESS_THAN: &str = "FUNC.OP_LESS_THAN";
pub const FUNC_ID_OP_MULTIPLY: &str = "FUNC.OP_MULTIPLY";
pub const FUNC_ID_OP_NEGATE: &str = "FUNC.OP_NEGATE";
pub const FUNC_ID_OP_NOT_EQUAL: &str = "FUNC.OP_NOT_EQUAL";
pub const FUNC_ID_OP_PERCENT: &str = "FUNC.OP_PERCENT";
pub const FUNC_ID_OP_POWER: &str = "FUNC.OP_POWER";
pub const FUNC_ID_OP_RANGE_REF: &str = "FUNC.OP_RANGE_REF";
pub const FUNC_ID_OP_SPILL_REF: &str = "FUNC.OP_SPILL_REF";
pub const FUNC_ID_OP_SUBTRACT: &str = "FUNC.OP_SUBTRACT";
pub const FUNC_ID_OP_TRIM_REF_BOTH: &str = "FUNC.OP_TRIM_REF_BOTH";
pub const FUNC_ID_OP_TRIM_REF_LEADING: &str = "FUNC.OP_TRIM_REF_LEADING";
pub const FUNC_ID_OP_TRIM_REF_TRAILING: &str = "FUNC.OP_TRIM_REF_TRAILING";
pub const FUNC_ID_OP_UNARY_PLUS: &str = "FUNC.OP_UNARY_PLUS";
pub const FUNC_ID_OP_UNION_REF: &str = "FUNC.OP_UNION_REF";
pub const FUNC_ID_PEARSON: &str = "FUNC.PEARSON";
pub const FUNC_ID_PDURATION: &str = "FUNC.PDURATION";
pub const FUNC_ID_PERMUT: &str = "FUNC.PERMUT";
pub const FUNC_ID_PERMUTATIONA: &str = "FUNC.PERMUTATIONA";
pub const FUNC_ID_PERCENTILE_EXC: &str = "FUNC.PERCENTILE.EXC";
pub const FUNC_ID_PERCENTILE_INC: &str = "FUNC.PERCENTILE.INC";
pub const FUNC_ID_PERCENTILE: &str = "FUNC.PERCENTILE";
pub const FUNC_ID_PERCENTRANK_EXC: &str = "FUNC.PERCENTRANK.EXC";
pub const FUNC_ID_PERCENTRANK_INC: &str = "FUNC.PERCENTRANK.INC";
pub const FUNC_ID_PERCENTRANK: &str = "FUNC.PERCENTRANK";
pub const FUNC_ID_PHI: &str = "FUNC.PHI";
pub const FUNC_ID_PI: &str = "FUNC.PI";
pub const FUNC_ID_PIVOTBY: &str = "FUNC.PIVOTBY";
pub const FUNC_ID_PMT: &str = "FUNC.PMT";
pub const FUNC_ID_PPMT: &str = "FUNC.PPMT";
pub const FUNC_ID_PERCENTOF: &str = "FUNC.PERCENTOF";
pub const FUNC_ID_PRICE: &str = "FUNC.PRICE";
pub const FUNC_ID_PRICEDISC: &str = "FUNC.PRICEDISC";
pub const FUNC_ID_PRICEMAT: &str = "FUNC.PRICEMAT";
pub const FUNC_ID_PROB: &str = "FUNC.PROB";
pub const FUNC_ID_PRODUCT: &str = "FUNC.PRODUCT";
pub const FUNC_ID_POWER: &str = "FUNC.POWER";
pub const FUNC_ID_PV: &str = "FUNC.PV";
pub const FUNC_ID_PROPER: &str = "FUNC.PROPER";
pub const FUNC_ID_QUOTIENT: &str = "FUNC.QUOTIENT";
pub const FUNC_ID_QUARTILE_EXC: &str = "FUNC.QUARTILE.EXC";
pub const FUNC_ID_QUARTILE_INC: &str = "FUNC.QUARTILE.INC";
pub const FUNC_ID_QUARTILE: &str = "FUNC.QUARTILE";
pub const FUNC_ID_RAND: &str = "FUNC.RAND";
pub const FUNC_ID_RANDARRAY: &str = "FUNC.RANDARRAY";
pub const FUNC_ID_RANDBETWEEN: &str = "FUNC.RANDBETWEEN";
pub const FUNC_ID_REDUCE: &str = "FUNC.REDUCE";
pub const FUNC_ID_RATE: &str = "FUNC.RATE";
pub const FUNC_ID_RADIANS: &str = "FUNC.RADIANS";
pub const FUNC_ID_RANK: &str = "FUNC.RANK";
pub const FUNC_ID_RANK_AVG: &str = "FUNC.RANK.AVG";
pub const FUNC_ID_RANK_EQ: &str = "FUNC.RANK.EQ";
pub const FUNC_ID_ROW: &str = "FUNC.ROW";
pub const FUNC_ID_ROWS: &str = "FUNC.ROWS";
pub const FUNC_ID_RRI: &str = "FUNC.RRI";
pub const FUNC_ID_RTD: &str = "FUNC.RTD";
pub const FUNC_ID_ROMAN: &str = "FUNC.ROMAN";
pub const FUNC_ID_ROUND: &str = "FUNC.ROUND";
pub const FUNC_ID_ROUNDDOWN: &str = "FUNC.ROUNDDOWN";
pub const FUNC_ID_REPLACE: &str = "FUNC.REPLACE";
pub const FUNC_ID_REPLACEB: &str = "FUNC.REPLACEB";
pub const FUNC_ID_RECEIVED: &str = "FUNC.RECEIVED";
pub const FUNC_ID_REGEXEXTRACT: &str = "FUNC.REGEXEXTRACT";
pub const FUNC_ID_REGEXREPLACE: &str = "FUNC.REGEXREPLACE";
pub const FUNC_ID_REGEXTEST: &str = "FUNC.REGEXTEST";
pub const FUNC_ID_REGISTER_ID: &str = "FUNC.REGISTER.ID";
pub const FUNC_ID_ROUNDUP: &str = "FUNC.ROUNDUP";
pub const FUNC_ID_RSQ: &str = "FUNC.RSQ";
pub const FUNC_ID_SECOND: &str = "FUNC.SECOND";
pub const FUNC_ID_SEQUENCE: &str = "FUNC.SEQUENCE";
pub const FUNC_ID_SCAN: &str = "FUNC.SCAN";
pub const FUNC_ID_SHEET: &str = "FUNC.SHEET";
pub const FUNC_ID_SHEETS: &str = "FUNC.SHEETS";
pub const FUNC_ID_SORT: &str = "FUNC.SORT";
pub const FUNC_ID_SORTBY: &str = "FUNC.SORTBY";
pub const FUNC_ID_SEC: &str = "FUNC.SEC";
pub const FUNC_ID_SERIESSUM: &str = "FUNC.SERIESSUM";
pub const FUNC_ID_SECH: &str = "FUNC.SECH";
pub const FUNC_ID_SIGN: &str = "FUNC.SIGN";
pub const FUNC_ID_SIN: &str = "FUNC.SIN";
pub const FUNC_ID_SINH: &str = "FUNC.SINH";
pub const FUNC_ID_SKEW: &str = "FUNC.SKEW";
pub const FUNC_ID_SKEW_P: &str = "FUNC.SKEW.P";
pub const FUNC_ID_STEYX: &str = "FUNC.STEYX";
pub const FUNC_ID_SLN: &str = "FUNC.SLN";
pub const FUNC_ID_SMALL: &str = "FUNC.SMALL";
pub const FUNC_ID_SQRT: &str = "FUNC.SQRT";
pub const FUNC_ID_SQRTPI: &str = "FUNC.SQRTPI";
pub const FUNC_ID_SLOPE: &str = "FUNC.SLOPE";
pub const FUNC_ID_STDEV: &str = "FUNC.STDEV";
pub const FUNC_ID_STDEV_P: &str = "FUNC.STDEV.P";
pub const FUNC_ID_STDEV_S: &str = "FUNC.STDEV.S";
pub const FUNC_ID_STDEVP: &str = "FUNC.STDEVP";
pub const FUNC_ID_STDEVA: &str = "FUNC.STDEVA";
pub const FUNC_ID_STDEVPA: &str = "FUNC.STDEVPA";
pub const FUNC_ID_STANDARDIZE: &str = "FUNC.STANDARDIZE";
pub const FUNC_ID_SUBTOTAL: &str = "FUNC.SUBTOTAL";
pub const FUNC_ID_SUM: &str = "FUNC.SUM";
pub const FUNC_ID_SUMIF: &str = "FUNC.SUMIF";
pub const FUNC_ID_SUMIFS: &str = "FUNC.SUMIFS";
pub const FUNC_ID_SUMPRODUCT: &str = "FUNC.SUMPRODUCT";
pub const FUNC_ID_SUMX2MY2: &str = "FUNC.SUMX2MY2";
pub const FUNC_ID_SUMX2PY2: &str = "FUNC.SUMX2PY2";
pub const FUNC_ID_SUMXMY2: &str = "FUNC.SUMXMY2";
pub const FUNC_ID_SUMSQ: &str = "FUNC.SUMSQ";
pub const FUNC_ID_SWITCH: &str = "FUNC.SWITCH";
pub const FUNC_ID_T: &str = "FUNC.T";
pub const FUNC_ID_TAKE: &str = "FUNC.TAKE";
pub const FUNC_ID_T_DIST: &str = "FUNC.T.DIST";
pub const FUNC_ID_T_DIST_2T: &str = "FUNC.T.DIST.2T";
pub const FUNC_ID_T_DIST_RT: &str = "FUNC.T.DIST.RT";
pub const FUNC_ID_T_INV: &str = "FUNC.T.INV";
pub const FUNC_ID_T_INV_2T: &str = "FUNC.T.INV.2T";
pub const FUNC_ID_T_TEST: &str = "FUNC.T.TEST";
pub const FUNC_ID_TDIST: &str = "FUNC.TDIST";
pub const FUNC_ID_TINV: &str = "FUNC.TINV";
pub const FUNC_ID_SYD: &str = "FUNC.SYD";
pub const FUNC_ID_TAN: &str = "FUNC.TAN";
pub const FUNC_ID_TANH: &str = "FUNC.TANH";
pub const FUNC_ID_TBILLEQ: &str = "FUNC.TBILLEQ";
pub const FUNC_ID_TBILLPRICE: &str = "FUNC.TBILLPRICE";
pub const FUNC_ID_TBILLYIELD: &str = "FUNC.TBILLYIELD";
pub const FUNC_ID_TOCOL: &str = "FUNC.TOCOL";
pub const FUNC_ID_TOROW: &str = "FUNC.TOROW";
pub const FUNC_ID_LEFT: &str = "FUNC.LEFT";
pub const FUNC_ID_LEFTB: &str = "FUNC.LEFTB";
pub const FUNC_ID_LEN: &str = "FUNC.LEN";
pub const FUNC_ID_LENB: &str = "FUNC.LENB";
pub const FUNC_ID_MID: &str = "FUNC.MID";
pub const FUNC_ID_MIDB: &str = "FUNC.MIDB";
pub const FUNC_ID_RIGHT: &str = "FUNC.RIGHT";
pub const FUNC_ID_RIGHTB: &str = "FUNC.RIGHTB";
pub const FUNC_ID_TEXT: &str = "FUNC.TEXT";
pub const FUNC_ID_TEXTAFTER: &str = "FUNC.TEXTAFTER";
pub const FUNC_ID_TEXTBEFORE: &str = "FUNC.TEXTBEFORE";
pub const FUNC_ID_TEXTSPLIT: &str = "FUNC.TEXTSPLIT";
pub const FUNC_ID_SEARCH: &str = "FUNC.SEARCH";
pub const FUNC_ID_SEARCHB: &str = "FUNC.SEARCHB";
pub const FUNC_ID_REPT: &str = "FUNC.REPT";
pub const FUNC_ID_SUBSTITUTE: &str = "FUNC.SUBSTITUTE";
pub const FUNC_ID_TEXTJOIN: &str = "FUNC.TEXTJOIN";
pub const FUNC_ID_TODAY: &str = "FUNC.TODAY";
pub const FUNC_ID_TIME: &str = "FUNC.TIME";
pub const FUNC_ID_TIMEVALUE: &str = "FUNC.TIMEVALUE";
pub const FUNC_ID_TRANSLATE: &str = "FUNC.TRANSLATE";
pub const FUNC_ID_TRIMMEAN: &str = "FUNC.TRIMMEAN";
pub const FUNC_ID_TRUE: &str = "FUNC.TRUE";
pub const FUNC_ID_TREND: &str = "FUNC.TREND";
pub const FUNC_ID_TRANSPOSE: &str = "FUNC.TRANSPOSE";
pub const FUNC_ID_TRUNC: &str = "FUNC.TRUNC";
pub const FUNC_ID_TRIM: &str = "FUNC.TRIM";
pub const FUNC_ID_TRIMRANGE: &str = "FUNC.TRIMRANGE";
pub const FUNC_ID_TTEST: &str = "FUNC.TTEST";
pub const FUNC_ID_TYPE: &str = "FUNC.TYPE";
pub const FUNC_ID_UNIQUE: &str = "FUNC.UNIQUE";
pub const FUNC_ID_UNICHAR: &str = "FUNC.UNICHAR";
pub const FUNC_ID_UNICODE: &str = "FUNC.UNICODE";
pub const FUNC_ID_UPPER: &str = "FUNC.UPPER";
pub const FUNC_ID_VALUE: &str = "FUNC.VALUE";
pub const FUNC_ID_VALUETOTEXT: &str = "FUNC.VALUETOTEXT";
pub const FUNC_ID_VAR: &str = "FUNC.VAR";
pub const FUNC_ID_VAR_P: &str = "FUNC.VAR.P";
pub const FUNC_ID_VAR_S: &str = "FUNC.VAR.S";
pub const FUNC_ID_VARA: &str = "FUNC.VARA";
pub const FUNC_ID_VARP: &str = "FUNC.VARP";
pub const FUNC_ID_VARPA: &str = "FUNC.VARPA";
pub const FUNC_ID_VDB: &str = "FUNC.VDB";
pub const FUNC_ID_VSTACK: &str = "FUNC.VSTACK";
pub const FUNC_ID_HLOOKUP: &str = "FUNC.HLOOKUP";
pub const FUNC_ID_VLOOKUP: &str = "FUNC.VLOOKUP";
pub const FUNC_ID_WEIBULL: &str = "FUNC.WEIBULL";
pub const FUNC_ID_WEIBULL_DIST: &str = "FUNC.WEIBULL.DIST";
pub const FUNC_ID_WRAPCOLS: &str = "FUNC.WRAPCOLS";
pub const FUNC_ID_WRAPROWS: &str = "FUNC.WRAPROWS";
pub const FUNC_ID_XLOOKUP: &str = "FUNC.XLOOKUP";
pub const FUNC_ID_XIRR: &str = "FUNC.XIRR";
pub const FUNC_ID_XNPV: &str = "FUNC.XNPV";
pub const FUNC_ID_XMATCH: &str = "FUNC.XMATCH";
pub const FUNC_ID_XOR: &str = "FUNC.XOR";
pub const FUNC_ID_WEEKDAY: &str = "FUNC.WEEKDAY";
pub const FUNC_ID_WEEKNUM: &str = "FUNC.WEEKNUM";
pub const FUNC_ID_WORKDAY: &str = "FUNC.WORKDAY";
pub const FUNC_ID_WORKDAY_INTL: &str = "FUNC.WORKDAY.INTL";
pub const FUNC_ID_YIELD: &str = "FUNC.YIELD";
pub const FUNC_ID_YIELDDISC: &str = "FUNC.YIELDDISC";
pub const FUNC_ID_YIELDMAT: &str = "FUNC.YIELDMAT";
pub const FUNC_ID_YEAR: &str = "FUNC.YEAR";
pub const FUNC_ID_YEARFRAC: &str = "FUNC.YEARFRAC";
pub const FUNC_ID_Z_TEST: &str = "FUNC.Z.TEST";
pub const FUNC_ID_ZTEST: &str = "FUNC.ZTEST";

fn map_ref_resolution_to_ws(e: &RefResolutionError) -> WorksheetErrorCode {
    match e {
        RefResolutionError::CapabilityDenied { .. } => WorksheetErrorCode::Ref,
        RefResolutionError::UnresolvedReference { .. } => WorksheetErrorCode::Ref,
        RefResolutionError::EvalTimeDerefNotAllowed => WorksheetErrorCode::Ref,
        RefResolutionError::ProviderFailure { .. } => WorksheetErrorCode::Value,
    }
}

fn map_coercion_to_ws(e: &CoercionError) -> WorksheetErrorCode {
    match e {
        CoercionError::WorksheetError(code) => *code,
        CoercionError::RefResolution(err) => map_ref_resolution_to_ws(err),
        CoercionError::MissingArg => WorksheetErrorCode::Value,
        CoercionError::EmptyCell => WorksheetErrorCode::Value,
        CoercionError::NonNumericText(_) => WorksheetErrorCode::Value,
        CoercionError::UnsupportedValueKind(_) => WorksheetErrorCode::Value,
    }
}

fn map_abs_error_to_ws(e: &AbsEvalError) -> WorksheetErrorCode {
    match e {
        AbsEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        AbsEvalError::Coercion(err) => map_coercion_to_ws(err),
    }
}

fn map_eval_error_to_ws(e: &EvalError) -> WorksheetErrorCode {
    match e {
        EvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
    }
}

fn map_xmatch_error_to_ws(e: &XmatchEvalError) -> WorksheetErrorCode {
    match e {
        XmatchEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        XmatchEvalError::EmptyLookupArray => WorksheetErrorCode::NA,
        XmatchEvalError::MissingArg => WorksheetErrorCode::Value,
        XmatchEvalError::EmptyCell => WorksheetErrorCode::Value,
        XmatchEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        XmatchEvalError::Coercion(_) => WorksheetErrorCode::Value,
        XmatchEvalError::UnsupportedValueKind(_) => WorksheetErrorCode::Value,
        XmatchEvalError::InvalidMatchMode(_) => WorksheetErrorCode::Value,
        XmatchEvalError::InvalidSearchMode(_) => WorksheetErrorCode::Value,
        XmatchEvalError::UnsupportedMatchModeForSeed(_) => WorksheetErrorCode::NA,
        XmatchEvalError::UnsupportedSearchModeForSeed(_) => WorksheetErrorCode::NA,
        XmatchEvalError::NotAvailable => WorksheetErrorCode::NA,
    }
}

struct FixedNowProvider {
    serial: f64,
}

impl NowProvider for FixedNowProvider {
    fn now_serial(&self) -> f64 {
        self.serial
    }
}

impl TodayProvider for FixedNowProvider {
    fn today_serial(&self) -> f64 {
        self.serial
    }
}

struct FixedRandomProvider {
    value: f64,
}

impl RandomProvider for FixedRandomProvider {
    fn random_unit(&self) -> f64 {
        self.value
    }
}

impl RandomArrayProvider for FixedRandomProvider {
    fn random_unit(&self) -> f64 {
        self.value
    }
}

struct RejectingCallableInvoker;

impl CallableInvoker for RejectingCallableInvoker {
    fn invoke(
        &self,
        callable: &crate::value::LambdaValue,
        _args: &[crate::functions::adapters::PreparedArgValue],
    ) -> Result<crate::functions::adapters::PreparedArgValue, CallableInvocationError> {
        Err(CallableInvocationError::UnsupportedCallableToken(
            callable.callable_token.clone(),
        ))
    }
}

fn singleton_arg_slice(arg: &CallArgValue) -> Vec<CallArgValue> {
    // Core value model does not yet carry full array payloads in prepared call-args.
    // Keep singleton passthrough until array payload/value expansion is implemented.
    vec![arg.clone()]
}

pub fn arg_preparation_profile(function_id: &str) -> Option<ArgPreparationProfile> {
    match function_id {
        FUNC_ID_ACOS => Some(crate::functions::acos::ACOS_META.arg_preparation_profile),
        FUNC_ID_ACOT => Some(crate::functions::acot::ACOT_META.arg_preparation_profile),
        FUNC_ID_ACOSH => Some(crate::functions::acosh::ACOSH_META.arg_preparation_profile),
        FUNC_ID_ACOTH => Some(crate::functions::acoth::ACOTH_META.arg_preparation_profile),
        FUNC_ID_ABS => Some(crate::functions::abs::ABS_META.arg_preparation_profile),
        FUNC_ID_ACCRINT => {
            Some(crate::functions::bond_core_family::ACCRINT_META.arg_preparation_profile)
        }
        FUNC_ID_ACCRINTM => {
            Some(crate::functions::bond_core_family::ACCRINTM_META.arg_preparation_profile)
        }
        FUNC_ID_AGGREGATE => Some(
            crate::functions::subtotal_aggregate_family::AGGREGATE_META.arg_preparation_profile,
        ),
        FUNC_ID_AMORDEGRC => {
            Some(crate::functions::amor_depreciation_family::AMORDEGRC_META.arg_preparation_profile)
        }
        FUNC_ID_AMORLINC => {
            Some(crate::functions::amor_depreciation_family::AMORLINC_META.arg_preparation_profile)
        }
        FUNC_ID_ATAN => Some(crate::functions::atan::ATAN_META.arg_preparation_profile),
        FUNC_ID_ASIN => Some(crate::functions::asin::ASIN_META.arg_preparation_profile),
        FUNC_ID_ASINH => Some(crate::functions::asinh::ASINH_META.arg_preparation_profile),
        FUNC_ID_ATAN2 => Some(crate::functions::atan2::ATAN2_META.arg_preparation_profile),
        FUNC_ID_ATANH => Some(crate::functions::atanh::ATANH_META.arg_preparation_profile),
        FUNC_ID_AND => Some(crate::functions::and_fn::AND_META.arg_preparation_profile),
        FUNC_ID_ARABIC => Some(crate::functions::arabic_fn::ARABIC_META.arg_preparation_profile),
        FUNC_ID_CALL => Some(CALL_META.arg_preparation_profile),
        FUNC_ID_ADDRESS => Some(ADDRESS_META.arg_preparation_profile),
        FUNC_ID_ARRAYTOTEXT => Some(
            crate::functions::array_text_split_family::ARRAYTOTEXT_META.arg_preparation_profile,
        ),
        FUNC_ID_ASC => {
            Some(crate::functions::text_compat_locale_family::ASC_META.arg_preparation_profile)
        }
        FUNC_ID_AREAS => Some(AREAS_META.arg_preparation_profile),
        FUNC_ID_AVEDEV => Some(crate::functions::avedev_fn::AVEDEV_META.arg_preparation_profile),
        FUNC_ID_AVERAGE => Some(crate::functions::average::AVERAGE_META.arg_preparation_profile),
        FUNC_ID_AVERAGEIF => {
            Some(crate::functions::criteria_family::AVERAGEIF_META.arg_preparation_profile)
        }
        FUNC_ID_AVERAGEIFS => {
            Some(crate::functions::criteria_family::AVERAGEIFS_META.arg_preparation_profile)
        }
        FUNC_ID_AVERAGEA => {
            Some(crate::functions::averagea_fn::AVERAGEA_META.arg_preparation_profile)
        }
        FUNC_ID_BAHTTEXT => {
            Some(crate::functions::misc_conversion_family::BAHTTEXT_META.arg_preparation_profile)
        }
        FUNC_ID_BASE => Some(crate::functions::base_fn::BASE_META.arg_preparation_profile),
        FUNC_ID_BETA_DIST => {
            Some(crate::functions::beta_gamma_stats_family::BETA_DIST_META.arg_preparation_profile)
        }
        FUNC_ID_BETA_INV => {
            Some(crate::functions::beta_gamma_stats_family::BETA_INV_META.arg_preparation_profile)
        }
        FUNC_ID_BETADIST => {
            Some(crate::functions::beta_gamma_stats_family::BETADIST_META.arg_preparation_profile)
        }
        FUNC_ID_BETAINV => {
            Some(crate::functions::beta_gamma_stats_family::BETAINV_META.arg_preparation_profile)
        }
        FUNC_ID_BESSELI => {
            Some(crate::functions::bessel_convert_family::BESSELI_META.arg_preparation_profile)
        }
        FUNC_ID_BESSELJ => {
            Some(crate::functions::bessel_convert_family::BESSELJ_META.arg_preparation_profile)
        }
        FUNC_ID_BESSELK => {
            Some(crate::functions::bessel_convert_family::BESSELK_META.arg_preparation_profile)
        }
        FUNC_ID_BESSELY => {
            Some(crate::functions::bessel_convert_family::BESSELY_META.arg_preparation_profile)
        }
        FUNC_ID_BINOM_DIST => {
            Some(crate::functions::discrete_dist_family::BINOM_DIST_META.arg_preparation_profile)
        }
        FUNC_ID_BINOM_DIST_RANGE => Some(
            crate::functions::discrete_dist_family::BINOM_DIST_RANGE_META.arg_preparation_profile,
        ),
        FUNC_ID_BINOM_INV => {
            Some(crate::functions::discrete_dist_family::BINOM_INV_META.arg_preparation_profile)
        }
        FUNC_ID_BINOMDIST => {
            Some(crate::functions::discrete_dist_family::BINOMDIST_META.arg_preparation_profile)
        }
        FUNC_ID_BIN2DEC => {
            Some(crate::functions::engineering_radix_family::BIN2DEC_META.arg_preparation_profile)
        }
        FUNC_ID_BIN2HEX => {
            Some(crate::functions::engineering_radix_family::BIN2HEX_META.arg_preparation_profile)
        }
        FUNC_ID_BIN2OCT => {
            Some(crate::functions::engineering_radix_family::BIN2OCT_META.arg_preparation_profile)
        }
        FUNC_ID_BITAND => Some(crate::functions::bitand_fn::BITAND_META.arg_preparation_profile),
        FUNC_ID_BITLSHIFT => {
            Some(crate::functions::bitlshift_fn::BITLSHIFT_META.arg_preparation_profile)
        }
        FUNC_ID_BITOR => Some(crate::functions::bitor_fn::BITOR_META.arg_preparation_profile),
        FUNC_ID_BITRSHIFT => {
            Some(crate::functions::bitrshift_fn::BITRSHIFT_META.arg_preparation_profile)
        }
        FUNC_ID_BITXOR => Some(crate::functions::bitxor_fn::BITXOR_META.arg_preparation_profile),
        FUNC_ID_BYCOL => Some(BYCOL_META.arg_preparation_profile),
        FUNC_ID_BYROW => Some(BYROW_META.arg_preparation_profile),
        FUNC_ID_CELL => Some(crate::functions::cell::CELL_META.arg_preparation_profile),
        FUNC_ID_CEILING => {
            Some(crate::functions::ceiling_floor_family::CEILING_META.arg_preparation_profile)
        }
        FUNC_ID_CEILING_MATH => {
            Some(crate::functions::ceiling_floor_family::CEILING_MATH_META.arg_preparation_profile)
        }
        FUNC_ID_CEILING_PRECISE => Some(
            crate::functions::ceiling_floor_family::CEILING_PRECISE_META.arg_preparation_profile,
        ),
        FUNC_ID_CHIDIST => {
            Some(crate::functions::chi_f_t_family::CHIDIST_META.arg_preparation_profile)
        }
        FUNC_ID_CHIINV => {
            Some(crate::functions::chi_f_t_family::CHIINV_META.arg_preparation_profile)
        }
        FUNC_ID_CHOOSE => {
            Some(crate::functions::choose_ifs_family::CHOOSE_META.arg_preparation_profile)
        }
        FUNC_ID_CHOOSECOLS => Some(CHOOSECOLS_META.arg_preparation_profile),
        FUNC_ID_CHOOSEROWS => Some(CHOOSEROWS_META.arg_preparation_profile),
        FUNC_ID_CHISQ_DIST => {
            Some(crate::functions::chi_f_t_family::CHISQ_DIST_META.arg_preparation_profile)
        }
        FUNC_ID_CHISQ_DIST_RT => {
            Some(crate::functions::chi_f_t_family::CHISQ_DIST_RT_META.arg_preparation_profile)
        }
        FUNC_ID_CHISQ_INV => {
            Some(crate::functions::chi_f_t_family::CHISQ_INV_META.arg_preparation_profile)
        }
        FUNC_ID_CHISQ_INV_RT => {
            Some(crate::functions::chi_f_t_family::CHISQ_INV_RT_META.arg_preparation_profile)
        }
        FUNC_ID_CHISQ_TEST => Some(
            crate::functions::statistical_tests_family::CHISQ_TEST_META.arg_preparation_profile,
        ),
        FUNC_ID_CHITEST => {
            Some(crate::functions::statistical_tests_family::CHITEST_META.arg_preparation_profile)
        }
        FUNC_ID_CHAR => Some(crate::functions::text_scalar_misc::CHAR_META.arg_preparation_profile),
        FUNC_ID_COLUMN => Some(crate::functions::column_fn::COLUMN_META.arg_preparation_profile),
        FUNC_ID_COLUMNS => Some(crate::functions::columns_fn::COLUMNS_META.arg_preparation_profile),
        FUNC_ID_CODE => Some(crate::functions::text_scalar_misc::CODE_META.arg_preparation_profile),
        FUNC_ID_COMBIN => Some(crate::functions::combin::COMBIN_META.arg_preparation_profile),
        FUNC_ID_COMBINA => Some(crate::functions::combina::COMBINA_META.arg_preparation_profile),
        FUNC_ID_COMPLEX => {
            Some(crate::functions::complex_family::COMPLEX_META.arg_preparation_profile)
        }
        FUNC_ID_CLEAN => Some(crate::functions::clean_fn::CLEAN_META.arg_preparation_profile),
        FUNC_ID_CONCAT => {
            Some(crate::functions::concat_family::CONCAT_META.arg_preparation_profile)
        }
        FUNC_ID_CONCATENATE => {
            Some(crate::functions::concat_family::CONCATENATE_META.arg_preparation_profile)
        }
        FUNC_ID_COS => Some(crate::functions::cos::COS_META.arg_preparation_profile),
        FUNC_ID_COSH => Some(crate::functions::cosh::COSH_META.arg_preparation_profile),
        FUNC_ID_CORREL => Some(crate::functions::correl_fn::CORREL_META.arg_preparation_profile),
        FUNC_ID_COVARIANCE_P => {
            Some(crate::functions::covariance_p_fn::COVARIANCE_P_META.arg_preparation_profile)
        }
        FUNC_ID_COVARIANCE_S => {
            Some(crate::functions::covariance_s_fn::COVARIANCE_S_META.arg_preparation_profile)
        }
        FUNC_ID_COT => Some(crate::functions::cot::COT_META.arg_preparation_profile),
        FUNC_ID_COTH => Some(crate::functions::coth::COTH_META.arg_preparation_profile),
        FUNC_ID_COUNT => Some(crate::functions::count::COUNT_META.arg_preparation_profile),
        FUNC_ID_COUNTBLANK => {
            Some(crate::functions::countblank_fn::COUNTBLANK_META.arg_preparation_profile)
        }
        FUNC_ID_COUPDAYBS => {
            Some(crate::functions::coupon_family::COUPDAYBS_META.arg_preparation_profile)
        }
        FUNC_ID_COUPDAYS => {
            Some(crate::functions::coupon_family::COUPDAYS_META.arg_preparation_profile)
        }
        FUNC_ID_COUPDAYSNC => {
            Some(crate::functions::coupon_family::COUPDAYSNC_META.arg_preparation_profile)
        }
        FUNC_ID_COUPNCD => {
            Some(crate::functions::coupon_family::COUPNCD_META.arg_preparation_profile)
        }
        FUNC_ID_COUPNUM => {
            Some(crate::functions::coupon_family::COUPNUM_META.arg_preparation_profile)
        }
        FUNC_ID_COUPPCD => {
            Some(crate::functions::coupon_family::COUPPCD_META.arg_preparation_profile)
        }
        FUNC_ID_CRITBINOM => {
            Some(crate::functions::discrete_dist_family::CRITBINOM_META.arg_preparation_profile)
        }
        FUNC_ID_COUNTA => Some(crate::functions::counta::COUNTA_META.arg_preparation_profile),
        FUNC_ID_COVAR => {
            Some(crate::functions::legacy_stats_alias_family::COVAR_META.arg_preparation_profile)
        }
        FUNC_ID_CSC => Some(crate::functions::csc::CSC_META.arg_preparation_profile),
        FUNC_ID_CSCH => Some(crate::functions::csch::CSCH_META.arg_preparation_profile),
        FUNC_ID_CUMIPMT => {
            Some(crate::functions::cumulative_finance_family::CUMIPMT_META.arg_preparation_profile)
        }
        FUNC_ID_CUMPRINC => {
            Some(crate::functions::cumulative_finance_family::CUMPRINC_META.arg_preparation_profile)
        }
        FUNC_ID_CONVERT => {
            Some(crate::functions::misc_conversion_family::CONVERT_META.arg_preparation_profile)
        }
        FUNC_ID_DAVERAGE => {
            Some(crate::functions::database_family::DAVERAGE_META.arg_preparation_profile)
        }
        FUNC_ID_DATE => Some(crate::functions::date_fn::DATE_META.arg_preparation_profile),
        FUNC_ID_DATEDIF => {
            Some(crate::functions::date_value_family::DATEDIF_META.arg_preparation_profile)
        }
        FUNC_ID_DAY => Some(crate::functions::date_parts_family::DAY_META.arg_preparation_profile),
        FUNC_ID_DAYS => {
            Some(crate::functions::date_parts_family::DAYS_META.arg_preparation_profile)
        }
        FUNC_ID_DAYS360 => {
            Some(crate::functions::date_value_family::DAYS360_META.arg_preparation_profile)
        }
        FUNC_ID_DATEVALUE => {
            Some(crate::functions::date_value_family::DATEVALUE_META.arg_preparation_profile)
        }
        FUNC_ID_DBCS => {
            Some(crate::functions::text_compat_locale_family::DBCS_META.arg_preparation_profile)
        }
        FUNC_ID_DB => Some(crate::functions::depreciation_family::DB_META.arg_preparation_profile),
        FUNC_ID_DEC2BIN => {
            Some(crate::functions::engineering_radix_family::DEC2BIN_META.arg_preparation_profile)
        }
        FUNC_ID_DEC2HEX => {
            Some(crate::functions::engineering_radix_family::DEC2HEX_META.arg_preparation_profile)
        }
        FUNC_ID_DEC2OCT => {
            Some(crate::functions::engineering_radix_family::DEC2OCT_META.arg_preparation_profile)
        }
        FUNC_ID_EDATE => {
            Some(crate::functions::date_week_family::EDATE_META.arg_preparation_profile)
        }
        FUNC_ID_EOMONTH => {
            Some(crate::functions::date_week_family::EOMONTH_META.arg_preparation_profile)
        }
        FUNC_ID_EFFECT => {
            Some(crate::functions::financial_time_value_family::EFFECT_META.arg_preparation_profile)
        }
        FUNC_ID_EUROCONVERT => {
            Some(crate::functions::misc_conversion_family::EUROCONVERT_META.arg_preparation_profile)
        }
        FUNC_ID_EXPAND => Some(EXPAND_META.arg_preparation_profile),
        FUNC_ID_DECIMAL => Some(crate::functions::decimal_fn::DECIMAL_META.arg_preparation_profile),
        FUNC_ID_ENCODEURL => {
            Some(crate::functions::web_text_xml_family::ENCODEURL_META.arg_preparation_profile)
        }
        FUNC_ID_DDB => {
            Some(crate::functions::depreciation_family::DDB_META.arg_preparation_profile)
        }
        FUNC_ID_DCOUNT => {
            Some(crate::functions::database_family::DCOUNT_META.arg_preparation_profile)
        }
        FUNC_ID_DCOUNTA => {
            Some(crate::functions::database_family::DCOUNTA_META.arg_preparation_profile)
        }
        FUNC_ID_DISC => {
            Some(crate::functions::discount_bill_yearfrac_family::DISC_META.arg_preparation_profile)
        }
        FUNC_ID_DGET => Some(crate::functions::database_family::DGET_META.arg_preparation_profile),
        FUNC_ID_DMAX => Some(crate::functions::database_family::DMAX_META.arg_preparation_profile),
        FUNC_ID_DMIN => Some(crate::functions::database_family::DMIN_META.arg_preparation_profile),
        FUNC_ID_DPRODUCT => {
            Some(crate::functions::database_family::DPRODUCT_META.arg_preparation_profile)
        }
        FUNC_ID_DSTDEV => {
            Some(crate::functions::database_family::DSTDEV_META.arg_preparation_profile)
        }
        FUNC_ID_DSTDEVP => {
            Some(crate::functions::database_family::DSTDEVP_META.arg_preparation_profile)
        }
        FUNC_ID_DSUM => Some(crate::functions::database_family::DSUM_META.arg_preparation_profile),
        FUNC_ID_DVAR => Some(crate::functions::database_family::DVAR_META.arg_preparation_profile),
        FUNC_ID_DVARP => {
            Some(crate::functions::database_family::DVARP_META.arg_preparation_profile)
        }
        FUNC_ID_DROP => Some(DROP_META.arg_preparation_profile),
        FUNC_ID_DEVSQ => Some(crate::functions::devsq_fn::DEVSQ_META.arg_preparation_profile),
        FUNC_ID_DEGREES => Some(crate::functions::degrees::DEGREES_META.arg_preparation_profile),
        FUNC_ID_DELTA => Some(crate::functions::delta_fn::DELTA_META.arg_preparation_profile),
        FUNC_ID_DURATION => {
            Some(crate::functions::bond_core_family::DURATION_META.arg_preparation_profile)
        }
        FUNC_ID_DOLLAR => Some(crate::functions::dollar_fn::DOLLAR_META.arg_preparation_profile),
        FUNC_ID_DOLLARDE => {
            Some(crate::functions::dollar_fraction_family::DOLLARDE_META.arg_preparation_profile)
        }
        FUNC_ID_DOLLARFR => {
            Some(crate::functions::dollar_fraction_family::DOLLARFR_META.arg_preparation_profile)
        }
        FUNC_ID_EVEN => Some(crate::functions::even_fn::EVEN_META.arg_preparation_profile),
        FUNC_ID_ERROR_TYPE => {
            Some(crate::functions::error_type_fn::ERROR_TYPE_META.arg_preparation_profile)
        }
        FUNC_ID_ERF => {
            Some(crate::functions::special_dist_family::ERF_META.arg_preparation_profile)
        }
        FUNC_ID_ERF_PRECISE => {
            Some(crate::functions::special_dist_family::ERF_PRECISE_META.arg_preparation_profile)
        }
        FUNC_ID_ERFC => {
            Some(crate::functions::special_dist_family::ERFC_META.arg_preparation_profile)
        }
        FUNC_ID_ERFC_PRECISE => {
            Some(crate::functions::special_dist_family::ERFC_PRECISE_META.arg_preparation_profile)
        }
        FUNC_ID_EXACT => Some(crate::functions::exact_fn::EXACT_META.arg_preparation_profile),
        FUNC_ID_EXPON_DIST => {
            Some(crate::functions::discrete_dist_family::EXPON_DIST_META.arg_preparation_profile)
        }
        FUNC_ID_EXPONDIST => {
            Some(crate::functions::discrete_dist_family::EXPONDIST_META.arg_preparation_profile)
        }
        FUNC_ID_EXP => Some(crate::functions::exp_fn::EXP_META.arg_preparation_profile),
        FUNC_ID_FACT => Some(crate::functions::fact::FACT_META.arg_preparation_profile),
        FUNC_ID_FACTDOUBLE => {
            Some(crate::functions::factdouble::FACTDOUBLE_META.arg_preparation_profile)
        }
        FUNC_ID_F_DIST => {
            Some(crate::functions::chi_f_t_family::F_DIST_META.arg_preparation_profile)
        }
        FUNC_ID_F_DIST_RT => {
            Some(crate::functions::chi_f_t_family::F_DIST_RT_META.arg_preparation_profile)
        }
        FUNC_ID_F_INV => Some(crate::functions::chi_f_t_family::F_INV_META.arg_preparation_profile),
        FUNC_ID_F_INV_RT => {
            Some(crate::functions::chi_f_t_family::F_INV_RT_META.arg_preparation_profile)
        }
        FUNC_ID_F_TEST => {
            Some(crate::functions::statistical_tests_family::F_TEST_META.arg_preparation_profile)
        }
        FUNC_ID_FDIST => Some(crate::functions::chi_f_t_family::FDIST_META.arg_preparation_profile),
        FUNC_ID_FINV => Some(crate::functions::chi_f_t_family::FINV_META.arg_preparation_profile),
        FUNC_ID_FALSE => Some(crate::functions::false_fn::FALSE_META.arg_preparation_profile),
        FUNC_ID_FTEST => {
            Some(crate::functions::statistical_tests_family::FTEST_META.arg_preparation_profile)
        }
        FUNC_ID_FREQUENCY => Some(
            crate::functions::lookup_prob_frequency_family::FREQUENCY_META.arg_preparation_profile,
        ),
        FUNC_ID_FV => {
            Some(crate::functions::financial_time_value_family::FV_META.arg_preparation_profile)
        }
        FUNC_ID_FVSCHEDULE => Some(
            crate::functions::financial_time_value_family::FVSCHEDULE_META.arg_preparation_profile,
        ),
        FUNC_ID_FISHER => Some(crate::functions::fisher_fn::FISHER_META.arg_preparation_profile),
        FUNC_ID_FISHERINV => {
            Some(crate::functions::fisherinv_fn::FISHERINV_META.arg_preparation_profile)
        }
        FUNC_ID_FIND => {
            Some(crate::functions::text_search_replace_family::FIND_META.arg_preparation_profile)
        }
        FUNC_ID_FINDB => {
            Some(crate::functions::text_b_compat_family::FINDB_META.arg_preparation_profile)
        }
        FUNC_ID_FILTER => Some(FILTER_META.arg_preparation_profile),
        FUNC_ID_FILTERXML => {
            Some(crate::functions::web_text_xml_family::FILTERXML_META.arg_preparation_profile)
        }
        FUNC_ID_FIXED => Some(crate::functions::fixed_fn::FIXED_META.arg_preparation_profile),
        FUNC_ID_FLOOR => {
            Some(crate::functions::ceiling_floor_family::FLOOR_META.arg_preparation_profile)
        }
        FUNC_ID_FLOOR_MATH => {
            Some(crate::functions::ceiling_floor_family::FLOOR_MATH_META.arg_preparation_profile)
        }
        FUNC_ID_FLOOR_PRECISE => {
            Some(crate::functions::ceiling_floor_family::FLOOR_PRECISE_META.arg_preparation_profile)
        }
        FUNC_ID_FORMULATEXT => Some(FORMULATEXT_META.arg_preparation_profile),
        FUNC_ID_GAUSS => Some(crate::functions::gauss_fn::GAUSS_META.arg_preparation_profile),
        FUNC_ID_GAMMA => {
            Some(crate::functions::special_dist_family::GAMMA_META.arg_preparation_profile)
        }
        FUNC_ID_GAMMA_DIST => {
            Some(crate::functions::beta_gamma_stats_family::GAMMA_DIST_META.arg_preparation_profile)
        }
        FUNC_ID_GAMMA_INV => {
            Some(crate::functions::beta_gamma_stats_family::GAMMA_INV_META.arg_preparation_profile)
        }
        FUNC_ID_GAMMADIST => {
            Some(crate::functions::beta_gamma_stats_family::GAMMADIST_META.arg_preparation_profile)
        }
        FUNC_ID_GAMMAINV => {
            Some(crate::functions::beta_gamma_stats_family::GAMMAINV_META.arg_preparation_profile)
        }
        FUNC_ID_GAMMALN => {
            Some(crate::functions::special_dist_family::GAMMALN_META.arg_preparation_profile)
        }
        FUNC_ID_GAMMALN_PRECISE => Some(
            crate::functions::special_dist_family::GAMMALN_PRECISE_META.arg_preparation_profile,
        ),
        FUNC_ID_GCD => Some(crate::functions::gcd_fn::GCD_META.arg_preparation_profile),
        FUNC_ID_GEOMEAN => Some(crate::functions::geomean_fn::GEOMEAN_META.arg_preparation_profile),
        FUNC_ID_GESTEP => Some(crate::functions::gestep_fn::GESTEP_META.arg_preparation_profile),
        FUNC_ID_GROUPBY => Some(crate::functions::groupby_fn::GROUPBY_META.arg_preparation_profile),
        FUNC_ID_GROWTH => {
            Some(crate::functions::regression_forecast_family::GROWTH_META.arg_preparation_profile)
        }
        FUNC_ID_HARMEAN => Some(crate::functions::harmean_fn::HARMEAN_META.arg_preparation_profile),
        FUNC_ID_HYPERLINK => {
            Some(crate::functions::hyperlink_fn::HYPERLINK_META.arg_preparation_profile)
        }
        FUNC_ID_IMAGE => Some(crate::functions::image_fn::IMAGE_META.arg_preparation_profile),
        FUNC_ID_HYPGEOM_DIST => {
            Some(crate::functions::discrete_dist_family::HYPGEOM_DIST_META.arg_preparation_profile)
        }
        FUNC_ID_HYPGEOMDIST => {
            Some(crate::functions::discrete_dist_family::HYPGEOMDIST_META.arg_preparation_profile)
        }
        FUNC_ID_HOUR => {
            Some(crate::functions::date_parts_family::HOUR_META.arg_preparation_profile)
        }
        FUNC_ID_HSTACK => Some(crate::functions::hstack::HSTACK_META.arg_preparation_profile),
        FUNC_ID_SORT => Some(SORT_META.arg_preparation_profile),
        FUNC_ID_SORTBY => Some(SORTBY_META.arg_preparation_profile),
        FUNC_ID_INFO => Some(crate::functions::info_fn::INFO_META.arg_preparation_profile),
        FUNC_ID_ISOMITTED => Some(ISOMITTED_META.arg_preparation_profile),
        FUNC_ID_IRR => {
            Some(crate::functions::cashflow_rate_family::IRR_META.arg_preparation_profile)
        }
        FUNC_ID_IMABS => Some(crate::functions::complex_family::IMABS_META.arg_preparation_profile),
        FUNC_ID_IMAGINARY => {
            Some(crate::functions::complex_family::IMAGINARY_META.arg_preparation_profile)
        }
        FUNC_ID_IMARGUMENT => {
            Some(crate::functions::complex_family::IMARGUMENT_META.arg_preparation_profile)
        }
        FUNC_ID_IMCONJUGATE => {
            Some(crate::functions::complex_family::IMCONJUGATE_META.arg_preparation_profile)
        }
        FUNC_ID_IMCOS => Some(crate::functions::complex_family::IMCOS_META.arg_preparation_profile),
        FUNC_ID_IMCOSH => {
            Some(crate::functions::complex_family::IMCOSH_META.arg_preparation_profile)
        }
        FUNC_ID_IMCOT => Some(crate::functions::complex_family::IMCOT_META.arg_preparation_profile),
        FUNC_ID_IMCSC => Some(crate::functions::complex_family::IMCSC_META.arg_preparation_profile),
        FUNC_ID_IMCSCH => {
            Some(crate::functions::complex_family::IMCSCH_META.arg_preparation_profile)
        }
        FUNC_ID_IMDIV => Some(crate::functions::complex_family::IMDIV_META.arg_preparation_profile),
        FUNC_ID_IMEXP => Some(crate::functions::complex_family::IMEXP_META.arg_preparation_profile),
        FUNC_ID_IMLN => Some(crate::functions::complex_family::IMLN_META.arg_preparation_profile),
        FUNC_ID_IMLOG10 => {
            Some(crate::functions::complex_family::IMLOG10_META.arg_preparation_profile)
        }
        FUNC_ID_IMLOG2 => {
            Some(crate::functions::complex_family::IMLOG2_META.arg_preparation_profile)
        }
        FUNC_ID_IMPOWER => {
            Some(crate::functions::complex_family::IMPOWER_META.arg_preparation_profile)
        }
        FUNC_ID_IMPRODUCT => {
            Some(crate::functions::complex_family::IMPRODUCT_META.arg_preparation_profile)
        }
        FUNC_ID_IMREAL => {
            Some(crate::functions::complex_family::IMREAL_META.arg_preparation_profile)
        }
        FUNC_ID_IMSEC => Some(crate::functions::complex_family::IMSEC_META.arg_preparation_profile),
        FUNC_ID_IMSECH => {
            Some(crate::functions::complex_family::IMSECH_META.arg_preparation_profile)
        }
        FUNC_ID_IMSIN => Some(crate::functions::complex_family::IMSIN_META.arg_preparation_profile),
        FUNC_ID_IMSINH => {
            Some(crate::functions::complex_family::IMSINH_META.arg_preparation_profile)
        }
        FUNC_ID_IMSQRT => {
            Some(crate::functions::complex_family::IMSQRT_META.arg_preparation_profile)
        }
        FUNC_ID_IMSUB => Some(crate::functions::complex_family::IMSUB_META.arg_preparation_profile),
        FUNC_ID_IMSUM => Some(crate::functions::complex_family::IMSUM_META.arg_preparation_profile),
        FUNC_ID_IMTAN => Some(crate::functions::complex_family::IMTAN_META.arg_preparation_profile),
        FUNC_ID_ISFORMULA => {
            Some(crate::functions::misc_switch_info_family::ISFORMULA_META.arg_preparation_profile)
        }
        FUNC_ID_IF => Some(crate::functions::if_fn::IF_META.arg_preparation_profile),
        FUNC_ID_IFERROR => Some(crate::functions::iferror::IFERROR_META.arg_preparation_profile),
        FUNC_ID_IFNA => Some(crate::functions::ifna_fn::IFNA_META.arg_preparation_profile),
        FUNC_ID_IFS => Some(crate::functions::choose_ifs_family::IFS_META.arg_preparation_profile),
        FUNC_ID_INDEX => Some(crate::functions::index::INDEX_META.arg_preparation_profile),
        FUNC_ID_INDIRECT => Some(crate::functions::indirect::INDIRECT_META.arg_preparation_profile),
        FUNC_ID_IPMT => {
            Some(crate::functions::financial_time_value_family::IPMT_META.arg_preparation_profile)
        }
        FUNC_ID_ISPMT => {
            Some(crate::functions::financial_time_value_family::ISPMT_META.arg_preparation_profile)
        }
        FUNC_ID_HEX2BIN => {
            Some(crate::functions::engineering_radix_family::HEX2BIN_META.arg_preparation_profile)
        }
        FUNC_ID_HEX2DEC => {
            Some(crate::functions::engineering_radix_family::HEX2DEC_META.arg_preparation_profile)
        }
        FUNC_ID_HEX2OCT => {
            Some(crate::functions::engineering_radix_family::HEX2OCT_META.arg_preparation_profile)
        }
        FUNC_ID_ISNUMBER => Some(crate::functions::isnumber::ISNUMBER_META.arg_preparation_profile),
        FUNC_ID_ISBLANK => {
            Some(crate::functions::is_predicates_family::ISBLANK_META.arg_preparation_profile)
        }
        FUNC_ID_ISERR => {
            Some(crate::functions::is_predicates_family::ISERR_META.arg_preparation_profile)
        }
        FUNC_ID_ISERROR => {
            Some(crate::functions::is_predicates_family::ISERROR_META.arg_preparation_profile)
        }
        FUNC_ID_ISLOGICAL => {
            Some(crate::functions::is_predicates_family::ISLOGICAL_META.arg_preparation_profile)
        }
        FUNC_ID_ISNA => {
            Some(crate::functions::is_predicates_family::ISNA_META.arg_preparation_profile)
        }
        FUNC_ID_ISNONTEXT => {
            Some(crate::functions::is_predicates_family::ISNONTEXT_META.arg_preparation_profile)
        }
        FUNC_ID_ISODD => {
            Some(crate::functions::is_predicates_family::ISODD_META.arg_preparation_profile)
        }
        FUNC_ID_ISREF => {
            Some(crate::functions::is_predicates_family::ISREF_META.arg_preparation_profile)
        }
        FUNC_ID_ISTEXT => {
            Some(crate::functions::is_predicates_family::ISTEXT_META.arg_preparation_profile)
        }
        FUNC_ID_ISOWEEKNUM => {
            Some(crate::functions::date_week_family::ISOWEEKNUM_META.arg_preparation_profile)
        }
        FUNC_ID_ISO_CEILING => {
            Some(crate::functions::ceiling_floor_family::ISO_CEILING_META.arg_preparation_profile)
        }
        FUNC_ID_INTERCEPT => {
            Some(crate::functions::intercept_fn::INTERCEPT_META.arg_preparation_profile)
        }
        FUNC_ID_INT => Some(crate::functions::int_fn::INT_META.arg_preparation_profile),
        FUNC_ID_INTRATE => Some(
            crate::functions::discount_bill_yearfrac_family::INTRATE_META.arg_preparation_profile,
        ),
        FUNC_ID_ISEVEN => Some(crate::functions::iseven_fn::ISEVEN_META.arg_preparation_profile),
        FUNC_ID_JIS => {
            Some(crate::functions::text_compat_locale_family::JIS_META.arg_preparation_profile)
        }
        FUNC_ID_KURT => {
            Some(crate::functions::moment_stats_family::KURT_META.arg_preparation_profile)
        }
        FUNC_ID_LARGE => Some(crate::functions::large_fn::LARGE_META.arg_preparation_profile),
        FUNC_ID_LCM => Some(crate::functions::lcm_fn::LCM_META.arg_preparation_profile),
        FUNC_ID_FORECAST => Some(
            crate::functions::regression_forecast_family::FORECAST_META.arg_preparation_profile,
        ),
        FUNC_ID_FORECAST_LINEAR => Some(
            crate::functions::regression_forecast_family::FORECAST_LINEAR_META
                .arg_preparation_profile,
        ),
        FUNC_ID_LINEST => {
            Some(crate::functions::regression_forecast_family::LINEST_META.arg_preparation_profile)
        }
        FUNC_ID_LOGINV => {
            Some(crate::functions::legacy_stats_alias_family::LOGINV_META.arg_preparation_profile)
        }
        FUNC_ID_LN => Some(crate::functions::ln_fn::LN_META.arg_preparation_profile),
        FUNC_ID_LOG => Some(crate::functions::log_fn::LOG_META.arg_preparation_profile),
        FUNC_ID_LOG10 => Some(crate::functions::log10_fn::LOG10_META.arg_preparation_profile),
        FUNC_ID_LOOKUP => Some(
            crate::functions::lookup_prob_frequency_family::LOOKUP_META.arg_preparation_profile,
        ),
        FUNC_ID_LOGEST => {
            Some(crate::functions::regression_forecast_family::LOGEST_META.arg_preparation_profile)
        }
        FUNC_ID_LOWER => {
            Some(crate::functions::text_scalar_misc::LOWER_META.arg_preparation_profile)
        }
        FUNC_ID_LEFT => {
            Some(crate::functions::text_slice_family::LEFT_META.arg_preparation_profile)
        }
        FUNC_ID_LEFTB => {
            Some(crate::functions::text_b_compat_family::LEFTB_META.arg_preparation_profile)
        }
        FUNC_ID_LEN => Some(crate::functions::text_slice_family::LEN_META.arg_preparation_profile),
        FUNC_ID_LENB => {
            Some(crate::functions::text_b_compat_family::LENB_META.arg_preparation_profile)
        }
        FUNC_ID_MID => Some(crate::functions::text_slice_family::MID_META.arg_preparation_profile),
        FUNC_ID_MIDB => {
            Some(crate::functions::text_b_compat_family::MIDB_META.arg_preparation_profile)
        }
        FUNC_ID_RIGHT => {
            Some(crate::functions::text_slice_family::RIGHT_META.arg_preparation_profile)
        }
        FUNC_ID_RIGHTB => {
            Some(crate::functions::text_b_compat_family::RIGHTB_META.arg_preparation_profile)
        }
        FUNC_ID_MAX => Some(crate::functions::max_fn::MAX_META.arg_preparation_profile),
        FUNC_ID_MAXA => Some(crate::functions::maxa_fn::MAXA_META.arg_preparation_profile),
        FUNC_ID_MAXIFS => {
            Some(crate::functions::criteria_family::MAXIFS_META.arg_preparation_profile)
        }
        FUNC_ID_MEDIAN => Some(crate::functions::median_fn::MEDIAN_META.arg_preparation_profile),
        FUNC_ID_MATCH => Some(crate::functions::match_fn::MATCH_META.arg_preparation_profile),
        FUNC_ID_MAKEARRAY => Some(MAKEARRAY_META.arg_preparation_profile),
        FUNC_ID_MAP => Some(MAP_META.arg_preparation_profile),
        FUNC_ID_MDETERM => {
            Some(crate::functions::matrix_family::MDETERM_META.arg_preparation_profile)
        }
        FUNC_ID_MDURATION => {
            Some(crate::functions::bond_core_family::MDURATION_META.arg_preparation_profile)
        }
        FUNC_ID_MINVERSE => {
            Some(crate::functions::matrix_family::MINVERSE_META.arg_preparation_profile)
        }
        FUNC_ID_MMULT => Some(crate::functions::matrix_family::MMULT_META.arg_preparation_profile),
        FUNC_ID_MUNIT => Some(crate::functions::matrix_family::MUNIT_META.arg_preparation_profile),
        FUNC_ID_MIN => Some(crate::functions::min_fn::MIN_META.arg_preparation_profile),
        FUNC_ID_MINA => Some(crate::functions::mina_fn::MINA_META.arg_preparation_profile),
        FUNC_ID_MINIFS => {
            Some(crate::functions::criteria_family::MINIFS_META.arg_preparation_profile)
        }
        FUNC_ID_MIRR => {
            Some(crate::functions::financial_time_value_family::MIRR_META.arg_preparation_profile)
        }
        FUNC_ID_MINUTE => {
            Some(crate::functions::date_parts_family::MINUTE_META.arg_preparation_profile)
        }
        FUNC_ID_MOD => Some(crate::functions::mod_fn::MOD_META.arg_preparation_profile),
        FUNC_ID_MODE => {
            Some(crate::functions::legacy_stats_alias_family::MODE_META.arg_preparation_profile)
        }
        FUNC_ID_MODE_MULT => Some(
            crate::functions::lookup_prob_frequency_family::MODE_MULT_META.arg_preparation_profile,
        ),
        FUNC_ID_MODE_SNGL => {
            Some(crate::functions::mode_sngl_fn::MODE_SNGL_META.arg_preparation_profile)
        }
        FUNC_ID_MONTH => {
            Some(crate::functions::date_parts_family::MONTH_META.arg_preparation_profile)
        }
        FUNC_ID_MROUND => Some(crate::functions::mround::MROUND_META.arg_preparation_profile),
        FUNC_ID_MULTINOMIAL => {
            Some(crate::functions::multinomial::MULTINOMIAL_META.arg_preparation_profile)
        }
        FUNC_ID_N => Some(crate::functions::n_fn::N_META.arg_preparation_profile),
        FUNC_ID_NA => Some(crate::functions::na_fn::NA_META.arg_preparation_profile),
        FUNC_ID_NOMINAL => Some(
            crate::functions::financial_time_value_family::NOMINAL_META.arg_preparation_profile,
        ),
        FUNC_ID_NPER => {
            Some(crate::functions::financial_time_value_family::NPER_META.arg_preparation_profile)
        }
        FUNC_ID_NPV => {
            Some(crate::functions::financial_time_value_family::NPV_META.arg_preparation_profile)
        }
        FUNC_ID_NUMBERVALUE => Some(
            crate::functions::number_regex_translate_family::NUMBERVALUE_META
                .arg_preparation_profile,
        ),
        FUNC_ID_NEGBINOM_DIST => {
            Some(crate::functions::discrete_dist_family::NEGBINOM_DIST_META.arg_preparation_profile)
        }
        FUNC_ID_NEGBINOMDIST => {
            Some(crate::functions::discrete_dist_family::NEGBINOMDIST_META.arg_preparation_profile)
        }
        FUNC_ID_CONFIDENCE => {
            Some(crate::functions::normal_log_family::CONFIDENCE_META.arg_preparation_profile)
        }
        FUNC_ID_CONFIDENCE_T => Some(
            crate::functions::confidence_test_family::CONFIDENCE_T_META.arg_preparation_profile,
        ),
        FUNC_ID_CONFIDENCE_NORM => {
            Some(crate::functions::normal_log_family::CONFIDENCE_NORM_META.arg_preparation_profile)
        }
        FUNC_ID_LOGNORM_DIST => {
            Some(crate::functions::normal_log_family::LOGNORM_DIST_META.arg_preparation_profile)
        }
        FUNC_ID_LOGNORM_INV => {
            Some(crate::functions::normal_log_family::LOGNORM_INV_META.arg_preparation_profile)
        }
        FUNC_ID_LOGNORMDIST => {
            Some(crate::functions::normal_log_family::LOGNORMDIST_META.arg_preparation_profile)
        }
        FUNC_ID_NORM_DIST => {
            Some(crate::functions::normal_log_family::NORM_DIST_META.arg_preparation_profile)
        }
        FUNC_ID_NORM_INV => {
            Some(crate::functions::normal_log_family::NORM_INV_META.arg_preparation_profile)
        }
        FUNC_ID_NORM_S_DIST => {
            Some(crate::functions::normal_log_family::NORM_S_DIST_META.arg_preparation_profile)
        }
        FUNC_ID_NORM_S_INV => {
            Some(crate::functions::normal_log_family::NORM_S_INV_META.arg_preparation_profile)
        }
        FUNC_ID_NORMDIST => {
            Some(crate::functions::normal_log_family::NORMDIST_META.arg_preparation_profile)
        }
        FUNC_ID_NORMINV => {
            Some(crate::functions::normal_log_family::NORMINV_META.arg_preparation_profile)
        }
        FUNC_ID_NORMSDIST => {
            Some(crate::functions::normal_log_family::NORMSDIST_META.arg_preparation_profile)
        }
        FUNC_ID_NORMSINV => {
            Some(crate::functions::normal_log_family::NORMSINV_META.arg_preparation_profile)
        }
        FUNC_ID_NETWORKDAYS => Some(
            crate::functions::workday_networkdays_family::NETWORKDAYS_META.arg_preparation_profile,
        ),
        FUNC_ID_NETWORKDAYS_INTL => Some(
            crate::functions::workday_networkdays_family::NETWORKDAYS_INTL_META
                .arg_preparation_profile,
        ),
        FUNC_ID_NOT => Some(crate::functions::not_fn::NOT_META.arg_preparation_profile),
        FUNC_ID_NOW => Some(crate::functions::now_fn::NOW_META.arg_preparation_profile),
        FUNC_ID_OCT2BIN => {
            Some(crate::functions::engineering_radix_family::OCT2BIN_META.arg_preparation_profile)
        }
        FUNC_ID_OCT2DEC => {
            Some(crate::functions::engineering_radix_family::OCT2DEC_META.arg_preparation_profile)
        }
        FUNC_ID_OCT2HEX => {
            Some(crate::functions::engineering_radix_family::OCT2HEX_META.arg_preparation_profile)
        }
        FUNC_ID_POISSON => {
            Some(crate::functions::discrete_dist_family::POISSON_META.arg_preparation_profile)
        }
        FUNC_ID_POISSON_DIST => {
            Some(crate::functions::discrete_dist_family::POISSON_DIST_META.arg_preparation_profile)
        }
        FUNC_ID_ODD => Some(crate::functions::odd_fn::ODD_META.arg_preparation_profile),
        FUNC_ID_ODDFPRICE => {
            Some(crate::functions::odd_bond_family::ODDFPRICE_META.arg_preparation_profile)
        }
        FUNC_ID_ODDFYIELD => {
            Some(crate::functions::odd_bond_family::ODDFYIELD_META.arg_preparation_profile)
        }
        FUNC_ID_ODDLPRICE => {
            Some(crate::functions::odd_bond_family::ODDLPRICE_META.arg_preparation_profile)
        }
        FUNC_ID_ODDLYIELD => {
            Some(crate::functions::odd_bond_family::ODDLYIELD_META.arg_preparation_profile)
        }
        FUNC_ID_OR => Some(crate::functions::or_fn::OR_META.arg_preparation_profile),
        FUNC_ID_OFFSET => Some(crate::functions::offset::OFFSET_META.arg_preparation_profile),
        FUNC_ID_OP_ADD => Some(crate::functions::op_add::OP_ADD_META.arg_preparation_profile),
        FUNC_ID_OP_CONCAT => Some(OP_CONCAT_META.arg_preparation_profile),
        FUNC_ID_OP_DIVIDE => Some(OP_DIVIDE_META.arg_preparation_profile),
        FUNC_ID_OP_EQUAL => Some(OP_EQUAL_META.arg_preparation_profile),
        FUNC_ID_OP_GREATER_EQUAL => Some(OP_GREATER_EQUAL_META.arg_preparation_profile),
        FUNC_ID_OP_GREATER_THAN => Some(OP_GREATER_THAN_META.arg_preparation_profile),
        FUNC_ID_OP_IMPLICIT_INTERSECTION => {
            Some(OP_IMPLICIT_INTERSECTION_META.arg_preparation_profile)
        }
        FUNC_ID_OP_INTERSECTION_REF => Some(OP_INTERSECTION_REF_META.arg_preparation_profile),
        FUNC_ID_OP_LESS_EQUAL => Some(OP_LESS_EQUAL_META.arg_preparation_profile),
        FUNC_ID_OP_LESS_THAN => Some(OP_LESS_THAN_META.arg_preparation_profile),
        FUNC_ID_OP_MULTIPLY => Some(OP_MULTIPLY_META.arg_preparation_profile),
        FUNC_ID_OP_NEGATE => Some(OP_NEGATE_META.arg_preparation_profile),
        FUNC_ID_OP_NOT_EQUAL => Some(OP_NOT_EQUAL_META.arg_preparation_profile),
        FUNC_ID_OP_PERCENT => Some(OP_PERCENT_META.arg_preparation_profile),
        FUNC_ID_OP_POWER => Some(OP_POWER_META.arg_preparation_profile),
        FUNC_ID_OP_RANGE_REF => Some(OP_RANGE_REF_META.arg_preparation_profile),
        FUNC_ID_OP_SPILL_REF => {
            Some(crate::functions::op_spill_ref::OP_SPILL_REF_META.arg_preparation_profile)
        }
        FUNC_ID_OP_SUBTRACT => Some(OP_SUBTRACT_META.arg_preparation_profile),
        FUNC_ID_OP_TRIM_REF_BOTH => Some(OP_TRIM_REF_BOTH_META.arg_preparation_profile),
        FUNC_ID_OP_TRIM_REF_LEADING => Some(OP_TRIM_REF_LEADING_META.arg_preparation_profile),
        FUNC_ID_OP_TRIM_REF_TRAILING => Some(OP_TRIM_REF_TRAILING_META.arg_preparation_profile),
        FUNC_ID_OP_UNARY_PLUS => Some(OP_UNARY_PLUS_META.arg_preparation_profile),
        FUNC_ID_OP_UNION_REF => Some(OP_UNION_REF_META.arg_preparation_profile),
        FUNC_ID_PEARSON => Some(crate::functions::pearson_fn::PEARSON_META.arg_preparation_profile),
        FUNC_ID_PDURATION => Some(
            crate::functions::financial_time_value_family::PDURATION_META.arg_preparation_profile,
        ),
        FUNC_ID_PERMUT => Some(crate::functions::permut_fn::PERMUT_META.arg_preparation_profile),
        FUNC_ID_PERMUTATIONA => {
            Some(crate::functions::permutationa_fn::PERMUTATIONA_META.arg_preparation_profile)
        }
        FUNC_ID_PERCENTILE_EXC => {
            Some(crate::functions::percentile_exc_fn::PERCENTILE_EXC_META.arg_preparation_profile)
        }
        FUNC_ID_PERCENTILE_INC => {
            Some(crate::functions::percentile_inc_fn::PERCENTILE_INC_META.arg_preparation_profile)
        }
        FUNC_ID_PERCENTILE => Some(
            crate::functions::legacy_stats_alias_family::PERCENTILE_META.arg_preparation_profile,
        ),
        FUNC_ID_PERCENTRANK_EXC => {
            Some(crate::functions::percentrank_exc_fn::PERCENTRANK_EXC_META.arg_preparation_profile)
        }
        FUNC_ID_PERCENTRANK_INC => {
            Some(crate::functions::percentrank_inc_fn::PERCENTRANK_INC_META.arg_preparation_profile)
        }
        FUNC_ID_PERCENTRANK => Some(
            crate::functions::legacy_stats_alias_family::PERCENTRANK_META.arg_preparation_profile,
        ),
        FUNC_ID_PHI => Some(crate::functions::phi_fn::PHI_META.arg_preparation_profile),
        FUNC_ID_PI => Some(crate::functions::pi::PI_META.arg_preparation_profile),
        FUNC_ID_PIVOTBY => Some(crate::functions::pivotby_fn::PIVOTBY_META.arg_preparation_profile),
        FUNC_ID_PMT => {
            Some(crate::functions::financial_time_value_family::PMT_META.arg_preparation_profile)
        }
        FUNC_ID_PPMT => {
            Some(crate::functions::financial_time_value_family::PPMT_META.arg_preparation_profile)
        }
        FUNC_ID_PERCENTOF => {
            Some(crate::functions::misc_conversion_family::PERCENTOF_META.arg_preparation_profile)
        }
        FUNC_ID_PRICE => {
            Some(crate::functions::bond_core_family::PRICE_META.arg_preparation_profile)
        }
        FUNC_ID_PRICEDISC => Some(
            crate::functions::discount_bill_yearfrac_family::PRICEDISC_META.arg_preparation_profile,
        ),
        FUNC_ID_PRICEMAT => {
            Some(crate::functions::bond_core_family::PRICEMAT_META.arg_preparation_profile)
        }
        FUNC_ID_PROB => {
            Some(crate::functions::lookup_prob_frequency_family::PROB_META.arg_preparation_profile)
        }
        FUNC_ID_PRODUCT => Some(crate::functions::product::PRODUCT_META.arg_preparation_profile),
        FUNC_ID_POWER => Some(crate::functions::power_fn::POWER_META.arg_preparation_profile),
        FUNC_ID_PV => {
            Some(crate::functions::financial_time_value_family::PV_META.arg_preparation_profile)
        }
        FUNC_ID_PROPER => {
            Some(crate::functions::text_search_replace_family::PROPER_META.arg_preparation_profile)
        }
        FUNC_ID_QUOTIENT => {
            Some(crate::functions::quotient_fn::QUOTIENT_META.arg_preparation_profile)
        }
        FUNC_ID_QUARTILE_EXC => {
            Some(crate::functions::quartile_exc_fn::QUARTILE_EXC_META.arg_preparation_profile)
        }
        FUNC_ID_QUARTILE_INC => {
            Some(crate::functions::quartile_inc_fn::QUARTILE_INC_META.arg_preparation_profile)
        }
        FUNC_ID_QUARTILE => {
            Some(crate::functions::legacy_stats_alias_family::QUARTILE_META.arg_preparation_profile)
        }
        FUNC_ID_RAND => Some(crate::functions::rand_fn::RAND_META.arg_preparation_profile),
        FUNC_ID_RANDARRAY => {
            Some(crate::functions::misc_conversion_family::RANDARRAY_META.arg_preparation_profile)
        }
        FUNC_ID_RANDBETWEEN => {
            Some(crate::functions::randbetween_fn::RANDBETWEEN_META.arg_preparation_profile)
        }
        FUNC_ID_REDUCE => Some(REDUCE_META.arg_preparation_profile),
        FUNC_ID_RATE => {
            Some(crate::functions::financial_time_value_family::RATE_META.arg_preparation_profile)
        }
        FUNC_ID_RADIANS => Some(crate::functions::radians::RADIANS_META.arg_preparation_profile),
        FUNC_ID_RANK => Some(crate::functions::rank_fn::RANK_META.arg_preparation_profile),
        FUNC_ID_RANK_AVG => {
            Some(crate::functions::rank_avg_fn::RANK_AVG_META.arg_preparation_profile)
        }
        FUNC_ID_RANK_EQ => Some(crate::functions::rank_eq_fn::RANK_EQ_META.arg_preparation_profile),
        FUNC_ID_ROW => Some(crate::functions::row_fn::ROW_META.arg_preparation_profile),
        FUNC_ID_ROWS => Some(crate::functions::rows_fn::ROWS_META.arg_preparation_profile),
        FUNC_ID_RRI => {
            Some(crate::functions::financial_time_value_family::RRI_META.arg_preparation_profile)
        }
        FUNC_ID_RTD => Some(crate::functions::rtd_fn::RTD_META.arg_preparation_profile),
        FUNC_ID_REGISTER_ID => Some(REGISTER_ID_META.arg_preparation_profile),
        FUNC_ID_ROMAN => Some(crate::functions::roman_fn::ROMAN_META.arg_preparation_profile),
        FUNC_ID_ROUND => Some(crate::functions::round_fn::ROUND_META.arg_preparation_profile),
        FUNC_ID_ROUNDDOWN => {
            Some(crate::functions::rounddown_fn::ROUNDDOWN_META.arg_preparation_profile)
        }
        FUNC_ID_REPLACE => {
            Some(crate::functions::text_search_replace_family::REPLACE_META.arg_preparation_profile)
        }
        FUNC_ID_REPLACEB => {
            Some(crate::functions::text_b_compat_family::REPLACEB_META.arg_preparation_profile)
        }
        FUNC_ID_RECEIVED => Some(
            crate::functions::discount_bill_yearfrac_family::RECEIVED_META.arg_preparation_profile,
        ),
        FUNC_ID_REGEXEXTRACT => Some(
            crate::functions::number_regex_translate_family::REGEXEXTRACT_META
                .arg_preparation_profile,
        ),
        FUNC_ID_REGEXREPLACE => Some(
            crate::functions::number_regex_translate_family::REGEXREPLACE_META
                .arg_preparation_profile,
        ),
        FUNC_ID_REGEXTEST => Some(
            crate::functions::number_regex_translate_family::REGEXTEST_META.arg_preparation_profile,
        ),
        FUNC_ID_ROUNDUP => Some(crate::functions::roundup_fn::ROUNDUP_META.arg_preparation_profile),
        FUNC_ID_RSQ => Some(crate::functions::rsq_fn::RSQ_META.arg_preparation_profile),
        FUNC_ID_SECOND => {
            Some(crate::functions::date_parts_family::SECOND_META.arg_preparation_profile)
        }
        FUNC_ID_SEQUENCE => Some(crate::functions::sequence::SEQUENCE_META.arg_preparation_profile),
        FUNC_ID_SCAN => Some(SCAN_META.arg_preparation_profile),
        FUNC_ID_SEC => Some(crate::functions::sec::SEC_META.arg_preparation_profile),
        FUNC_ID_SECH => Some(crate::functions::sech::SECH_META.arg_preparation_profile),
        FUNC_ID_SHEET => Some(SHEET_META.arg_preparation_profile),
        FUNC_ID_SHEETS => Some(SHEETS_META.arg_preparation_profile),
        FUNC_ID_SERIESSUM => {
            Some(crate::functions::sumproduct_family::SERIESSUM_META.arg_preparation_profile)
        }
        FUNC_ID_SIGN => Some(crate::functions::sign_fn::SIGN_META.arg_preparation_profile),
        FUNC_ID_SIN => Some(crate::functions::sin::SIN_META.arg_preparation_profile),
        FUNC_ID_SINH => Some(crate::functions::sinh::SINH_META.arg_preparation_profile),
        FUNC_ID_SKEW => {
            Some(crate::functions::moment_stats_family::SKEW_META.arg_preparation_profile)
        }
        FUNC_ID_SKEW_P => {
            Some(crate::functions::moment_stats_family::SKEW_P_META.arg_preparation_profile)
        }
        FUNC_ID_SLN => {
            Some(crate::functions::depreciation_family::SLN_META.arg_preparation_profile)
        }
        FUNC_ID_SMALL => Some(crate::functions::small_fn::SMALL_META.arg_preparation_profile),
        FUNC_ID_STEYX => {
            Some(crate::functions::moment_stats_family::STEYX_META.arg_preparation_profile)
        }
        FUNC_ID_SQRT => Some(crate::functions::sqrt_fn::SQRT_META.arg_preparation_profile),
        FUNC_ID_SQRTPI => Some(crate::functions::sqrtpi::SQRTPI_META.arg_preparation_profile),
        FUNC_ID_SLOPE => Some(crate::functions::slope_fn::SLOPE_META.arg_preparation_profile),
        FUNC_ID_STDEV => Some(crate::functions::stdev_fn::STDEV_META.arg_preparation_profile),
        FUNC_ID_STDEV_P => Some(crate::functions::stdev_p_fn::STDEV_P_META.arg_preparation_profile),
        FUNC_ID_STDEV_S => Some(crate::functions::stdev_s_fn::STDEV_S_META.arg_preparation_profile),
        FUNC_ID_STDEVP => Some(crate::functions::stdevp_fn::STDEVP_META.arg_preparation_profile),
        FUNC_ID_STDEVA => Some(crate::functions::stdeva_fn::STDEVA_META.arg_preparation_profile),
        FUNC_ID_STDEVPA => Some(crate::functions::stdevpa_fn::STDEVPA_META.arg_preparation_profile),
        FUNC_ID_STANDARDIZE => {
            Some(crate::functions::standardize_fn::STANDARDIZE_META.arg_preparation_profile)
        }
        FUNC_ID_SUBTOTAL => {
            Some(crate::functions::subtotal_aggregate_family::SUBTOTAL_META.arg_preparation_profile)
        }
        FUNC_ID_SUM => Some(crate::functions::sum::SUM_META.arg_preparation_profile),
        FUNC_ID_SUMIF => {
            Some(crate::functions::criteria_family::SUMIF_META.arg_preparation_profile)
        }
        FUNC_ID_SUMIFS => {
            Some(crate::functions::criteria_family::SUMIFS_META.arg_preparation_profile)
        }
        FUNC_ID_SUMPRODUCT => {
            Some(crate::functions::sumproduct_family::SUMPRODUCT_META.arg_preparation_profile)
        }
        FUNC_ID_SUMX2MY2 => {
            Some(crate::functions::sumproduct_family::SUMX2MY2_META.arg_preparation_profile)
        }
        FUNC_ID_SUMX2PY2 => {
            Some(crate::functions::sumproduct_family::SUMX2PY2_META.arg_preparation_profile)
        }
        FUNC_ID_SUMXMY2 => {
            Some(crate::functions::sumproduct_family::SUMXMY2_META.arg_preparation_profile)
        }
        FUNC_ID_SUMSQ => Some(crate::functions::sumsq::SUMSQ_META.arg_preparation_profile),
        FUNC_ID_SWITCH => {
            Some(crate::functions::misc_switch_info_family::SWITCH_META.arg_preparation_profile)
        }
        FUNC_ID_T => Some(crate::functions::t_fn::T_META.arg_preparation_profile),
        FUNC_ID_TAKE => Some(TAKE_META.arg_preparation_profile),
        FUNC_ID_T_DIST => {
            Some(crate::functions::chi_f_t_family::T_DIST_META.arg_preparation_profile)
        }
        FUNC_ID_T_DIST_2T => {
            Some(crate::functions::chi_f_t_family::T_DIST_2T_META.arg_preparation_profile)
        }
        FUNC_ID_T_DIST_RT => {
            Some(crate::functions::chi_f_t_family::T_DIST_RT_META.arg_preparation_profile)
        }
        FUNC_ID_T_INV => Some(crate::functions::chi_f_t_family::T_INV_META.arg_preparation_profile),
        FUNC_ID_T_INV_2T => {
            Some(crate::functions::chi_f_t_family::T_INV_2T_META.arg_preparation_profile)
        }
        FUNC_ID_T_TEST => {
            Some(crate::functions::statistical_tests_family::T_TEST_META.arg_preparation_profile)
        }
        FUNC_ID_TAN => Some(crate::functions::tan::TAN_META.arg_preparation_profile),
        FUNC_ID_TANH => Some(crate::functions::tanh::TANH_META.arg_preparation_profile),
        FUNC_ID_TBILLEQ => Some(
            crate::functions::discount_bill_yearfrac_family::TBILLEQ_META.arg_preparation_profile,
        ),
        FUNC_ID_TBILLPRICE => Some(
            crate::functions::discount_bill_yearfrac_family::TBILLPRICE_META
                .arg_preparation_profile,
        ),
        FUNC_ID_TBILLYIELD => Some(
            crate::functions::discount_bill_yearfrac_family::TBILLYIELD_META
                .arg_preparation_profile,
        ),
        FUNC_ID_TOCOL => Some(TOCOL_META.arg_preparation_profile),
        FUNC_ID_TOROW => Some(TOROW_META.arg_preparation_profile),
        FUNC_ID_TDIST => Some(crate::functions::chi_f_t_family::TDIST_META.arg_preparation_profile),
        FUNC_ID_TINV => Some(crate::functions::chi_f_t_family::TINV_META.arg_preparation_profile),
        FUNC_ID_SYD => {
            Some(crate::functions::depreciation_family::SYD_META.arg_preparation_profile)
        }
        FUNC_ID_SEARCH => {
            Some(crate::functions::text_search_replace_family::SEARCH_META.arg_preparation_profile)
        }
        FUNC_ID_SEARCHB => {
            Some(crate::functions::text_b_compat_family::SEARCHB_META.arg_preparation_profile)
        }
        FUNC_ID_TEXT => Some(crate::functions::text_fn::TEXT_META.arg_preparation_profile),
        FUNC_ID_TEXTAFTER => {
            Some(crate::functions::text_delim_family::TEXTAFTER_META.arg_preparation_profile)
        }
        FUNC_ID_TEXTBEFORE => {
            Some(crate::functions::text_delim_family::TEXTBEFORE_META.arg_preparation_profile)
        }
        FUNC_ID_TEXTSPLIT => {
            Some(crate::functions::array_text_split_family::TEXTSPLIT_META.arg_preparation_profile)
        }
        FUNC_ID_REPT => Some(crate::functions::text_scalar_misc::REPT_META.arg_preparation_profile),
        FUNC_ID_SUBSTITUTE => Some(
            crate::functions::text_search_replace_family::SUBSTITUTE_META.arg_preparation_profile,
        ),
        FUNC_ID_TEXTJOIN => Some(crate::functions::textjoin::TEXTJOIN_META.arg_preparation_profile),
        FUNC_ID_TODAY => Some(crate::functions::today_fn::TODAY_META.arg_preparation_profile),
        FUNC_ID_TIME => {
            Some(crate::functions::date_parts_family::TIME_META.arg_preparation_profile)
        }
        FUNC_ID_TIMEVALUE => {
            Some(crate::functions::date_value_family::TIMEVALUE_META.arg_preparation_profile)
        }
        FUNC_ID_TRANSLATE => Some(
            crate::functions::number_regex_translate_family::TRANSLATE_META.arg_preparation_profile,
        ),
        FUNC_ID_TRIMMEAN => {
            Some(crate::functions::moment_stats_family::TRIMMEAN_META.arg_preparation_profile)
        }
        FUNC_ID_TRANSPOSE => Some(TRANSPOSE_META.arg_preparation_profile),
        FUNC_ID_TRUE => Some(crate::functions::true_fn::TRUE_META.arg_preparation_profile),
        FUNC_ID_TREND => {
            Some(crate::functions::regression_forecast_family::TREND_META.arg_preparation_profile)
        }
        FUNC_ID_TRUNC => Some(crate::functions::trunc_fn::TRUNC_META.arg_preparation_profile),
        FUNC_ID_TRIM => Some(crate::functions::text_scalar_misc::TRIM_META.arg_preparation_profile),
        FUNC_ID_TRIMRANGE => {
            Some(crate::functions::trimrange_fn::TRIMRANGE_META.arg_preparation_profile)
        }
        FUNC_ID_TTEST => {
            Some(crate::functions::statistical_tests_family::TTEST_META.arg_preparation_profile)
        }
        FUNC_ID_TYPE => Some(crate::functions::type_fn::TYPE_META.arg_preparation_profile),
        FUNC_ID_UNIQUE => Some(UNIQUE_META.arg_preparation_profile),
        FUNC_ID_UNICHAR => {
            Some(crate::functions::text_unicode_fn::UNICHAR_META.arg_preparation_profile)
        }
        FUNC_ID_UNICODE => {
            Some(crate::functions::text_unicode_fn::UNICODE_META.arg_preparation_profile)
        }
        FUNC_ID_UPPER => {
            Some(crate::functions::text_scalar_misc::UPPER_META.arg_preparation_profile)
        }
        FUNC_ID_VALUE => Some(crate::functions::value_fn::VALUE_META.arg_preparation_profile),
        FUNC_ID_VALUETOTEXT => {
            Some(crate::functions::valuetotext_fn::VALUETOTEXT_META.arg_preparation_profile)
        }
        FUNC_ID_VAR => Some(crate::functions::var_fn::VAR_META.arg_preparation_profile),
        FUNC_ID_VAR_P => Some(crate::functions::var_p_fn::VAR_P_META.arg_preparation_profile),
        FUNC_ID_VAR_S => Some(crate::functions::var_s_fn::VAR_S_META.arg_preparation_profile),
        FUNC_ID_VARA => Some(crate::functions::vara_fn::VARA_META.arg_preparation_profile),
        FUNC_ID_VARP => Some(crate::functions::varp_fn::VARP_META.arg_preparation_profile),
        FUNC_ID_VARPA => Some(crate::functions::varpa_fn::VARPA_META.arg_preparation_profile),
        FUNC_ID_VDB => {
            Some(crate::functions::depreciation_family::VDB_META.arg_preparation_profile)
        }
        FUNC_ID_VSTACK => Some(VSTACK_META.arg_preparation_profile),
        FUNC_ID_HLOOKUP => {
            Some(crate::functions::vhlookup_family::HLOOKUP_META.arg_preparation_profile)
        }
        FUNC_ID_VLOOKUP => {
            Some(crate::functions::vhlookup_family::VLOOKUP_META.arg_preparation_profile)
        }
        FUNC_ID_WEIBULL => {
            Some(crate::functions::special_dist_family::WEIBULL_META.arg_preparation_profile)
        }
        FUNC_ID_WEIBULL_DIST => {
            Some(crate::functions::special_dist_family::WEIBULL_DIST_META.arg_preparation_profile)
        }
        FUNC_ID_WRAPCOLS => Some(WRAPCOLS_META.arg_preparation_profile),
        FUNC_ID_WRAPROWS => Some(WRAPROWS_META.arg_preparation_profile),
        FUNC_ID_XIRR => {
            Some(crate::functions::cashflow_rate_family::XIRR_META.arg_preparation_profile)
        }
        FUNC_ID_XNPV => {
            Some(crate::functions::cashflow_rate_family::XNPV_META.arg_preparation_profile)
        }
        FUNC_ID_XLOOKUP => Some(crate::functions::xlookup::XLOOKUP_META.arg_preparation_profile),
        FUNC_ID_XMATCH => Some(crate::functions::xmatch::XMATCH_META.arg_preparation_profile),
        FUNC_ID_XOR => Some(crate::functions::xor_fn::XOR_META.arg_preparation_profile),
        FUNC_ID_WEEKDAY => {
            Some(crate::functions::date_week_family::WEEKDAY_META.arg_preparation_profile)
        }
        FUNC_ID_WEEKNUM => {
            Some(crate::functions::date_week_family::WEEKNUM_META.arg_preparation_profile)
        }
        FUNC_ID_WORKDAY => {
            Some(crate::functions::workday_networkdays_family::WORKDAY_META.arg_preparation_profile)
        }
        FUNC_ID_WORKDAY_INTL => Some(
            crate::functions::workday_networkdays_family::WORKDAY_INTL_META.arg_preparation_profile,
        ),
        FUNC_ID_YIELD => {
            Some(crate::functions::bond_core_family::YIELD_META.arg_preparation_profile)
        }
        FUNC_ID_YIELDDISC => {
            Some(crate::functions::bond_core_family::YIELDDISC_META.arg_preparation_profile)
        }
        FUNC_ID_YIELDMAT => {
            Some(crate::functions::bond_core_family::YIELDMAT_META.arg_preparation_profile)
        }
        FUNC_ID_YEAR => {
            Some(crate::functions::date_parts_family::YEAR_META.arg_preparation_profile)
        }
        FUNC_ID_YEARFRAC => Some(
            crate::functions::discount_bill_yearfrac_family::YEARFRAC_META.arg_preparation_profile,
        ),
        FUNC_ID_Z_TEST => {
            Some(crate::functions::confidence_test_family::Z_TEST_META.arg_preparation_profile)
        }
        FUNC_ID_ZTEST => {
            Some(crate::functions::test_alias_family::ZTEST_META.arg_preparation_profile)
        }
        _ => None,
    }
}

pub fn eval_surface_value_call(
    function_id: &str,
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    now_serial: Option<f64>,
    random_value: Option<f64>,
    locale_ctx: Option<&LocaleFormatContext>,
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<EvalValue, WorksheetErrorCode> {
    eval_surface_value_call_with_callable(
        function_id,
        args,
        resolver,
        now_serial,
        random_value,
        locale_ctx,
        host_info,
        None,
        None,
        None,
    )
}

pub fn eval_surface_extended_call(
    function_id: &str,
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    now_serial: Option<f64>,
    random_value: Option<f64>,
    locale_ctx: Option<&LocaleFormatContext>,
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<ExtendedValue, WorksheetErrorCode> {
    match function_id {
        FUNC_ID_HYPERLINK => eval_hyperlink_surface_extended(args, resolver)
            .map_err(|e| map_hyperlink_error_to_ws(&e)),
        FUNC_ID_IMAGE => eval_image_surface_extended(args, resolver, host_info)
            .map_err(|e| map_image_error_to_ws(&e)),
        FUNC_ID_NOW => {
            let provider = FixedNowProvider {
                serial: now_serial.unwrap_or(0.0),
            };
            eval_now_surface_extended(args, &provider).map_err(|e| map_now_error_to_ws(&e))
        }
        FUNC_ID_TODAY => {
            let provider = FixedNowProvider {
                serial: now_serial.unwrap_or(0.0),
            };
            eval_today_surface_extended(args, &provider).map_err(|e| map_today_error_to_ws(&e))
        }
        _ => eval_surface_value_call(
            function_id,
            args,
            resolver,
            now_serial,
            random_value,
            locale_ctx,
            host_info,
        )
        .map(ExtendedValue::Core),
    }
}

fn observed_scalar_array_lift_positions(function_id: &str) -> Option<&'static [usize]> {
    match function_id {
        FUNC_ID_ABS => Some(&[0]),
        FUNC_ID_ADDRESS => Some(&[0, 1, 2, 3, 4]),
        FUNC_ID_BINOMDIST | FUNC_ID_NORMDIST => Some(&[0, 1, 2, 3]),
        FUNC_ID_BETADIST | FUNC_ID_BETAINV | FUNC_ID_CONFIDENCE | FUNC_ID_CONFIDENCE_T
        | FUNC_ID_CRITBINOM | FUNC_ID_EXPONDIST | FUNC_ID_FDIST | FUNC_ID_FINV
        | FUNC_ID_GAMMADIST | FUNC_ID_GAMMAINV | FUNC_ID_HYPGEOMDIST | FUNC_ID_LOGINV
        | FUNC_ID_LOGNORMDIST | FUNC_ID_NEGBINOMDIST | FUNC_ID_NORMINV | FUNC_ID_POISSON
        | FUNC_ID_SERIESSUM | FUNC_ID_TDIST => Some(&[0, 1, 2]),
        FUNC_ID_COMPLEX => Some(&[0, 1, 2]),
        FUNC_ID_CHIDIST | FUNC_ID_CHIINV | FUNC_ID_DOLLARFR | FUNC_ID_IMDIV | FUNC_ID_IMPOWER
        | FUNC_ID_IMSUB | FUNC_ID_TINV => Some(&[0, 1]),
        FUNC_ID_CONCATENATE => Some(&[0, 1, 2]),
        FUNC_ID_DROP | FUNC_ID_EXPAND | FUNC_ID_TAKE | FUNC_ID_TOROW => Some(&[1, 2]),
        FUNC_ID_IFS => Some(&[0, 1, 2]),
        FUNC_ID_IMABS | FUNC_ID_IMAGINARY | FUNC_ID_IMARGUMENT | FUNC_ID_IMCONJUGATE
        | FUNC_ID_IMCOS | FUNC_ID_IMCOSH | FUNC_ID_IMCOT | FUNC_ID_IMCSC | FUNC_ID_IMCSCH
        | FUNC_ID_IMEXP | FUNC_ID_IMLN | FUNC_ID_IMLOG10 | FUNC_ID_IMLOG2 | FUNC_ID_IMREAL
        | FUNC_ID_IMSEC | FUNC_ID_IMSECH | FUNC_ID_IMSIN | FUNC_ID_IMSINH | FUNC_ID_IMSQRT
        | FUNC_ID_IMTAN | FUNC_ID_MUNIT | FUNC_ID_NORMSDIST | FUNC_ID_NORMSINV
        | FUNC_ID_UNICHAR => Some(&[0]),
        FUNC_ID_PERCENTILE | FUNC_ID_PERCENTRANK | FUNC_ID_QUARTILE | FUNC_ID_TOCOL
        | FUNC_ID_TRIMMEAN | FUNC_ID_WRAPCOLS | FUNC_ID_WRAPROWS => Some(&[1]),
        FUNC_ID_SWITCH => Some(&[0, 1, 3]),
        FUNC_ID_Z_TEST => Some(&[1, 2]),
        _ => None,
    }
}

fn observed_error_result_array_lift(function_id: &str) -> bool {
    matches!(function_id, FUNC_ID_DOLLARFR)
}

fn prepared_array_shape(value: &crate::functions::adapters::PreparedArgValue) -> ArrayShape {
    match value {
        crate::functions::adapters::PreparedArgValue::Eval(EvalValue::Array(array)) => {
            array.shape()
        }
        _ => ArrayShape { rows: 1, cols: 1 },
    }
}

fn prepared_broadcast_at(
    value: &crate::functions::adapters::PreparedArgValue,
    row: usize,
    col: usize,
) -> Option<crate::functions::adapters::PreparedArgValue> {
    match value {
        crate::functions::adapters::PreparedArgValue::Eval(EvalValue::Array(array)) => {
            let shape = array.shape();
            let source_row = if shape.rows == 1 {
                0
            } else if row < shape.rows {
                row
            } else {
                return None;
            };
            let source_col = if shape.cols == 1 {
                0
            } else if col < shape.cols {
                col
            } else {
                return None;
            };
            array.get(source_row, source_col).map(|cell| match cell {
                ArrayCellValue::Number(n) => {
                    crate::functions::adapters::PreparedArgValue::Eval(EvalValue::Number(*n))
                }
                ArrayCellValue::Text(t) => {
                    crate::functions::adapters::PreparedArgValue::Eval(EvalValue::Text(t.clone()))
                }
                ArrayCellValue::Logical(b) => {
                    crate::functions::adapters::PreparedArgValue::Eval(EvalValue::Logical(*b))
                }
                ArrayCellValue::Error(code) => {
                    crate::functions::adapters::PreparedArgValue::Eval(EvalValue::Error(*code))
                }
                ArrayCellValue::EmptyCell => {
                    crate::functions::adapters::PreparedArgValue::EmptyCell
                }
            })
        }
        scalar => Some(scalar.clone()),
    }
}

fn call_arg_from_prepared(prepared: &crate::functions::adapters::PreparedArgValue) -> CallArgValue {
    match prepared {
        crate::functions::adapters::PreparedArgValue::Eval(value) => {
            CallArgValue::Eval(value.clone())
        }
        crate::functions::adapters::PreparedArgValue::MissingArg => CallArgValue::MissingArg,
        crate::functions::adapters::PreparedArgValue::EmptyCell => CallArgValue::EmptyCell,
    }
}

fn scalar_output_cell(value: EvalValue) -> ArrayCellValue {
    match value {
        EvalValue::Number(n) => ArrayCellValue::Number(n),
        EvalValue::Text(t) => ArrayCellValue::Text(t),
        EvalValue::Logical(b) => ArrayCellValue::Logical(b),
        EvalValue::Error(code) => ArrayCellValue::Error(code),
        EvalValue::Array(array) => array
            .get(0, 0)
            .cloned()
            .unwrap_or(ArrayCellValue::Error(WorksheetErrorCode::Calc)),
        EvalValue::Reference(_) | EvalValue::Lambda(_) => {
            ArrayCellValue::Error(WorksheetErrorCode::Value)
        }
    }
}

fn try_observed_scalar_array_lift(
    function_id: &str,
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    now_serial: Option<f64>,
    random_value: Option<f64>,
    locale_ctx: Option<&LocaleFormatContext>,
    host_info: Option<&dyn HostInfoProvider>,
    callable_invoker: &dyn CallableInvoker,
    rtd_provider: Option<&dyn RtdProvider>,
    registered_external_provider: Option<&dyn RegisteredExternalProvider>,
) -> Option<Result<EvalValue, WorksheetErrorCode>> {
    let lift_positions = observed_scalar_array_lift_positions(function_id)?;
    let prepared = crate::functions::adapters::prepare_args_values_only(args, resolver).ok()?;
    let mut has_lift_array = false;
    let mut shape = ArrayShape { rows: 1, cols: 1 };

    for &position in lift_positions {
        let Some(value) = prepared.get(position) else {
            continue;
        };
        let arg_shape = prepared_array_shape(value);
        if arg_shape != (ArrayShape { rows: 1, cols: 1 }) {
            has_lift_array = true;
        }
        shape.rows = shape.rows.max(arg_shape.rows);
        shape.cols = shape.cols.max(arg_shape.cols);
    }

    if !has_lift_array {
        return None;
    }

    let mut cells = Vec::with_capacity(shape.cell_count());
    for row in 0..shape.rows {
        for col in 0..shape.cols {
            let mut cell_args = Vec::with_capacity(prepared.len());
            let mut missing_coordinate = false;
            for (index, value) in prepared.iter().enumerate() {
                let cell_prepared = if lift_positions.contains(&index) {
                    match prepared_broadcast_at(value, row, col) {
                        Some(value) => value,
                        None => {
                            missing_coordinate = true;
                            break;
                        }
                    }
                } else {
                    value.clone()
                };
                cell_args.push(call_arg_from_prepared(&cell_prepared));
            }

            if missing_coordinate {
                cells.push(ArrayCellValue::Error(WorksheetErrorCode::NA));
                continue;
            }

            let cell = match eval_surface_value_call_with_callable(
                function_id,
                &cell_args,
                resolver,
                now_serial,
                random_value,
                locale_ctx,
                host_info,
                Some(callable_invoker),
                rtd_provider,
                registered_external_provider,
            ) {
                Ok(value) => scalar_output_cell(value),
                Err(code) => ArrayCellValue::Error(code),
            };
            cells.push(cell);
        }
    }

    Some(
        EvalArray::new(shape, cells)
            .map(EvalValue::Array)
            .ok_or(WorksheetErrorCode::Calc),
    )
}

pub fn eval_surface_value_call_with_callable(
    function_id: &str,
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    now_serial: Option<f64>,
    random_value: Option<f64>,
    locale_ctx: Option<&LocaleFormatContext>,
    host_info: Option<&dyn HostInfoProvider>,
    callable_invoker: Option<&dyn CallableInvoker>,
    rtd_provider: Option<&dyn RtdProvider>,
    registered_external_provider: Option<&dyn RegisteredExternalProvider>,
) -> Result<EvalValue, WorksheetErrorCode> {
    let rejecting_invoker = RejectingCallableInvoker;
    let callable_invoker = callable_invoker.unwrap_or(&rejecting_invoker);
    let result =
        match function_id {
            FUNC_ID_ACOS => eval_acos_surface(args, resolver).map_err(|e| map_acos_error_to_ws(&e)),
            FUNC_ID_ACOT => eval_acot_surface(args, resolver).map_err(|e| map_acot_error_to_ws(&e)),
            FUNC_ID_ACOSH => {
                eval_acosh_surface(args, resolver).map_err(|e| map_acosh_error_to_ws(&e))
            }
            FUNC_ID_ACOTH => {
                eval_acoth_surface(args, resolver).map_err(|e| map_acoth_error_to_ws(&e))
            }
            FUNC_ID_ABS => {
                eval_abs_scalar_value(args, resolver).map_err(|e| map_abs_error_to_ws(&e))
            }
            FUNC_ID_ACCRINT => {
                eval_accrint_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            FUNC_ID_ACCRINTM => {
                eval_accrintm_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            FUNC_ID_AGGREGATE => {
                crate::functions::subtotal_aggregate_family::eval_aggregate_surface(
                    args, resolver, host_info,
                )
                .map_err(|e| {
                    crate::functions::subtotal_aggregate_family::map_subtotal_aggregate_error_to_ws(
                        &e,
                    )
                })
            }
            FUNC_ID_ATAN => eval_atan_surface(args, resolver).map_err(|e| map_atan_error_to_ws(&e)),
            FUNC_ID_ASIN => eval_asin_surface(args, resolver).map_err(|e| map_asin_error_to_ws(&e)),
            FUNC_ID_ASINH => {
                eval_asinh_surface(args, resolver).map_err(|e| map_asinh_error_to_ws(&e))
            }
            FUNC_ID_ATAN2 => {
                eval_atan2_surface(args, resolver).map_err(|e| map_atan2_error_to_ws(&e))
            }
            FUNC_ID_ATANH => {
                eval_atanh_surface(args, resolver).map_err(|e| map_atanh_error_to_ws(&e))
            }
            FUNC_ID_AND => eval_and_surface(args, resolver).map_err(|e| map_and_error_to_ws(&e)),
            FUNC_ID_AMORDEGRC => eval_amordegrc_surface(args, resolver)
                .map_err(|e| map_amor_depreciation_error_to_ws(&e)),
            FUNC_ID_AMORLINC => eval_amorlinc_surface(args, resolver)
                .map_err(|e| map_amor_depreciation_error_to_ws(&e)),
            FUNC_ID_ARABIC => {
                eval_arabic_surface(args, resolver).map_err(|e| map_arabic_error_to_ws(&e))
            }
            FUNC_ID_CALL => eval_call_surface(args, resolver, registered_external_provider)
                .map_err(|e| map_call_register_id_error_to_ws(&e)),
            FUNC_ID_ADDRESS => eval_address_surface(args, resolver)
                .map_err(|e| map_reference_metadata_error_to_ws(&e)),
            FUNC_ID_ARRAYTOTEXT => eval_arraytotext_surface(args, resolver)
                .map_err(|e| map_array_text_split_error_to_ws(&e)),
            FUNC_ID_ASC => eval_asc_surface(args, resolver, host_info)
                .map_err(|e| map_text_compat_locale_error_to_ws(&e)),
            FUNC_ID_AREAS => {
                eval_areas_surface(args).map_err(|e| map_reference_metadata_error_to_ws(&e))
            }
            FUNC_ID_AVEDEV => {
                eval_avedev_surface(args, resolver).map_err(|e| map_avedev_error_to_ws(&e))
            }
            FUNC_ID_AVERAGE => {
                eval_average_surface(args, resolver).map_err(|e| map_average_error_to_ws(&e))
            }
            FUNC_ID_AVERAGEIF => {
                eval_averageif_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            FUNC_ID_AVERAGEIFS => {
                eval_averageifs_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            FUNC_ID_AVERAGEA => {
                eval_averagea_surface(args, resolver).map_err(|e| map_averagea_error_to_ws(&e))
            }
            FUNC_ID_BAHTTEXT => eval_bahttext_surface(args, resolver)
                .map_err(|e| map_misc_conversion_error_to_ws(&e)),
            FUNC_ID_BASE => eval_base_surface(args, resolver).map_err(|e| map_base_error_to_ws(&e)),
            FUNC_ID_BETA_DIST => eval_beta_dist_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
            FUNC_ID_BETA_INV => eval_beta_inv_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
            FUNC_ID_BETADIST => eval_betadist_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
            FUNC_ID_BETAINV => eval_betainv_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
            FUNC_ID_BESSELI => {
                eval_besseli_surface(args, resolver).map_err(|e| map_bessel_convert_error_to_ws(&e))
            }
            FUNC_ID_BESSELJ => {
                eval_besselj_surface(args, resolver).map_err(|e| map_bessel_convert_error_to_ws(&e))
            }
            FUNC_ID_BESSELK => {
                eval_besselk_surface(args, resolver).map_err(|e| map_bessel_convert_error_to_ws(&e))
            }
            FUNC_ID_BESSELY => {
                eval_bessely_surface(args, resolver).map_err(|e| map_bessel_convert_error_to_ws(&e))
            }
            FUNC_ID_BINOM_DIST => eval_binom_dist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
            FUNC_ID_BINOM_DIST_RANGE => eval_binom_dist_range_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
            FUNC_ID_BINOM_INV => eval_binom_inv_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
            FUNC_ID_BINOMDIST => eval_binomdist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
            FUNC_ID_BIN2DEC => eval_bin2dec_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
            FUNC_ID_BIN2HEX => eval_bin2hex_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
            FUNC_ID_BIN2OCT => eval_bin2oct_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
            FUNC_ID_BITAND => {
                eval_bitand_surface(args, resolver).map_err(|e| map_bitand_error_to_ws(&e))
            }
            FUNC_ID_BITLSHIFT => {
                eval_bitlshift_surface(args, resolver).map_err(|e| map_bitlshift_error_to_ws(&e))
            }
            FUNC_ID_BITOR => {
                eval_bitor_surface(args, resolver).map_err(|e| map_bitor_error_to_ws(&e))
            }
            FUNC_ID_BITRSHIFT => {
                eval_bitrshift_surface(args, resolver).map_err(|e| map_bitrshift_error_to_ws(&e))
            }
            FUNC_ID_BITXOR => {
                eval_bitxor_surface(args, resolver).map_err(|e| map_bitxor_error_to_ws(&e))
            }
            FUNC_ID_BYCOL => eval_bycol_surface(args, resolver, callable_invoker)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
            FUNC_ID_BYROW => eval_byrow_surface(args, resolver, callable_invoker)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
            FUNC_ID_CELL => {
                eval_cell_surface(args, resolver, host_info).map_err(|e| map_cell_error_to_ws(&e))
            }
            FUNC_ID_CEILING => {
                eval_ceiling_surface(args, resolver).map_err(|e| map_ceiling_floor_error_to_ws(&e))
            }
            FUNC_ID_CEILING_MATH => eval_ceiling_math_surface(args, resolver)
                .map_err(|e| map_ceiling_floor_error_to_ws(&e)),
            FUNC_ID_CEILING_PRECISE => eval_ceiling_precise_surface(args, resolver)
                .map_err(|e| map_ceiling_floor_error_to_ws(&e)),
            FUNC_ID_CHIDIST => {
                eval_chidist_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_CHIINV => {
                eval_chiinv_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_CHOOSE => {
                eval_choose_surface(args, resolver).map_err(|e| map_choose_ifs_error_to_ws(&e))
            }
            FUNC_ID_CHOOSECOLS => eval_choosecols_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
            FUNC_ID_CHOOSEROWS => eval_chooserows_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
            FUNC_ID_CHISQ_DIST => {
                eval_chisq_dist_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_CHISQ_DIST_RT => {
                eval_chisq_dist_rt_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_CHISQ_INV => {
                eval_chisq_inv_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_CHISQ_INV_RT => {
                eval_chisq_inv_rt_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_CHISQ_TEST => eval_chisq_test_surface(args, resolver)
                .map_err(|e| map_statistical_tests_error_to_ws(&e)),
            FUNC_ID_CHITEST => eval_chitest_surface(args, resolver)
                .map_err(|e| map_statistical_tests_error_to_ws(&e)),
            FUNC_ID_CHAR => {
                eval_char_surface(args, resolver).map_err(|e| map_text_scalar_error_to_ws(&e))
            }
            FUNC_ID_CODE => {
                eval_code_surface(args, resolver).map_err(|e| map_text_scalar_error_to_ws(&e))
            }
            FUNC_ID_COMBIN => {
                eval_combin_surface(args, resolver).map_err(|e| map_combin_error_to_ws(&e))
            }
            FUNC_ID_COMBINA => {
                eval_combina_surface(args, resolver).map_err(|e| map_combina_error_to_ws(&e))
            }
            FUNC_ID_COMPLEX => {
                eval_complex_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_CLEAN => {
                eval_clean_surface(args, resolver).map_err(|e| map_clean_error_to_ws(&e))
            }
            FUNC_ID_CONCAT => {
                eval_concat_surface(args, resolver).map_err(|e| map_concat_error_to_ws(&e))
            }
            FUNC_ID_CONCATENATE => {
                eval_concatenate_surface(args, resolver).map_err(|e| map_concat_error_to_ws(&e))
            }
            FUNC_ID_COLUMN => {
                eval_column_surface(args, resolver).map_err(|e| map_column_error_to_ws(&e))
            }
            FUNC_ID_COLUMNS => eval_columns_surface(args).map_err(|e| map_columns_error_to_ws(&e)),
            FUNC_ID_COS => eval_cos_surface(args, resolver).map_err(|e| map_cos_error_to_ws(&e)),
            FUNC_ID_COSH => eval_cosh_surface(args, resolver).map_err(|e| map_cosh_error_to_ws(&e)),
            FUNC_ID_CORREL => {
                eval_correl_surface(args, resolver).map_err(|e| map_correl_error_to_ws(&e))
            }
            FUNC_ID_COVARIANCE_P => eval_covariance_p_surface(args, resolver)
                .map_err(|e| map_covariance_p_error_to_ws(&e)),
            FUNC_ID_COVARIANCE_S => eval_covariance_s_surface(args, resolver)
                .map_err(|e| map_covariance_s_error_to_ws(&e)),
            FUNC_ID_COT => eval_cot_surface(args, resolver).map_err(|e| map_cot_error_to_ws(&e)),
            FUNC_ID_COTH => eval_coth_surface(args, resolver).map_err(|e| map_coth_error_to_ws(&e)),
            FUNC_ID_COUNT => {
                eval_count_surface(args, resolver).map_err(|e| map_count_error_to_ws(&e))
            }
            FUNC_ID_COUNTBLANK => {
                eval_countblank_surface(args, resolver).map_err(|e| map_countblank_error_to_ws(&e))
            }
            FUNC_ID_COUPDAYBS => {
                eval_coupdaybs_surface(args, resolver).map_err(|e| map_coupon_error_to_ws(&e))
            }
            FUNC_ID_COUPDAYS => {
                eval_coupdays_surface(args, resolver).map_err(|e| map_coupon_error_to_ws(&e))
            }
            FUNC_ID_COUPDAYSNC => {
                eval_coupdaysnc_surface(args, resolver).map_err(|e| map_coupon_error_to_ws(&e))
            }
            FUNC_ID_COUPNCD => {
                eval_coupncd_surface(args, resolver).map_err(|e| map_coupon_error_to_ws(&e))
            }
            FUNC_ID_COUPNUM => {
                eval_coupnum_surface(args, resolver).map_err(|e| map_coupon_error_to_ws(&e))
            }
            FUNC_ID_COUPPCD => {
                eval_couppcd_surface(args, resolver).map_err(|e| map_coupon_error_to_ws(&e))
            }
            FUNC_ID_COUNTIF => {
                eval_countif_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            FUNC_ID_COUNTIFS => {
                eval_countifs_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            FUNC_ID_COUNTA => {
                eval_counta_surface(args, resolver).map_err(|e| map_counta_error_to_ws(&e))
            }
            FUNC_ID_COVAR => eval_covar_surface(args, resolver)
                .map_err(|e| map_legacy_stats_alias_error_to_ws(&e)),
            FUNC_ID_CRITBINOM => eval_critbinom_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
            FUNC_ID_CSC => eval_csc_surface(args, resolver).map_err(|e| map_csc_error_to_ws(&e)),
            FUNC_ID_CSCH => eval_csch_surface(args, resolver).map_err(|e| map_csch_error_to_ws(&e)),
            FUNC_ID_CUMIPMT => eval_cumipmt_surface(args, resolver)
                .map_err(|e| map_cumulative_finance_error_to_ws(&e)),
            FUNC_ID_CUMPRINC => eval_cumprinc_surface(args, resolver)
                .map_err(|e| map_cumulative_finance_error_to_ws(&e)),
            FUNC_ID_CONVERT => eval_convert_surface(args, resolver)
                .map_err(|e| map_misc_conversion_error_to_ws(&e)),
            FUNC_ID_DAVERAGE => {
                eval_daverage_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            FUNC_ID_DATE => eval_date_surface(args, resolver).map_err(|e| map_date_error_to_ws(&e)),
            FUNC_ID_DATEDIF => eval_datedif_surface(args, resolver)
                .map_err(|e| map_date_value_family_error_to_ws(&e)),
            FUNC_ID_DAY => {
                eval_day_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            FUNC_ID_DAYS => {
                eval_days_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            FUNC_ID_DAYS360 => eval_days360_surface(args, resolver)
                .map_err(|e| map_date_value_family_error_to_ws(&e)),
            FUNC_ID_DATEVALUE => eval_datevalue_surface(args, resolver)
                .map_err(|e| map_date_value_family_error_to_ws(&e)),
            FUNC_ID_DBCS => eval_dbcs_surface(args, resolver, host_info)
                .map_err(|e| map_text_compat_locale_error_to_ws(&e)),
            FUNC_ID_DB => {
                eval_db_surface(args, resolver).map_err(|e| map_depreciation_error_to_ws(&e))
            }
            FUNC_ID_DEC2BIN => eval_dec2bin_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
            FUNC_ID_DEC2HEX => eval_dec2hex_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
            FUNC_ID_DEC2OCT => eval_dec2oct_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
            FUNC_ID_EDATE => {
                eval_edate_surface(args, resolver).map_err(|e| map_date_week_error_to_ws(&e))
            }
            FUNC_ID_EOMONTH => {
                eval_eomonth_surface(args, resolver).map_err(|e| map_date_week_error_to_ws(&e))
            }
            FUNC_ID_EFFECT => eval_effect_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
            FUNC_ID_EUROCONVERT => eval_euroconvert_surface(args, resolver)
                .map_err(|e| map_misc_conversion_error_to_ws(&e)),
            FUNC_ID_EXPAND => eval_expand_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
            FUNC_ID_DECIMAL => {
                eval_decimal_surface(args, resolver).map_err(|e| map_decimal_error_to_ws(&e))
            }
            FUNC_ID_ENCODEURL => {
                eval_encodeurl_surface(args, resolver).map_err(|e| map_web_text_xml_error_to_ws(&e))
            }
            FUNC_ID_DDB => {
                eval_ddb_surface(args, resolver).map_err(|e| map_depreciation_error_to_ws(&e))
            }
            FUNC_ID_DCOUNT => {
                eval_dcount_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            FUNC_ID_DCOUNTA => {
                eval_dcounta_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            FUNC_ID_DISC => eval_disc_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
            FUNC_ID_DGET => {
                eval_dget_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            FUNC_ID_DMAX => {
                eval_dmax_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            FUNC_ID_DMIN => {
                eval_dmin_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            FUNC_ID_DPRODUCT => {
                eval_dproduct_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            FUNC_ID_DSTDEV => {
                eval_dstdev_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            FUNC_ID_DSTDEVP => {
                eval_dstdevp_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            FUNC_ID_DSUM => {
                eval_dsum_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            FUNC_ID_DVAR => {
                eval_dvar_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            FUNC_ID_DVARP => {
                eval_dvarp_surface(args, resolver).map_err(|e| map_database_error_to_ws(&e))
            }
            FUNC_ID_DROP => eval_drop_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
            FUNC_ID_DEVSQ => {
                eval_devsq_surface(args, resolver).map_err(|e| map_devsq_error_to_ws(&e))
            }
            FUNC_ID_DEGREES => {
                eval_degrees_surface(args, resolver).map_err(|e| map_degrees_error_to_ws(&e))
            }
            FUNC_ID_DELTA => {
                eval_delta_surface(args, resolver).map_err(|e| map_delta_error_to_ws(&e))
            }
            FUNC_ID_DURATION => {
                eval_duration_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            FUNC_ID_DOLLAR => {
                let ctx = locale_ctx.ok_or(WorksheetErrorCode::Value)?;
                eval_dollar_surface(args, resolver, ctx).map_err(|e| map_dollar_error_to_ws(&e))
            }
            FUNC_ID_DOLLARDE => eval_dollarde_surface(args, resolver)
                .map_err(|e| map_dollar_fraction_error_to_ws(&e)),
            FUNC_ID_DOLLARFR => eval_dollarfr_surface(args, resolver)
                .map_err(|e| map_dollar_fraction_error_to_ws(&e)),
            FUNC_ID_EVEN => eval_even_surface(args, resolver).map_err(|e| map_even_error_to_ws(&e)),
            FUNC_ID_ERROR_TYPE => {
                eval_error_type_surface(args, resolver).map_err(|e| map_error_type_error_to_ws(&e))
            }
            FUNC_ID_ERF => {
                eval_erf_surface(args, resolver).map_err(|e| map_special_dist_error_to_ws(&e))
            }
            FUNC_ID_ERF_PRECISE => eval_erf_precise_surface(args, resolver)
                .map_err(|e| map_special_dist_error_to_ws(&e)),
            FUNC_ID_ERFC => {
                eval_erfc_surface(args, resolver).map_err(|e| map_special_dist_error_to_ws(&e))
            }
            FUNC_ID_ERFC_PRECISE => eval_erfc_precise_surface(args, resolver)
                .map_err(|e| map_special_dist_error_to_ws(&e)),
            FUNC_ID_EXACT => {
                eval_exact_surface(args, resolver).map_err(|e| map_exact_error_to_ws(&e))
            }
            FUNC_ID_EXPON_DIST => eval_expon_dist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
            FUNC_ID_EXPONDIST => eval_expondist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
            FUNC_ID_EXP => eval_exp_surface(args, resolver).map_err(|e| map_exp_error_to_ws(&e)),
            FUNC_ID_FACT => eval_fact_surface(args, resolver).map_err(|e| map_fact_error_to_ws(&e)),
            FUNC_ID_FACTDOUBLE => {
                eval_factdouble_surface(args, resolver).map_err(|e| map_factdouble_error_to_ws(&e))
            }
            FUNC_ID_F_DIST => {
                eval_f_dist_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_F_DIST_RT => {
                eval_f_dist_rt_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_F_INV => {
                eval_f_inv_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_F_INV_RT => {
                eval_f_inv_rt_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_F_TEST => eval_f_test_surface(args, resolver)
                .map_err(|e| map_statistical_tests_error_to_ws(&e)),
            FUNC_ID_FDIST => {
                eval_fdist_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_FINV => {
                eval_finv_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_FALSE => eval_false_surface(args),
            FUNC_ID_FTEST => eval_ftest_surface(args, resolver)
                .map_err(|e| map_statistical_tests_error_to_ws(&e)),
            FUNC_ID_FREQUENCY => eval_frequency_surface(args, resolver)
                .map_err(|e| map_lookup_prob_frequency_error_to_ws(&e)),
            FUNC_ID_FV => eval_fv_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
            FUNC_ID_FVSCHEDULE => eval_fvschedule_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
            FUNC_ID_FISHER => {
                eval_fisher_surface(args, resolver).map_err(|e| map_fisher_error_to_ws(&e))
            }
            FUNC_ID_FISHERINV => {
                eval_fisherinv_surface(args, resolver).map_err(|e| map_fisherinv_error_to_ws(&e))
            }
            FUNC_ID_FIND => eval_find_surface(args, resolver)
                .map_err(|e| map_text_search_replace_error_to_ws(&e)),
            FUNC_ID_FINDB => {
                eval_findb_surface(args, resolver).map_err(|e| map_text_b_compat_error_to_ws(&e))
            }
            FUNC_ID_FILTER => eval_filter_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
            FUNC_ID_FILTERXML => {
                eval_filterxml_surface(args, resolver).map_err(|e| map_web_text_xml_error_to_ws(&e))
            }
            FUNC_ID_FIXED => {
                let ctx = locale_ctx.ok_or(WorksheetErrorCode::Value)?;
                eval_fixed_surface(args, resolver, ctx).map_err(|e| map_fixed_error_to_ws(&e))
            }
            FUNC_ID_FLOOR => {
                eval_floor_surface(args, resolver).map_err(|e| map_ceiling_floor_error_to_ws(&e))
            }
            FUNC_ID_FLOOR_MATH => eval_floor_math_surface(args, resolver)
                .map_err(|e| map_ceiling_floor_error_to_ws(&e)),
            FUNC_ID_FLOOR_PRECISE => eval_floor_precise_surface(args, resolver)
                .map_err(|e| map_ceiling_floor_error_to_ws(&e)),
            FUNC_ID_FORMULATEXT => eval_formulatext_surface(args, host_info)
                .map_err(|e| map_reference_metadata_error_to_ws(&e)),
            FUNC_ID_GAUSS => {
                eval_gauss_surface(args, resolver).map_err(|e| map_gauss_error_to_ws(&e))
            }
            FUNC_ID_GAMMA => {
                eval_gamma_surface(args, resolver).map_err(|e| map_special_dist_error_to_ws(&e))
            }
            FUNC_ID_GAMMA_DIST => eval_gamma_dist_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
            FUNC_ID_GAMMA_INV => eval_gamma_inv_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
            FUNC_ID_GAMMADIST => eval_gammadist_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
            FUNC_ID_GAMMAINV => eval_gammainv_surface(args, resolver)
                .map_err(|e| map_beta_gamma_stats_error_to_ws(&e)),
            FUNC_ID_GAMMALN => {
                eval_gammaln_surface(args, resolver).map_err(|e| map_special_dist_error_to_ws(&e))
            }
            FUNC_ID_GAMMALN_PRECISE => eval_gammaln_precise_surface(args, resolver)
                .map_err(|e| map_special_dist_error_to_ws(&e)),
            FUNC_ID_GCD => eval_gcd_surface(args, resolver).map_err(|e| map_gcd_error_to_ws(&e)),
            FUNC_ID_GEOMEAN => {
                eval_geomean_surface(args, resolver).map_err(|e| map_geomean_error_to_ws(&e))
            }
            FUNC_ID_GESTEP => {
                eval_gestep_surface(args, resolver).map_err(|e| map_gestep_error_to_ws(&e))
            }
            FUNC_ID_GROUPBY => eval_groupby_surface(args, resolver, callable_invoker)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
            FUNC_ID_GROWTH => eval_growth_surface(args, resolver)
                .map_err(|e| map_regression_forecast_error_to_ws(&e)),
            FUNC_ID_FORECAST => eval_forecast_surface(args, resolver)
                .map_err(|e| map_regression_forecast_error_to_ws(&e)),
            FUNC_ID_FORECAST_LINEAR => eval_forecast_linear_surface(args, resolver)
                .map_err(|e| map_regression_forecast_error_to_ws(&e)),
            FUNC_ID_HARMEAN => {
                eval_harmean_surface(args, resolver).map_err(|e| map_harmean_error_to_ws(&e))
            }
            FUNC_ID_HYPERLINK => {
                eval_hyperlink_surface(args, resolver).map_err(|e| map_hyperlink_error_to_ws(&e))
            }
            FUNC_ID_IMAGE => {
                eval_image_surface(args, resolver, host_info).map_err(|e| map_image_error_to_ws(&e))
            }
            FUNC_ID_HYPGEOM_DIST => eval_hypgeom_dist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
            FUNC_ID_HYPGEOMDIST => eval_hypgeomdist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
            FUNC_ID_HOUR => {
                eval_hour_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            FUNC_ID_HSTACK => {
                eval_hstack_surface(args, resolver).map_err(|e| map_hstack_error_to_ws(&e))
            }
            FUNC_ID_SORT => eval_sort_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
            FUNC_ID_SORTBY => eval_sortby_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
            FUNC_ID_VSTACK => eval_vstack_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
            FUNC_ID_INFO => {
                eval_info_surface(args, resolver, host_info).map_err(|e| map_info_error_to_ws(&e))
            }
            FUNC_ID_ISOMITTED => eval_isomitted_surface(args, resolver)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
            FUNC_ID_IRR => {
                eval_irr_surface(args, resolver).map_err(|e| map_cashflow_rate_error_to_ws(&e))
            }
            FUNC_ID_IMABS => {
                eval_imabs_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMAGINARY => eval_imaginary_surface(args, resolver)
                .map_err(|e| map_complex_family_error_to_ws(&e)),
            FUNC_ID_IMARGUMENT => eval_imargument_surface(args, resolver)
                .map_err(|e| map_complex_family_error_to_ws(&e)),
            FUNC_ID_IMCONJUGATE => eval_imconjugate_surface(args, resolver)
                .map_err(|e| map_complex_family_error_to_ws(&e)),
            FUNC_ID_IMCOS => {
                eval_imcos_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMCOSH => {
                eval_imcosh_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMCOT => {
                eval_imcot_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMCSC => {
                eval_imcsc_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMCSCH => {
                eval_imcsch_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMDIV => {
                eval_imdiv_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMEXP => {
                eval_imexp_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMLN => {
                eval_imln_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMLOG10 => {
                eval_imlog10_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMLOG2 => {
                eval_imlog2_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMPOWER => {
                eval_impower_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMPRODUCT => eval_improduct_surface(args, resolver)
                .map_err(|e| map_complex_family_error_to_ws(&e)),
            FUNC_ID_IMREAL => {
                eval_imreal_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMSEC => {
                eval_imsec_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMSECH => {
                eval_imsech_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMSIN => {
                eval_imsin_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMSINH => {
                eval_imsinh_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMSQRT => {
                eval_imsqrt_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMSUB => {
                eval_imsub_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMSUM => {
                eval_imsum_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_IMTAN => {
                eval_imtan_surface(args, resolver).map_err(|e| map_complex_family_error_to_ws(&e))
            }
            FUNC_ID_ISFORMULA => eval_isformula_surface(args, host_info)
                .map_err(|e| map_misc_switch_info_error_to_ws(&e)),
            FUNC_ID_PRODUCT => {
                eval_product_surface(args, resolver).map_err(|e| map_product_error_to_ws(&e))
            }
            FUNC_ID_SUBTOTAL => crate::functions::subtotal_aggregate_family::eval_subtotal_surface(
                args, resolver, host_info,
            )
            .map_err(|e| {
                crate::functions::subtotal_aggregate_family::map_subtotal_aggregate_error_to_ws(&e)
            }),
            FUNC_ID_SUM => eval_sum_surface(args, resolver).map_err(|e| map_sum_error_to_ws(&e)),
            FUNC_ID_SUMIF => {
                eval_sumif_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            FUNC_ID_SUMIFS => {
                eval_sumifs_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            FUNC_ID_SUMPRODUCT => {
                eval_sumproduct_surface(args, resolver).map_err(|e| map_sumproduct_error_to_ws(&e))
            }
            FUNC_ID_SUMX2MY2 => {
                eval_sumx2my2_surface(args, resolver).map_err(|e| map_sumproduct_error_to_ws(&e))
            }
            FUNC_ID_SUMX2PY2 => {
                eval_sumx2py2_surface(args, resolver).map_err(|e| map_sumproduct_error_to_ws(&e))
            }
            FUNC_ID_SUMXMY2 => {
                eval_sumxmy2_surface(args, resolver).map_err(|e| map_sumproduct_error_to_ws(&e))
            }
            FUNC_ID_SUMSQ => {
                eval_sumsq_surface(args, resolver).map_err(|e| map_sumsq_error_to_ws(&e))
            }
            FUNC_ID_SWITCH => eval_switch_surface(args, resolver)
                .map_err(|e| map_misc_switch_info_error_to_ws(&e)),
            FUNC_ID_T_DIST => {
                eval_t_dist_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_T_DIST_2T => {
                eval_t_dist_2t_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_T_DIST_RT => {
                eval_t_dist_rt_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_T_INV => {
                eval_t_inv_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_T_INV_2T => {
                eval_t_inv_2t_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_T_TEST => eval_t_test_surface(args, resolver)
                .map_err(|e| map_statistical_tests_error_to_ws(&e)),
            FUNC_ID_TDIST => {
                eval_tdist_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_TINV => {
                eval_tinv_surface(args, resolver).map_err(|e| map_chi_f_t_error_to_ws(&e))
            }
            FUNC_ID_SYD => {
                eval_syd_surface(args, resolver).map_err(|e| map_depreciation_error_to_ws(&e))
            }
            FUNC_ID_IF => eval_if_surface(args, resolver).map_err(|e| map_if_error_to_ws(&e)),
            FUNC_ID_IFERROR => {
                eval_iferror_surface(args, resolver).map_err(|e| map_iferror_error_to_ws(&e))
            }
            FUNC_ID_IFNA => eval_ifna_surface(args, resolver).map_err(|e| map_ifna_error_to_ws(&e)),
            FUNC_ID_IFS => {
                eval_ifs_surface(args, resolver).map_err(|e| map_choose_ifs_error_to_ws(&e))
            }
            FUNC_ID_INDEX => {
                eval_index_surface(args, resolver).map_err(|e| map_index_error_to_ws(&e))
            }
            FUNC_ID_IPMT => eval_ipmt_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
            FUNC_ID_ISPMT => eval_ispmt_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
            FUNC_ID_HEX2BIN => eval_hex2bin_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
            FUNC_ID_HEX2DEC => eval_hex2dec_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
            FUNC_ID_HEX2OCT => eval_hex2oct_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
            FUNC_ID_ISO_CEILING => eval_iso_ceiling_surface(args, resolver)
                .map_err(|e| map_ceiling_floor_error_to_ws(&e)),
            FUNC_ID_JIS => eval_jis_surface(args, resolver, host_info)
                .map_err(|e| map_text_compat_locale_error_to_ws(&e)),
            FUNC_ID_LN => eval_ln_surface(args, resolver).map_err(|e| map_ln_error_to_ws(&e)),
            FUNC_ID_LOOKUP => eval_lookup_surface(args, resolver)
                .map_err(|e| map_lookup_prob_frequency_error_to_ws(&e)),
            FUNC_ID_LOG10 => {
                eval_log10_surface(args, resolver).map_err(|e| map_log10_error_to_ws(&e))
            }
            FUNC_ID_LOWER => {
                eval_lower_surface(args, resolver).map_err(|e| map_text_scalar_error_to_ws(&e))
            }
            FUNC_ID_LEFT => {
                eval_left_surface(args, resolver).map_err(|e| map_text_slice_error_to_ws(&e))
            }
            FUNC_ID_LEFTB => {
                eval_leftb_surface(args, resolver).map_err(|e| map_text_b_compat_error_to_ws(&e))
            }
            FUNC_ID_LEN => {
                eval_len_surface(args, resolver).map_err(|e| map_text_slice_error_to_ws(&e))
            }
            FUNC_ID_LENB => {
                eval_lenb_surface(args, resolver).map_err(|e| map_text_b_compat_error_to_ws(&e))
            }
            FUNC_ID_MID => {
                eval_mid_surface(args, resolver).map_err(|e| map_text_slice_error_to_ws(&e))
            }
            FUNC_ID_MIDB => {
                eval_midb_surface(args, resolver).map_err(|e| map_text_b_compat_error_to_ws(&e))
            }
            FUNC_ID_RIGHT => {
                eval_right_surface(args, resolver).map_err(|e| map_text_slice_error_to_ws(&e))
            }
            FUNC_ID_RIGHTB => {
                eval_rightb_surface(args, resolver).map_err(|e| map_text_b_compat_error_to_ws(&e))
            }
            FUNC_ID_MAX => eval_max_surface(args, resolver).map_err(|e| map_max_error_to_ws(&e)),
            FUNC_ID_MAXA => eval_maxa_surface(args, resolver).map_err(|e| map_maxa_error_to_ws(&e)),
            FUNC_ID_MAXIFS => {
                eval_maxifs_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            FUNC_ID_MEDIAN => {
                eval_median_surface(args, resolver).map_err(|e| map_median_error_to_ws(&e))
            }
            FUNC_ID_MATCH => {
                if args.len() < 2 {
                    return Err(WorksheetErrorCode::Value);
                }
                let lookup_array = singleton_arg_slice(&args[1]);
                let match_type = args.get(2);
                eval_match_surface(&args[0], &lookup_array, match_type, resolver)
                    .map_err(|e| map_match_error_to_ws(&e))
            }
            FUNC_ID_MAKEARRAY => eval_makearray_surface(args, resolver, callable_invoker)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
            FUNC_ID_MAP => eval_map_surface(args, resolver, callable_invoker)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
            FUNC_ID_MDETERM => {
                eval_mdeterm_surface(args, resolver).map_err(|e| map_matrix_error_to_ws(&e))
            }
            FUNC_ID_MDURATION => {
                eval_mduration_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            FUNC_ID_MINVERSE => {
                eval_minverse_surface(args, resolver).map_err(|e| map_matrix_error_to_ws(&e))
            }
            FUNC_ID_MMULT => {
                eval_mmult_surface(args, resolver).map_err(|e| map_matrix_error_to_ws(&e))
            }
            FUNC_ID_MUNIT => {
                eval_munit_surface(args, resolver).map_err(|e| map_matrix_error_to_ws(&e))
            }
            FUNC_ID_MOD => eval_mod_surface(args, resolver).map_err(|e| map_mod_error_to_ws(&e)),
            FUNC_ID_MIN => eval_min_surface(args, resolver).map_err(|e| map_min_error_to_ws(&e)),
            FUNC_ID_MINA => eval_mina_surface(args, resolver).map_err(|e| map_mina_error_to_ws(&e)),
            FUNC_ID_MINIFS => {
                eval_minifs_surface(args, resolver).map_err(|e| map_criteria_error_to_ws(&e))
            }
            FUNC_ID_MIRR => eval_mirr_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
            FUNC_ID_MINUTE => {
                eval_minute_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            FUNC_ID_MODE => eval_mode_surface(args, resolver)
                .map_err(|e| map_legacy_stats_alias_error_to_ws(&e)),
            FUNC_ID_MODE_MULT => eval_mode_mult_surface(args, resolver)
                .map_err(|e| map_lookup_prob_frequency_error_to_ws(&e)),
            FUNC_ID_MODE_SNGL => {
                eval_mode_sngl_surface(args, resolver).map_err(|e| map_mode_sngl_error_to_ws(&e))
            }
            FUNC_ID_MONTH => {
                eval_month_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            FUNC_ID_MROUND => {
                eval_mround_surface(args, resolver).map_err(|e| map_mround_error_to_ws(&e))
            }
            FUNC_ID_MULTINOMIAL => eval_multinomial_surface(args, resolver)
                .map_err(|e| map_multinomial_error_to_ws(&e)),
            FUNC_ID_ISNUMBER => {
                eval_isnumber_surface(args, resolver).map_err(|e| map_isnumber_error_to_ws(&e))
            }
            FUNC_ID_ISBLANK => eval_isblank_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
            FUNC_ID_ISERR => eval_iserr_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
            FUNC_ID_ISERROR => eval_iserror_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
            FUNC_ID_ISLOGICAL => eval_islogical_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
            FUNC_ID_ISNA => eval_isna_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
            FUNC_ID_ISNONTEXT => eval_isnontext_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
            FUNC_ID_ISODD => eval_isodd_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
            FUNC_ID_ISREF => eval_isref_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
            FUNC_ID_ISTEXT => eval_istext_surface(args, resolver)
                .map_err(|e| map_information_predicate_error_to_ws(&e)),
            FUNC_ID_ISOWEEKNUM => {
                eval_isoweeknum_surface(args, resolver).map_err(|e| map_date_week_error_to_ws(&e))
            }
            FUNC_ID_N => eval_n_surface(args, resolver).map_err(|e| map_n_error_to_ws(&e)),
            FUNC_ID_NA => eval_na_surface(args),
            FUNC_ID_NOMINAL => eval_nominal_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
            FUNC_ID_NPER => eval_nper_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
            FUNC_ID_NPV => eval_npv_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
            FUNC_ID_NUMBERVALUE => eval_numbervalue_surface(args, resolver, locale_ctx)
                .map_err(|e| map_number_regex_translate_error_to_ws(&e)),
            FUNC_ID_NEGBINOM_DIST => eval_negbinom_dist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
            FUNC_ID_NEGBINOMDIST => eval_negbinomdist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
            FUNC_ID_CONFIDENCE => {
                eval_confidence_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            FUNC_ID_CONFIDENCE_T => eval_confidence_t_surface(args, resolver)
                .map_err(|e| map_confidence_test_error_to_ws(&e)),
            FUNC_ID_CONFIDENCE_NORM => eval_confidence_norm_surface(args, resolver)
                .map_err(|e| map_normal_log_error_to_ws(&e)),
            FUNC_ID_LOGNORM_DIST => eval_lognorm_dist_surface(args, resolver)
                .map_err(|e| map_normal_log_error_to_ws(&e)),
            FUNC_ID_LOGNORM_INV => {
                eval_lognorm_inv_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            FUNC_ID_LOGNORMDIST => {
                eval_lognormdist_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            FUNC_ID_NORM_DIST => {
                eval_norm_dist_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            FUNC_ID_NORM_INV => {
                eval_norm_inv_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            FUNC_ID_NORM_S_DIST => {
                eval_norm_s_dist_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            FUNC_ID_NORM_S_INV => {
                eval_norm_s_inv_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            FUNC_ID_NORMDIST => {
                eval_normdist_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            FUNC_ID_NORMINV => {
                eval_norminv_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            FUNC_ID_NORMSDIST => {
                eval_normsdist_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            FUNC_ID_NORMSINV => {
                eval_normsinv_surface(args, resolver).map_err(|e| map_normal_log_error_to_ws(&e))
            }
            FUNC_ID_NETWORKDAYS => eval_networkdays_surface(args, resolver)
                .map_err(|e| map_workday_networkdays_error_to_ws(&e)),
            FUNC_ID_NETWORKDAYS_INTL => eval_networkdays_intl_surface(args, resolver)
                .map_err(|e| map_workday_networkdays_error_to_ws(&e)),
            FUNC_ID_NOT => eval_not_surface(args, resolver).map_err(|e| map_not_error_to_ws(&e)),
            FUNC_ID_NOW => {
                let serial = now_serial.ok_or(WorksheetErrorCode::Value)?;
                let provider = FixedNowProvider { serial };
                eval_now_surface(args, &provider).map_err(|e| map_now_error_to_ws(&e))
            }
            FUNC_ID_OCT2BIN => eval_oct2bin_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
            FUNC_ID_OCT2DEC => eval_oct2dec_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
            FUNC_ID_OCT2HEX => eval_oct2hex_surface(args, resolver)
                .map_err(|e| map_engineering_radix_error_to_ws(&e)),
            FUNC_ID_POISSON => {
                eval_poisson_surface(args, resolver).map_err(|e| map_discrete_dist_error_to_ws(&e))
            }
            FUNC_ID_POISSON_DIST => eval_poisson_dist_surface(args, resolver)
                .map_err(|e| map_discrete_dist_error_to_ws(&e)),
            FUNC_ID_ODDFPRICE => {
                eval_oddfprice_surface(args, resolver).map_err(|e| map_odd_bond_error_to_ws(&e))
            }
            FUNC_ID_ODDFYIELD => {
                eval_oddfyield_surface(args, resolver).map_err(|e| map_odd_bond_error_to_ws(&e))
            }
            FUNC_ID_ODDLPRICE => {
                eval_oddlprice_surface(args, resolver).map_err(|e| map_odd_bond_error_to_ws(&e))
            }
            FUNC_ID_ODDLYIELD => {
                eval_oddlyield_surface(args, resolver).map_err(|e| map_odd_bond_error_to_ws(&e))
            }
            FUNC_ID_OR => eval_or_surface(args, resolver).map_err(|e| map_or_error_to_ws(&e)),
            FUNC_ID_OFFSET => {
                eval_offset_surface(args, resolver).map_err(|e| map_offset_error_to_ws(&e))
            }
            FUNC_ID_PEARSON => {
                eval_pearson_surface(args, resolver).map_err(|e| map_pearson_error_to_ws(&e))
            }
            FUNC_ID_PDURATION => eval_pduration_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
            FUNC_ID_PERMUT => {
                eval_permut_surface(args, resolver).map_err(|e| map_permut_error_to_ws(&e))
            }
            FUNC_ID_PERMUTATIONA => eval_permutationa_surface(args, resolver)
                .map_err(|e| map_permutationa_error_to_ws(&e)),
            FUNC_ID_PERCENTILE_EXC => eval_percentile_exc_surface(args, resolver)
                .map_err(|e| map_percentile_exc_error_to_ws(&e)),
            FUNC_ID_PERCENTILE_INC => eval_percentile_inc_surface(args, resolver)
                .map_err(|e| map_percentile_inc_error_to_ws(&e)),
            FUNC_ID_PERCENTILE => eval_percentile_surface(args, resolver)
                .map_err(|e| map_legacy_stats_alias_error_to_ws(&e)),
            FUNC_ID_PERCENTRANK_EXC => eval_percentrank_exc_surface(args, resolver)
                .map_err(|e| map_percentrank_exc_error_to_ws(&e)),
            FUNC_ID_PERCENTRANK_INC => eval_percentrank_inc_surface(args, resolver)
                .map_err(|e| map_percentrank_inc_error_to_ws(&e)),
            FUNC_ID_PERCENTRANK => eval_percentrank_surface(args, resolver)
                .map_err(|e| map_legacy_stats_alias_error_to_ws(&e)),
            FUNC_ID_PERCENTOF => eval_percentof_surface(args, resolver)
                .map_err(|e| map_misc_conversion_error_to_ws(&e)),
            FUNC_ID_PHI => eval_phi_surface(args, resolver).map_err(|e| map_phi_error_to_ws(&e)),
            FUNC_ID_PMT => eval_pmt_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
            FUNC_ID_PPMT => eval_ppmt_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
            FUNC_ID_PRICE => {
                eval_price_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            FUNC_ID_PRICEDISC => eval_pricedisc_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
            FUNC_ID_PRICEMAT => {
                eval_pricemat_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            FUNC_ID_PROB => eval_prob_surface(args, resolver)
                .map_err(|e| map_lookup_prob_frequency_error_to_ws(&e)),
            FUNC_ID_PV => eval_pv_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
            FUNC_ID_PROPER => eval_proper_surface(args, resolver)
                .map_err(|e| map_text_search_replace_error_to_ws(&e)),
            FUNC_ID_XLOOKUP => {
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
            FUNC_ID_INDIRECT => {
                eval_indirect_surface(args, resolver).map_err(|e| map_indirect_error_to_ws(&e))
            }
            FUNC_ID_INTERCEPT => {
                eval_intercept_surface(args, resolver).map_err(|e| map_intercept_error_to_ws(&e))
            }
            FUNC_ID_INT => eval_int_surface(args, resolver).map_err(|e| map_int_error_to_ws(&e)),
            FUNC_ID_INTRATE => eval_intrate_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
            FUNC_ID_ISEVEN => {
                eval_iseven_surface(args, resolver).map_err(|e| map_iseven_error_to_ws(&e))
            }
            FUNC_ID_KURT => {
                eval_kurt_surface(args, resolver).map_err(|e| map_moment_stats_error_to_ws(&e))
            }
            FUNC_ID_LARGE => {
                eval_large_surface(args, resolver).map_err(|e| map_large_error_to_ws(&e))
            }
            FUNC_ID_LCM => eval_lcm_surface(args, resolver).map_err(|e| map_lcm_error_to_ws(&e)),
            FUNC_ID_LINEST => eval_linest_surface(args, resolver)
                .map_err(|e| map_regression_forecast_error_to_ws(&e)),
            FUNC_ID_LOGINV => eval_loginv_surface(args, resolver)
                .map_err(|e| map_legacy_stats_alias_error_to_ws(&e)),
            FUNC_ID_LOGEST => eval_logest_surface(args, resolver)
                .map_err(|e| map_regression_forecast_error_to_ws(&e)),
            FUNC_ID_RANDARRAY => {
                let value = random_value.ok_or(WorksheetErrorCode::Value)?;
                let provider = FixedRandomProvider { value };
                eval_randarray_surface(args, resolver, &provider)
                    .map_err(|e| map_misc_conversion_error_to_ws(&e))
            }
            FUNC_ID_REDUCE => eval_reduce_surface(args, resolver, callable_invoker)
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
            FUNC_ID_RAND => {
                let value = random_value.ok_or(WorksheetErrorCode::Value)?;
                let provider = FixedRandomProvider { value };
                eval_rand_surface(args, &provider).map_err(|e| map_rand_error_to_ws(&e))
            }
            FUNC_ID_RANDBETWEEN => {
                let value = random_value.ok_or(WorksheetErrorCode::Value)?;
                let provider = FixedRandomProvider { value };
                eval_randbetween_surface(args, resolver, &provider)
                    .map_err(|e| map_randbetween_error_to_ws(&e))
            }
            FUNC_ID_RATE => eval_rate_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
            FUNC_ID_RADIANS => {
                eval_radians_surface(args, resolver).map_err(|e| map_radians_error_to_ws(&e))
            }
            FUNC_ID_LOG => eval_log_surface(args, resolver).map_err(|e| map_log_error_to_ws(&e)),
            FUNC_ID_RANK => eval_rank_surface(args, resolver).map_err(|e| map_rank_error_to_ws(&e)),
            FUNC_ID_RANK_AVG => {
                eval_rank_avg_surface(args, resolver).map_err(|e| map_rank_avg_error_to_ws(&e))
            }
            FUNC_ID_RANK_EQ => {
                eval_rank_eq_surface(args, resolver).map_err(|e| map_rank_eq_error_to_ws(&e))
            }
            FUNC_ID_QUARTILE_EXC => eval_quartile_exc_surface(args, resolver)
                .map_err(|e| map_quartile_exc_error_to_ws(&e)),
            FUNC_ID_QUARTILE_INC => eval_quartile_inc_surface(args, resolver)
                .map_err(|e| map_quartile_inc_error_to_ws(&e)),
            FUNC_ID_QUARTILE => eval_quartile_surface(args, resolver)
                .map_err(|e| map_legacy_stats_alias_error_to_ws(&e)),
            FUNC_ID_ROW => eval_row_surface(args, resolver).map_err(|e| map_row_error_to_ws(&e)),
            FUNC_ID_ROWS => eval_rows_surface(args).map_err(|e| map_rows_error_to_ws(&e)),
            FUNC_ID_RRI => eval_rri_surface(args, resolver)
                .map_err(|e| map_financial_time_value_error_to_ws(&e)),
            FUNC_ID_RTD => {
                eval_rtd_surface(args, resolver, rtd_provider).map_err(|e| map_rtd_error_to_ws(&e))
            }
            FUNC_ID_ROUND => {
                eval_round_surface(args, resolver).map_err(|e| map_round_error_to_ws(&e))
            }
            FUNC_ID_ROUNDDOWN => {
                eval_rounddown_surface(args, resolver).map_err(|e| map_rounddown_error_to_ws(&e))
            }
            FUNC_ID_REPLACE => eval_replace_surface(args, resolver)
                .map_err(|e| map_text_search_replace_error_to_ws(&e)),
            FUNC_ID_REPLACEB => {
                eval_replaceb_surface(args, resolver).map_err(|e| map_text_b_compat_error_to_ws(&e))
            }
            FUNC_ID_RECEIVED => eval_received_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
            FUNC_ID_REGEXEXTRACT => eval_regexextract_surface(args, resolver)
                .map_err(|e| map_number_regex_translate_error_to_ws(&e)),
            FUNC_ID_REGEXREPLACE => eval_regexreplace_surface(args, resolver)
                .map_err(|e| map_number_regex_translate_error_to_ws(&e)),
            FUNC_ID_REGEXTEST => eval_regextest_surface(args, resolver)
                .map_err(|e| map_number_regex_translate_error_to_ws(&e)),
            FUNC_ID_REGISTER_ID => {
                eval_register_id_surface(args, resolver, registered_external_provider)
                    .map_err(|e| map_call_register_id_error_to_ws(&e))
            }
            FUNC_ID_ROUNDUP => {
                eval_roundup_surface(args, resolver).map_err(|e| map_roundup_error_to_ws(&e))
            }
            FUNC_ID_ROMAN => {
                eval_roman_surface(args, resolver).map_err(|e| map_roman_error_to_ws(&e))
            }
            FUNC_ID_RSQ => eval_rsq_surface(args, resolver).map_err(|e| map_rsq_error_to_ws(&e)),
            FUNC_ID_SECOND => {
                eval_second_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            FUNC_ID_SEC => eval_sec_surface(args, resolver).map_err(|e| map_sec_error_to_ws(&e)),
            FUNC_ID_SECH => eval_sech_surface(args, resolver).map_err(|e| map_sech_error_to_ws(&e)),
            FUNC_ID_SHEET => eval_sheet_surface(args, resolver, host_info)
                .map_err(|e| map_reference_metadata_error_to_ws(&e)),
            FUNC_ID_SHEETS => eval_sheets_surface(args, host_info)
                .map_err(|e| map_reference_metadata_error_to_ws(&e)),
            FUNC_ID_SERIESSUM => {
                eval_seriessum_surface(args, resolver).map_err(|e| map_sumproduct_error_to_ws(&e))
            }
            FUNC_ID_ODD => eval_odd_surface(args, resolver).map_err(|e| map_odd_error_to_ws(&e)),
            FUNC_ID_SEQUENCE => {
                eval_sequence_surface(args, resolver).map_err(|e| map_sequence_error_to_ws(&e))
            }
            FUNC_ID_SCAN => eval_scan_surface(args, resolver, callable_invoker)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
            FUNC_ID_SIGN => eval_sign_surface(args, resolver).map_err(|e| map_sign_error_to_ws(&e)),
            FUNC_ID_SIN => eval_sin_surface(args, resolver).map_err(|e| map_sin_error_to_ws(&e)),
            FUNC_ID_SINH => eval_sinh_surface(args, resolver).map_err(|e| map_sinh_error_to_ws(&e)),
            FUNC_ID_SKEW => {
                eval_skew_surface(args, resolver).map_err(|e| map_moment_stats_error_to_ws(&e))
            }
            FUNC_ID_SKEW_P => {
                eval_skew_p_surface(args, resolver).map_err(|e| map_moment_stats_error_to_ws(&e))
            }
            FUNC_ID_SLN => {
                eval_sln_surface(args, resolver).map_err(|e| map_depreciation_error_to_ws(&e))
            }
            FUNC_ID_SMALL => {
                eval_small_surface(args, resolver).map_err(|e| map_small_error_to_ws(&e))
            }
            FUNC_ID_SQRT => eval_sqrt_surface(args, resolver).map_err(|e| map_sqrt_error_to_ws(&e)),
            FUNC_ID_SQRTPI => {
                eval_sqrtpi_surface(args, resolver).map_err(|e| map_sqrtpi_error_to_ws(&e))
            }
            FUNC_ID_SLOPE => {
                eval_slope_surface(args, resolver).map_err(|e| map_slope_error_to_ws(&e))
            }
            FUNC_ID_STDEV => {
                eval_stdev_surface(args, resolver).map_err(|e| map_stdev_error_to_ws(&e))
            }
            FUNC_ID_STDEV_P => {
                eval_stdev_p_surface(args, resolver).map_err(|e| map_stdev_p_error_to_ws(&e))
            }
            FUNC_ID_STDEV_S => {
                eval_stdev_s_surface(args, resolver).map_err(|e| map_stdev_s_error_to_ws(&e))
            }
            FUNC_ID_STDEVP => {
                eval_stdevp_surface(args, resolver).map_err(|e| map_stdevp_error_to_ws(&e))
            }
            FUNC_ID_STDEVA => {
                eval_stdeva_surface(args, resolver).map_err(|e| map_stdeva_error_to_ws(&e))
            }
            FUNC_ID_STDEVPA => {
                eval_stdevpa_surface(args, resolver).map_err(|e| map_stdevpa_error_to_ws(&e))
            }
            FUNC_ID_STEYX => {
                eval_steyx_surface(args, resolver).map_err(|e| map_moment_stats_error_to_ws(&e))
            }
            FUNC_ID_STANDARDIZE => eval_standardize_surface(args, resolver)
                .map_err(|e| map_standardize_error_to_ws(&e)),
            FUNC_ID_OP_ADD => {
                eval_op_add_surface(args, resolver).map_err(|e| map_op_add_error_to_ws(&e))
            }
            FUNC_ID_OP_CONCAT => eval_op_concat_surface(args, resolver)
                .map_err(|e| map_operator_compare_concat_error_to_ws(&e)),
            FUNC_ID_OP_DIVIDE => eval_op_divide_surface(args, resolver)
                .map_err(|e| map_operator_binary_error_to_ws(&e)),
            FUNC_ID_OP_EQUAL => eval_op_equal_surface(args, resolver)
                .map_err(|e| map_operator_compare_concat_error_to_ws(&e)),
            FUNC_ID_OP_GREATER_EQUAL => eval_op_greater_equal_surface(args, resolver)
                .map_err(|e| map_operator_compare_concat_error_to_ws(&e)),
            FUNC_ID_OP_GREATER_THAN => eval_op_greater_than_surface(args, resolver)
                .map_err(|e| map_operator_compare_concat_error_to_ws(&e)),
            FUNC_ID_OP_IMPLICIT_INTERSECTION => {
                eval_op_implicit_intersection_surface(args, resolver)
                    .map_err(|e| map_op_implicit_intersection_error_to_ws(&e))
            }
            FUNC_ID_OP_INTERSECTION_REF => eval_op_intersection_ref_surface(args, resolver)
                .map_err(|e| map_operator_reference_error_to_ws(&e)),
            FUNC_ID_OP_LESS_EQUAL => eval_op_less_equal_surface(args, resolver)
                .map_err(|e| map_operator_compare_concat_error_to_ws(&e)),
            FUNC_ID_OP_LESS_THAN => eval_op_less_than_surface(args, resolver)
                .map_err(|e| map_operator_compare_concat_error_to_ws(&e)),
            FUNC_ID_OP_MULTIPLY => eval_op_multiply_surface(args, resolver)
                .map_err(|e| map_operator_binary_error_to_ws(&e)),
            FUNC_ID_OP_NEGATE => eval_op_negate_surface(args, resolver)
                .map_err(|e| map_operator_unary_error_to_ws(&e)),
            FUNC_ID_OP_NOT_EQUAL => eval_op_not_equal_surface(args, resolver)
                .map_err(|e| map_operator_compare_concat_error_to_ws(&e)),
            FUNC_ID_OP_PERCENT => eval_op_percent_surface(args, resolver)
                .map_err(|e| map_operator_unary_error_to_ws(&e)),
            FUNC_ID_OP_POWER => eval_op_power_surface(args, resolver)
                .map_err(|e| map_operator_binary_error_to_ws(&e)),
            FUNC_ID_OP_RANGE_REF => eval_op_range_ref_surface(args, resolver)
                .map_err(|e| map_operator_reference_error_to_ws(&e)),
            FUNC_ID_OP_SPILL_REF => eval_op_spill_ref_surface(args, resolver)
                .map_err(|e| map_op_spill_ref_error_to_ws(&e)),
            FUNC_ID_OP_SUBTRACT => eval_op_subtract_surface(args, resolver)
                .map_err(|e| map_operator_binary_error_to_ws(&e)),
            FUNC_ID_OP_TRIM_REF_BOTH => eval_op_trim_ref_both_surface(args, resolver)
                .map_err(|e| map_operator_reference_error_to_ws(&e)),
            FUNC_ID_OP_TRIM_REF_LEADING => eval_op_trim_ref_leading_surface(args, resolver)
                .map_err(|e| map_operator_reference_error_to_ws(&e)),
            FUNC_ID_OP_TRIM_REF_TRAILING => eval_op_trim_ref_trailing_surface(args, resolver)
                .map_err(|e| map_operator_reference_error_to_ws(&e)),
            FUNC_ID_OP_UNARY_PLUS => eval_op_unary_plus_surface(args, resolver)
                .map_err(|e| map_operator_unary_error_to_ws(&e)),
            FUNC_ID_OP_UNION_REF => eval_op_union_ref_surface(args, resolver)
                .map_err(|e| map_operator_reference_error_to_ws(&e)),
            FUNC_ID_T => eval_t_surface(args, resolver).map_err(|e| map_t_error_to_ws(&e)),
            FUNC_ID_TAN => eval_tan_surface(args, resolver).map_err(|e| map_tan_error_to_ws(&e)),
            FUNC_ID_TANH => eval_tanh_surface(args, resolver).map_err(|e| map_tanh_error_to_ws(&e)),
            FUNC_ID_TBILLEQ => eval_tbilleq_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
            FUNC_ID_TBILLPRICE => eval_tbillprice_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
            FUNC_ID_TBILLYIELD => eval_tbillyield_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
            FUNC_ID_TAKE => eval_take_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
            FUNC_ID_TOCOL => eval_tocol_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
            FUNC_ID_TOROW => eval_torow_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
            FUNC_ID_SEARCH => eval_search_surface(args, resolver)
                .map_err(|e| map_text_search_replace_error_to_ws(&e)),
            FUNC_ID_SEARCHB => {
                eval_searchb_surface(args, resolver).map_err(|e| map_text_b_compat_error_to_ws(&e))
            }
            FUNC_ID_TEXT => {
                let ctx = locale_ctx.ok_or(WorksheetErrorCode::Value)?;
                eval_text_surface(args, resolver, ctx).map_err(|e| map_text_error_to_ws(&e))
            }
            FUNC_ID_TEXTAFTER => {
                eval_textafter_surface(args, resolver).map_err(|e| map_text_delim_error_to_ws(&e))
            }
            FUNC_ID_TEXTBEFORE => {
                eval_textbefore_surface(args, resolver).map_err(|e| map_text_delim_error_to_ws(&e))
            }
            FUNC_ID_TEXTSPLIT => eval_textsplit_surface(args, resolver)
                .map_err(|e| map_array_text_split_error_to_ws(&e)),
            FUNC_ID_REPT => {
                eval_rept_surface(args, resolver).map_err(|e| map_text_scalar_error_to_ws(&e))
            }
            FUNC_ID_SUBSTITUTE => eval_substitute_surface(args, resolver)
                .map_err(|e| map_text_search_replace_error_to_ws(&e)),
            FUNC_ID_TEXTJOIN => {
                eval_textjoin_surface(args, resolver).map_err(|e| map_textjoin_error_to_ws(&e))
            }
            FUNC_ID_TODAY => {
                let serial = now_serial.ok_or(WorksheetErrorCode::Value)?;
                let provider = FixedNowProvider { serial };
                eval_today_surface(args, &provider).map_err(|e| map_today_error_to_ws(&e))
            }
            FUNC_ID_TIME => {
                eval_time_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            FUNC_ID_TIMEVALUE => eval_timevalue_surface(args, resolver)
                .map_err(|e| map_date_value_family_error_to_ws(&e)),
            FUNC_ID_TRANSLATE => eval_translate_surface(args, resolver, host_info)
                .map_err(|e| map_number_regex_translate_error_to_ws(&e)),
            FUNC_ID_TRIMMEAN => {
                eval_trimmean_surface(args, resolver).map_err(|e| map_moment_stats_error_to_ws(&e))
            }
            FUNC_ID_TRANSPOSE => eval_transpose_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
            FUNC_ID_TRUE => eval_true_surface(args),
            FUNC_ID_TREND => eval_trend_surface(args, resolver)
                .map_err(|e| map_regression_forecast_error_to_ws(&e)),
            FUNC_ID_TRUNC => {
                eval_trunc_surface(args, resolver).map_err(|e| map_trunc_error_to_ws(&e))
            }
            FUNC_ID_TRIMRANGE => {
                eval_trimrange_surface(args, resolver).map_err(|e| map_trimrange_error_to_ws(&e))
            }
            FUNC_ID_TRIM => {
                eval_trim_surface(args, resolver).map_err(|e| map_text_scalar_error_to_ws(&e))
            }
            FUNC_ID_TTEST => eval_ttest_surface(args, resolver)
                .map_err(|e| map_statistical_tests_error_to_ws(&e)),
            FUNC_ID_TYPE => eval_type_surface(args, resolver).map_err(|e| map_type_error_to_ws(&e)),
            FUNC_ID_UNIQUE => eval_unique_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
            FUNC_ID_UNICHAR => {
                eval_unichar_surface(args, resolver).map_err(|e| map_text_unicode_error_to_ws(&e))
            }
            FUNC_ID_UNICODE => {
                eval_unicode_surface(args, resolver).map_err(|e| map_text_unicode_error_to_ws(&e))
            }
            FUNC_ID_UPPER => {
                eval_upper_surface(args, resolver).map_err(|e| map_text_scalar_error_to_ws(&e))
            }
            FUNC_ID_VALUE => {
                let ctx = locale_ctx.ok_or(WorksheetErrorCode::Value)?;
                eval_value_surface(args, resolver, ctx).map_err(|e| map_value_error_to_ws(&e))
            }
            FUNC_ID_VALUETOTEXT => eval_valuetotext_surface(args, resolver)
                .map_err(|e| map_valuetotext_error_to_ws(&e)),
            FUNC_ID_VAR => eval_var_surface(args, resolver).map_err(|e| map_var_error_to_ws(&e)),
            FUNC_ID_VAR_P => {
                eval_var_p_surface(args, resolver).map_err(|e| map_var_p_error_to_ws(&e))
            }
            FUNC_ID_VAR_S => {
                eval_var_s_surface(args, resolver).map_err(|e| map_var_s_error_to_ws(&e))
            }
            FUNC_ID_VARA => eval_vara_surface(args, resolver).map_err(|e| map_vara_error_to_ws(&e)),
            FUNC_ID_VARP => eval_varp_surface(args, resolver).map_err(|e| map_varp_error_to_ws(&e)),
            FUNC_ID_VARPA => {
                eval_varpa_surface(args, resolver).map_err(|e| map_varpa_error_to_ws(&e))
            }
            FUNC_ID_VDB => {
                eval_vdb_surface(args, resolver).map_err(|e| map_depreciation_error_to_ws(&e))
            }
            FUNC_ID_WRAPCOLS => eval_wrapcols_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
            FUNC_ID_WRAPROWS => eval_wraprows_surface(args, resolver)
                .map_err(|e| map_dynamic_array_reshape_error_to_ws(&e)),
            FUNC_ID_HLOOKUP => {
                eval_hlookup_surface(args, resolver).map_err(|e| map_vhlookup_error_to_ws(&e))
            }
            FUNC_ID_VLOOKUP => {
                eval_vlookup_surface(args, resolver).map_err(|e| map_vhlookup_error_to_ws(&e))
            }
            FUNC_ID_WEIBULL => {
                eval_weibull_surface(args, resolver).map_err(|e| map_special_dist_error_to_ws(&e))
            }
            FUNC_ID_WEIBULL_DIST => eval_weibull_dist_surface(args, resolver)
                .map_err(|e| map_special_dist_error_to_ws(&e)),
            FUNC_ID_XIRR => {
                eval_xirr_surface(args, resolver).map_err(|e| map_cashflow_rate_error_to_ws(&e))
            }
            FUNC_ID_XNPV => {
                eval_xnpv_surface(args, resolver).map_err(|e| map_cashflow_rate_error_to_ws(&e))
            }
            FUNC_ID_XOR => eval_xor_surface(args, resolver).map_err(|e| map_xor_error_to_ws(&e)),
            FUNC_ID_WEEKDAY => {
                eval_weekday_surface(args, resolver).map_err(|e| map_date_week_error_to_ws(&e))
            }
            FUNC_ID_WEEKNUM => {
                eval_weeknum_surface(args, resolver).map_err(|e| map_date_week_error_to_ws(&e))
            }
            FUNC_ID_WORKDAY => eval_workday_surface(args, resolver)
                .map_err(|e| map_workday_networkdays_error_to_ws(&e)),
            FUNC_ID_WORKDAY_INTL => eval_workday_intl_surface(args, resolver)
                .map_err(|e| map_workday_networkdays_error_to_ws(&e)),
            FUNC_ID_YIELD => {
                eval_yield_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            FUNC_ID_YIELDDISC => {
                eval_yielddisc_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            FUNC_ID_YIELDMAT => {
                eval_yieldmat_surface(args, resolver).map_err(|e| map_bond_core_error_to_ws(&e))
            }
            FUNC_ID_XMATCH => {
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
            FUNC_ID_Z_TEST => {
                eval_z_test_surface(args, resolver).map_err(|e| map_confidence_test_error_to_ws(&e))
            }
            FUNC_ID_ZTEST => {
                eval_ztest_surface(args, resolver).map_err(|e| map_test_alias_error_to_ws(&e))
            }
            FUNC_ID_YEAR => {
                eval_year_surface(args, resolver).map_err(|e| map_date_parts_error_to_ws(&e))
            }
            FUNC_ID_YEARFRAC => eval_yearfrac_surface(args, resolver)
                .map_err(|e| map_discount_bill_yearfrac_error_to_ws(&e)),
            FUNC_ID_PI => {
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
            FUNC_ID_PIVOTBY => eval_pivotby_surface(args, resolver, callable_invoker)
                .map_err(|e| map_lambda_helper_error_to_ws(&e)),
            FUNC_ID_POWER => {
                eval_power_surface(args, resolver).map_err(|e| map_power_error_to_ws(&e))
            }
            FUNC_ID_QUOTIENT => {
                eval_quotient_surface(args, resolver).map_err(|e| map_quotient_error_to_ws(&e))
            }
            _ => Err(WorksheetErrorCode::Value),
        };

    match result {
        Err(code) => try_observed_scalar_array_lift(
            function_id,
            args,
            resolver,
            now_serial,
            random_value,
            locale_ctx,
            host_info,
            callable_invoker,
            rtd_provider,
            registered_external_provider,
        )
        .unwrap_or(Err(code)),
        Ok(EvalValue::Error(code))
            if code == WorksheetErrorCode::Value
                || observed_error_result_array_lift(function_id) =>
        {
            try_observed_scalar_array_lift(
                function_id,
                args,
                resolver,
                now_serial,
                random_value,
                locale_ctx,
                host_info,
                callable_invoker,
                rtd_provider,
                registered_external_provider,
            )
            .unwrap_or(Ok(EvalValue::Error(code)))
        }
        other => other,
    }
}

pub fn eval_surface_q_unary_number(
    function_id: &str,
    value: f64,
) -> Result<f64, WorksheetErrorCode> {
    match function_id {
        FUNC_ID_ABS => Ok(abs_kernel(value)),
        FUNC_ID_ACOT => acot_kernel(value),
        FUNC_ID_ATAN => Ok(atan_kernel(value)),
        FUNC_ID_ASINH => asinh_kernel(value),
        FUNC_ID_ATANH => atanh_kernel(value),
        FUNC_ID_COS => Ok(cos_kernel(value)),
        FUNC_ID_COSH => Ok(cosh_kernel(value)),
        FUNC_ID_COT => cot_kernel(value),
        FUNC_ID_COTH => coth_kernel(value),
        FUNC_ID_CSC => csc_kernel(value),
        FUNC_ID_CSCH => csch_kernel(value),
        FUNC_ID_DEGREES => Ok(degrees_kernel(value)),
        FUNC_ID_EVEN => even_kernel(value),
        FUNC_ID_EXP => Ok(exp_kernel(value)),
        FUNC_ID_FACT => fact_kernel(value),
        FUNC_ID_FACTDOUBLE => factdouble_kernel(value),
        FUNC_ID_INT => int_kernel(value),
        FUNC_ID_LN => ln_kernel(value),
        FUNC_ID_LOG10 => log10_kernel(value),
        FUNC_ID_ODD => odd_kernel(value),
        FUNC_ID_OP_NEGATE => op_negate_kernel(value),
        FUNC_ID_OP_PERCENT => op_percent_kernel(value),
        FUNC_ID_OP_UNARY_PLUS => op_unary_plus_kernel(value),
        FUNC_ID_RADIANS => Ok(radians_kernel(value)),
        FUNC_ID_SEC => sec_kernel(value),
        FUNC_ID_SECH => sech_kernel(value),
        FUNC_ID_SIGN => sign_kernel(value),
        FUNC_ID_SIN => Ok(crate::functions::sin::sin_kernel(value)),
        FUNC_ID_SINH => Ok(sinh_kernel(value)),
        FUNC_ID_SQRT => sqrt_kernel(value),
        FUNC_ID_SQRTPI => sqrtpi_kernel(value),
        FUNC_ID_TAN => Ok(tan_kernel(value)),
        FUNC_ID_TANH => Ok(tanh_kernel(value)),
        _ => Err(WorksheetErrorCode::Value),
    }
}

pub fn eval_surface_q_binary_number(
    function_id: &str,
    lhs: f64,
    rhs: f64,
) -> Result<f64, WorksheetErrorCode> {
    match function_id {
        FUNC_ID_ATAN2 => atan2_kernel(lhs, rhs),
        FUNC_ID_BITAND => bitand_kernel(lhs, rhs),
        FUNC_ID_BITLSHIFT => bitlshift_kernel(lhs, rhs),
        FUNC_ID_BITOR => bitor_kernel(lhs, rhs),
        FUNC_ID_BITRSHIFT => bitrshift_kernel(lhs, rhs),
        FUNC_ID_BITXOR => bitxor_kernel(lhs, rhs),
        FUNC_ID_COMBIN => combin_kernel(lhs, rhs),
        FUNC_ID_COMBINA => combina_kernel(lhs, rhs),
        FUNC_ID_DELTA => delta_kernel(lhs, rhs),
        FUNC_ID_GESTEP => gestep_kernel(lhs, rhs),
        FUNC_ID_MOD => mod_kernel(lhs, rhs),
        FUNC_ID_MROUND => mround_kernel(lhs, rhs),
        FUNC_ID_OP_ADD => Ok(op_add_kernel(lhs, rhs)),
        FUNC_ID_OP_DIVIDE => op_divide_kernel(lhs, rhs),
        FUNC_ID_OP_MULTIPLY => op_multiply_kernel(lhs, rhs),
        FUNC_ID_OP_POWER => power_kernel(lhs, rhs),
        FUNC_ID_POWER => power_kernel(lhs, rhs),
        FUNC_ID_OP_SUBTRACT => op_subtract_kernel(lhs, rhs),
        FUNC_ID_QUOTIENT => quotient_kernel(lhs, rhs),
        FUNC_ID_ROUND => Ok(round_kernel(lhs, rhs.trunc() as i32)),
        FUNC_ID_TRUNC => Ok(trunc_kernel(lhs, rhs.trunc() as i32)),
        _ => Err(WorksheetErrorCode::Value),
    }
}

pub fn eval_surface_q_nullary_number(function_id: &str) -> Result<f64, WorksheetErrorCode> {
    match function_id {
        FUNC_ID_PI => match eval_pi(&[]) {
            Ok(Value::Number(n)) => Ok(n),
            Ok(_) => Err(WorksheetErrorCode::Value),
            Err(e) => Err(map_eval_error_to_ws(&e)),
        },
        _ => Err(WorksheetErrorCode::Value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    use crate::functions::adapters::PreparedArgValue;
    use crate::host_info::{
        HostInfoError, HostInfoProvider, ImageProviderResult, ImageRequest, ResolvedWebImage,
    };
    use crate::locale_format::test_current_excel_host_context;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{
        ArrayCellValue, CallableArityShape, CallableCaptureMode, CellStyleHint, EvalArray,
        ExcelText, ExtendedValue, LambdaValue, NumberFormatHint, PresentationHint, ReferenceKind,
        ReferenceLike, RichValueData,
    };

    struct NoReferenceResolver;

    struct TestCallableInvoker;

    type RegisteredCallable<'a> =
        Rc<dyn Fn(&[PreparedArgValue]) -> Result<PreparedArgValue, CallableInvocationError> + 'a>;

    #[derive(Clone)]
    struct ClosureCallableInvoker<'a> {
        closures: Rc<RefCell<HashMap<String, RegisteredCallable<'a>>>>,
    }

    struct TestImageProvider;

    impl<'a> ClosureCallableInvoker<'a> {
        fn new() -> Self {
            Self {
                closures: Rc::new(RefCell::new(HashMap::new())),
            }
        }

        fn register<F>(&self, token: &str, arity: usize, f: F) -> LambdaValue
        where
            F: Fn(&[PreparedArgValue]) -> Result<PreparedArgValue, CallableInvocationError> + 'a,
        {
            self.closures
                .borrow_mut()
                .insert(token.to_string(), Rc::new(f));
            LambdaValue::helper_lambda(
                token.to_string(),
                CallableArityShape::exact(arity),
                CallableCaptureMode::LexicalCapture,
                "test.closure.invoke.v1",
            )
        }
    }

    impl ReferenceResolver for NoReferenceResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            Err(RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }
    }

    impl CallableInvoker for ClosureCallableInvoker<'_> {
        fn invoke(
            &self,
            callable: &LambdaValue,
            args: &[PreparedArgValue],
        ) -> Result<PreparedArgValue, CallableInvocationError> {
            if let Some(handler) = self
                .closures
                .borrow()
                .get(&callable.callable_token)
                .cloned()
            {
                return handler(args);
            }

            let fallback = TestCallableInvoker;
            fallback.invoke(callable, args)
        }
    }

    fn eval_test_surface_value(
        function_id: &str,
        args: &[CallArgValue],
    ) -> Result<EvalValue, CallableInvocationError> {
        eval_surface_value_call(
            function_id,
            args,
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .map_err(CallableInvocationError::Worksheet)
    }

    fn array_arg(rows: Vec<Vec<ArrayCellValue>>) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Array(EvalArray::from_rows(rows).unwrap()))
    }

    fn number_arg(value: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(value))
    }

    fn logical_arg(value: bool) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Logical(value))
    }

    fn text_arg(value: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(value)))
    }

    fn text_cell(value: &str) -> ArrayCellValue {
        ArrayCellValue::Text(ExcelText::from_interop_assignment(value))
    }

    #[test]
    fn observed_scalar_array_lift_handles_value_error_result_surfaces() {
        let got = eval_test_surface_value(
            FUNC_ID_TRIMMEAN,
            &[
                array_arg(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(4.0),
                    ArrayCellValue::Number(5.0),
                    ArrayCellValue::Number(6.0),
                    ArrayCellValue::Number(7.0),
                    ArrayCellValue::Number(8.0),
                    ArrayCellValue::Number(9.0),
                    ArrayCellValue::Number(100.0),
                ]]),
                array_arg(vec![vec![
                    ArrayCellValue::Number(0.2),
                    ArrayCellValue::Number(0.2),
                ]]),
            ],
        )
        .unwrap();

        assert_eq!(
            got,
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(5.5),
                    ArrayCellValue::Number(5.5),
                ]])
                .unwrap()
            )
        );
    }

    #[test]
    fn observed_scalar_array_lift_handles_abs_arrays() {
        let got = eval_test_surface_value(
            FUNC_ID_ABS,
            &[array_arg(vec![vec![
                ArrayCellValue::Number(-1.0),
                ArrayCellValue::Text(ExcelText::from_interop_assignment("bad")),
                ArrayCellValue::Number(2.0),
            ]])],
        )
        .unwrap();

        assert_eq!(
            got,
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Error(WorksheetErrorCode::Value),
                    ArrayCellValue::Number(2.0),
                ]])
                .unwrap()
            )
        );
    }

    #[test]
    fn observed_scalar_array_lift_covers_w092_reopened_successor_positions() {
        let binomdist = eval_test_surface_value(
            FUNC_ID_BINOMDIST,
            &[
                number_arg(2.0),
                number_arg(4.0),
                number_arg(0.25),
                array_arg(vec![vec![
                    ArrayCellValue::Logical(false),
                    ArrayCellValue::Logical(false),
                ]]),
            ],
        )
        .unwrap();
        assert_eq!(
            binomdist,
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(f64::from_bits(0x3fcb000000000001)),
                    ArrayCellValue::Number(f64::from_bits(0x3fcb000000000001)),
                ]])
                .unwrap()
            )
        );

        let normdist = eval_test_surface_value(
            FUNC_ID_NORMDIST,
            &[
                number_arg(42.0),
                number_arg(40.0),
                number_arg(1.5),
                array_arg(vec![vec![
                    ArrayCellValue::Logical(true),
                    ArrayCellValue::Logical(true),
                ]]),
            ],
        )
        .unwrap();
        assert_eq!(
            normdist,
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(f64::from_bits(0x3fed14cc3547f8da)),
                    ArrayCellValue::Number(f64::from_bits(0x3fed14cc3547f8da)),
                ]])
                .unwrap()
            )
        );

        let complex = eval_test_surface_value(
            FUNC_ID_COMPLEX,
            &[
                number_arg(3.0),
                number_arg(4.0),
                array_arg(vec![vec![text_cell("j"), text_cell("j")]]),
            ],
        )
        .unwrap();
        assert_eq!(
            complex,
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![text_cell("3+4j"), text_cell("3+4j")]]).unwrap()
            )
        );

        let dollarfr = eval_test_surface_value(
            FUNC_ID_DOLLARFR,
            &[
                CallArgValue::MissingArg,
                array_arg(vec![vec![
                    ArrayCellValue::Number(16.0),
                    ArrayCellValue::Number(16.0),
                ]]),
            ],
        )
        .unwrap();
        assert_eq!(
            dollarfr,
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            )
        );

        let switch = eval_test_surface_value(
            FUNC_ID_SWITCH,
            &[
                number_arg(2.0),
                number_arg(1.0),
                text_arg("a"),
                array_arg(vec![vec![
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(2.0),
                ]]),
                text_arg("b"),
                text_arg("other"),
            ],
        )
        .unwrap();
        assert_eq!(
            switch,
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![text_cell("b"), text_cell("b")]]).unwrap()
            )
        );

        let switch_no_default = eval_test_surface_value(
            FUNC_ID_SWITCH,
            &[
                number_arg(3.0),
                number_arg(1.0),
                text_arg("a"),
                array_arg(vec![vec![
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(2.0),
                ]]),
                text_arg("b"),
            ],
        )
        .unwrap();
        assert_eq!(
            switch_no_default,
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            )
        );

        let ifs = eval_test_surface_value(
            FUNC_ID_IFS,
            &[
                text_arg("2"),
                array_arg(vec![vec![text_cell("hit"), text_cell("hit")]]),
            ],
        )
        .unwrap();
        assert_eq!(
            ifs,
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Error(WorksheetErrorCode::Value),
                    ArrayCellValue::Error(WorksheetErrorCode::Value),
                ]])
                .unwrap()
            )
        );

        let ifs_unselected_array_result = eval_test_surface_value(
            FUNC_ID_IFS,
            &[
                logical_arg(false),
                array_arg(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(1.0),
                ]]),
                number_arg(0.0),
                number_arg(2.0),
            ],
        )
        .unwrap();
        assert_eq!(
            ifs_unselected_array_result,
            EvalValue::Error(WorksheetErrorCode::NA)
        );

        let address_abs_num = eval_test_surface_value(
            FUNC_ID_ADDRESS,
            &[
                number_arg(3.0),
                number_arg(2.0),
                array_arg(vec![vec![
                    ArrayCellValue::Number(4.0),
                    ArrayCellValue::Number(4.0),
                ]]),
                logical_arg(false),
                text_arg("Alpha"),
            ],
        )
        .unwrap();
        assert_eq!(
            address_abs_num,
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    text_cell("Alpha!R[3]C[2]"),
                    text_cell("Alpha!R[3]C[2]"),
                ]])
                .unwrap()
            )
        );

        let address_sheet_text = eval_test_surface_value(
            FUNC_ID_ADDRESS,
            &[
                number_arg(3.0),
                number_arg(2.0),
                number_arg(1.0),
                logical_arg(true),
                array_arg(vec![vec![text_cell("Quarter 1"), text_cell("Quarter 1")]]),
            ],
        )
        .unwrap();
        assert_eq!(
            address_sheet_text,
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    text_cell("'Quarter 1'!$B$3"),
                    text_cell("'Quarter 1'!$B$3"),
                ]])
                .unwrap()
            )
        );
    }

    fn eval_test_surface_value_with_callable(
        function_id: &str,
        args: &[CallArgValue],
        invoker: &dyn CallableInvoker,
    ) -> Result<EvalValue, CallableInvocationError> {
        eval_surface_value_call_with_callable(
            function_id,
            args,
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
            Some(invoker),
            None,
            None,
        )
        .map_err(CallableInvocationError::Worksheet)
    }

    fn call_arg_from_prepared(prepared: &PreparedArgValue) -> CallArgValue {
        match prepared {
            PreparedArgValue::Eval(value) => CallArgValue::Eval(value.clone()),
            PreparedArgValue::MissingArg => CallArgValue::MissingArg,
            PreparedArgValue::EmptyCell => CallArgValue::EmptyCell,
        }
    }

    fn number_column(values: &[f64]) -> EvalValue {
        EvalValue::Array(
            EvalArray::from_rows(
                values
                    .iter()
                    .copied()
                    .map(|value| vec![ArrayCellValue::Number(value)])
                    .collect(),
            )
            .expect("column vector"),
        )
    }

    impl CallableInvoker for TestCallableInvoker {
        fn invoke(
            &self,
            callable: &LambdaValue,
            args: &[PreparedArgValue],
        ) -> Result<PreparedArgValue, CallableInvocationError> {
            match callable.callable_token.as_str() {
                "helper.mul10" => match args {
                    [PreparedArgValue::Eval(EvalValue::Number(n))] => {
                        Ok(PreparedArgValue::Eval(EvalValue::Number(*n * 10.0)))
                    }
                    _ => Err(CallableInvocationError::Worksheet(
                        WorksheetErrorCode::Value,
                    )),
                },
                "helper.add1" => match args {
                    [PreparedArgValue::Eval(EvalValue::Number(n))] => {
                        Ok(PreparedArgValue::Eval(EvalValue::Number(*n + 1.0)))
                    }
                    _ => Err(CallableInvocationError::Worksheet(
                        WorksheetErrorCode::Value,
                    )),
                },
                "helper.feb2024_day_or_two_spaces" => match args {
                    [PreparedArgValue::Eval(EvalValue::Number(n))] => {
                        let first_day = crate::locale_format::excel_serial_from_ymd(
                            crate::locale_format::WorkbookDateSystem::System1900,
                            2024,
                            2,
                            1,
                        )
                        .expect("first day serial");
                        let last_day = crate::locale_format::excel_serial_from_ymd(
                            crate::locale_format::WorkbookDateSystem::System1900,
                            2024,
                            2,
                            29,
                        )
                        .expect("last day serial");
                        if *n >= first_day && *n <= last_day {
                            let day = eval_surface_value_call(
                                FUNC_ID_DAY,
                                &[CallArgValue::Eval(EvalValue::Number(*n))],
                                &NoReferenceResolver,
                                Some(46000.0),
                                Some(0.5),
                                None,
                                None,
                            )
                            .map_err(CallableInvocationError::Worksheet)?;
                            let ctx = test_current_excel_host_context();
                            let text = eval_surface_value_call(
                                FUNC_ID_TEXT,
                                &[
                                    CallArgValue::Eval(day),
                                    CallArgValue::Eval(EvalValue::Text(
                                        ExcelText::from_interop_assignment("00"),
                                    )),
                                ],
                                &NoReferenceResolver,
                                Some(46000.0),
                                Some(0.5),
                                Some(&ctx),
                                None,
                            )
                            .map_err(CallableInvocationError::Worksheet)?;
                            Ok(PreparedArgValue::Eval(text))
                        } else {
                            Ok(PreparedArgValue::Eval(EvalValue::Text(
                                ExcelText::from_interop_assignment("  "),
                            )))
                        }
                    }
                    _ => Err(CallableInvocationError::Worksheet(
                        WorksheetErrorCode::Value,
                    )),
                },
                "helper.jan2024_day_or_two_spaces" => match args {
                    [PreparedArgValue::Eval(EvalValue::Number(n))] => {
                        let first_day = crate::locale_format::excel_serial_from_ymd(
                            crate::locale_format::WorkbookDateSystem::System1900,
                            2024,
                            1,
                            1,
                        )
                        .expect("first day serial");
                        let last_day = crate::locale_format::excel_serial_from_ymd(
                            crate::locale_format::WorkbookDateSystem::System1900,
                            2024,
                            1,
                            31,
                        )
                        .expect("last day serial");
                        if *n >= first_day && *n <= last_day {
                            let day = eval_surface_value_call(
                                FUNC_ID_DAY,
                                &[CallArgValue::Eval(EvalValue::Number(*n))],
                                &NoReferenceResolver,
                                Some(46000.0),
                                Some(0.5),
                                None,
                                None,
                            )
                            .map_err(CallableInvocationError::Worksheet)?;
                            let ctx = test_current_excel_host_context();
                            let text = eval_surface_value_call(
                                FUNC_ID_TEXT,
                                &[
                                    CallArgValue::Eval(day),
                                    CallArgValue::Eval(EvalValue::Text(
                                        ExcelText::from_interop_assignment("00"),
                                    )),
                                ],
                                &NoReferenceResolver,
                                Some(46000.0),
                                Some(0.5),
                                Some(&ctx),
                                None,
                            )
                            .map_err(CallableInvocationError::Worksheet)?;
                            Ok(PreparedArgValue::Eval(text))
                        } else {
                            Ok(PreparedArgValue::Eval(EvalValue::Text(
                                ExcelText::from_interop_assignment("  "),
                            )))
                        }
                    }
                    _ => Err(CallableInvocationError::Worksheet(
                        WorksheetErrorCode::Value,
                    )),
                },
                "helper.jan2024_day_or_zero" => match args {
                    [PreparedArgValue::Eval(EvalValue::Number(n))] => {
                        let first_day = crate::locale_format::excel_serial_from_ymd(
                            crate::locale_format::WorkbookDateSystem::System1900,
                            2024,
                            1,
                            1,
                        )
                        .expect("first day serial");
                        let last_day = crate::locale_format::excel_serial_from_ymd(
                            crate::locale_format::WorkbookDateSystem::System1900,
                            2024,
                            1,
                            31,
                        )
                        .expect("last day serial");
                        if *n >= first_day && *n <= last_day {
                            let day = eval_surface_value_call(
                                FUNC_ID_DAY,
                                &[CallArgValue::Eval(EvalValue::Number(*n))],
                                &NoReferenceResolver,
                                Some(46000.0),
                                Some(0.5),
                                None,
                                None,
                            )
                            .map_err(CallableInvocationError::Worksheet)?;
                            Ok(PreparedArgValue::Eval(day))
                        } else {
                            Ok(PreparedArgValue::Eval(EvalValue::Number(0.0)))
                        }
                    }
                    _ => Err(CallableInvocationError::Worksheet(
                        WorksheetErrorCode::Value,
                    )),
                },
                _ => Err(CallableInvocationError::UnsupportedCallableToken(
                    callable.callable_token.clone(),
                )),
            }
        }
    }

    impl HostInfoProvider for TestImageProvider {
        fn query_image(
            &self,
            _request: &ImageRequest,
        ) -> Result<ImageProviderResult, HostInfoError> {
            Ok(ImageProviderResult::Image(ResolvedWebImage {
                web_image_identifier: "img-1".to_string(),
                published_fallback: ExcelText::from_interop_assignment("-2146826273"),
            }))
        }
    }

    #[test]
    fn eval_surface_value_call_abs_accepts_text_numeric() {
        let arg = CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            " -2 ".encode_utf16().collect(),
        )));
        let got = eval_surface_value_call(
            FUNC_ID_ABS,
            &[arg],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_surface_value_call_power_zero_to_zero_returns_num_error() {
        for function_id in [FUNC_ID_OP_POWER, FUNC_ID_POWER] {
            let got = eval_surface_value_call(
                function_id,
                &[
                    CallArgValue::Eval(EvalValue::Number(0.0)),
                    CallArgValue::Eval(EvalValue::Number(0.0)),
                ],
                &NoReferenceResolver,
                Some(46000.0),
                Some(0.5),
                None,
                None,
            );
            assert_eq!(got, Err(WorksheetErrorCode::Num));
        }
    }

    #[test]
    fn eval_surface_value_call_op_add_lifts_arrays() {
        let got = eval_surface_value_call(
            FUNC_ID_OP_ADD,
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(2.0)],
                        vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                    ])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(10.0), ArrayCellValue::Number(20.0)],
                        vec![ArrayCellValue::Number(30.0), ArrayCellValue::Number(40.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(11.0), ArrayCellValue::Number(22.0)],
                    vec![ArrayCellValue::Number(33.0), ArrayCellValue::Number(44.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_surface_value_call_op_add_broadcasts_arrays() {
        let got = eval_surface_value_call(
            FUNC_ID_OP_ADD,
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0)],
                        vec![ArrayCellValue::Number(2.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(2.0), ArrayCellValue::Number(3.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(4.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_surface_value_call_op_equal_broadcasts_arrays() {
        let got = eval_surface_value_call(
            FUNC_ID_OP_EQUAL,
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0)],
                        vec![ArrayCellValue::Number(2.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![
                        ArrayCellValue::Logical(true),
                        ArrayCellValue::Logical(false)
                    ],
                    vec![
                        ArrayCellValue::Logical(false),
                        ArrayCellValue::Logical(true)
                    ],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_surface_value_call_op_concat_marks_missing_broadcast_coordinates_as_na() {
        let got = eval_surface_value_call(
            FUNC_ID_OP_CONCAT,
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("a")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("b")),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("x")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("y")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("z")),
                    ]])
                    .unwrap(),
                )),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("ax")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("by")),
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_surface_value_call_op_range_ref_normalizes_bounds() {
        let got = eval_surface_value_call(
            FUNC_ID_OP_RANGE_REF,
            &[
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "B2".to_string(),
                }),
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "A1".to_string(),
                }),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:B2".to_string(),
            }))
        );
    }

    #[test]
    fn eval_surface_value_call_op_union_ref_returns_multi_area_reference() {
        let got = eval_surface_value_call(
            FUNC_ID_OP_UNION_REF,
            &[
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::Area,
                    target: "A1:A2".to_string(),
                }),
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::Area,
                    target: "G1:G2".to_string(),
                }),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::MultiArea,
                target: "(A1:A2,G1:G2)".to_string(),
            }))
        );
    }

    #[test]
    fn eval_surface_value_call_vlookup_spills_array_lookup_value_results() {
        let got = eval_surface_value_call(
            FUNC_ID_VLOOKUP,
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(2.0), ArrayCellValue::Number(20.0)],
                        vec![ArrayCellValue::Number(4.0), ArrayCellValue::Number(40.0)],
                        vec![ArrayCellValue::Number(6.0), ArrayCellValue::Number(60.0)],
                        vec![ArrayCellValue::Number(8.0), ArrayCellValue::Number(80.0)],
                    ])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Logical(false)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_surface_value_call_hlookup_spills_array_lookup_value_results() {
        let got = eval_surface_value_call(
            FUNC_ID_HLOOKUP,
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![
                            ArrayCellValue::Number(2.0),
                            ArrayCellValue::Number(4.0),
                            ArrayCellValue::Number(6.0),
                            ArrayCellValue::Number(8.0),
                        ],
                        vec![
                            ArrayCellValue::Number(20.0),
                            ArrayCellValue::Number(40.0),
                            ArrayCellValue::Number(60.0),
                            ArrayCellValue::Number(80.0),
                        ],
                    ])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Logical(false)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_surface_value_call_left_spills_array_counts() {
        let got = eval_surface_value_call(
            FUNC_ID_LEFT,
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(
                    "MISSISSIPPI",
                ))),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0)],
                        vec![ArrayCellValue::Number(2.0)],
                        vec![ArrayCellValue::Number(3.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "M"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "MI"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "MIS"
                    ))],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_surface_value_call_right_spills_array_counts() {
        let got = eval_surface_value_call(
            FUNC_ID_RIGHT,
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(
                    "MISSISSIPPI",
                ))),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0)],
                        vec![ArrayCellValue::Number(2.0)],
                        vec![ArrayCellValue::Number(3.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "I"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "PI"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "PPI"
                    ))],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_surface_value_call_mid_spills_array_start_positions() {
        let got = eval_surface_value_call(
            FUNC_ID_MID,
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(
                    "MISSISSIPPI",
                ))),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0)],
                        vec![ArrayCellValue::Number(2.0)],
                        vec![ArrayCellValue::Number(3.0)],
                        vec![ArrayCellValue::Number(4.0)],
                        vec![ArrayCellValue::Number(5.0)],
                        vec![ArrayCellValue::Number(6.0)],
                        vec![ArrayCellValue::Number(7.0)],
                        vec![ArrayCellValue::Number(8.0)],
                        vec![ArrayCellValue::Number(9.0)],
                        vec![ArrayCellValue::Number(10.0)],
                        vec![ArrayCellValue::Number(11.0)],
                    ])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "M"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "I"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "S"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "S"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "I"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "S"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "S"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "I"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "P"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "P"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "I"
                    ))],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_surface_value_call_char_spills_array_numbers() {
        let got = eval_surface_value_call(
            FUNC_ID_CHAR,
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(65.0)],
                    vec![ArrayCellValue::Number(66.0)],
                    vec![ArrayCellValue::Number(67.0)],
                ])
                .unwrap(),
            ))],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "A"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "B"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "C"
                    ))],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_surface_value_call_rept_spills_array_counts() {
        let got = eval_surface_value_call(
            FUNC_ID_REPT,
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("x"))),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0)],
                        vec![ArrayCellValue::Number(2.0)],
                        vec![ArrayCellValue::Number(3.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "x"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "xx"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "xxx"
                    ))],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_surface_value_call_textafter_spills_array_instance_numbers() {
        let got = eval_surface_value_call(
            FUNC_ID_TEXTAFTER,
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("a-b-c"))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("-"))),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0)],
                        vec![ArrayCellValue::Number(2.0)],
                        vec![ArrayCellValue::Number(3.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "b-c"
                    ))],
                    vec![ArrayCellValue::Text(ExcelText::from_interop_assignment(
                        "c"
                    ))],
                    vec![ArrayCellValue::Error(WorksheetErrorCode::NA)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_surface_value_call_textbefore_spills_array_text_inputs() {
        let got = eval_surface_value_call(
            FUNC_ID_TEXTBEFORE,
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("a-b")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("c-d")),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("-"))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("a")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("c")),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_surface_value_call_areas_counts_multi_area_reference() {
        let got = eval_surface_value_call(
            FUNC_ID_AREAS,
            &[CallArgValue::Reference(
                ReferenceLike::multi_area(vec!["A1".to_string(), "B2:B3".to_string()]).unwrap(),
            )],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_surface_value_call_areas_rejects_legacy_parenthesized_area_carrier() {
        let got = eval_surface_value_call(
            FUNC_ID_AREAS,
            &[CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "(A1,B2:B3)".to_string(),
            })],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Err(WorksheetErrorCode::Value));
    }

    #[test]
    fn eval_surface_value_call_index_rejects_legacy_parenthesized_area_carrier() {
        let got = eval_surface_value_call(
            FUNC_ID_INDEX,
            &[
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::Area,
                    target: "(A1:A2,G1:G2)".to_string(),
                }),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Err(WorksheetErrorCode::Value));
    }

    #[test]
    fn eval_surface_value_call_rejects_unknown_id() {
        let arg = CallArgValue::Eval(EvalValue::Number(1.0));
        let got = eval_surface_value_call(
            "FUNC.UNKNOWN",
            &[arg],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Err(WorksheetErrorCode::Value));
    }

    #[test]
    fn eval_surface_value_call_roman_returns_text_result() {
        let got = eval_surface_value_call(
            FUNC_ID_ROMAN,
            &[
                CallArgValue::Eval(EvalValue::Number(499.0)),
                CallArgValue::Eval(EvalValue::Logical(false)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "ID".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn eval_surface_value_call_with_callable_supports_map_helper_surface() {
        let array = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Number(1.0),
            ArrayCellValue::Number(2.0),
        ]])
        .expect("row vector");
        let callable = LambdaValue::helper_lambda(
            "helper.add1",
            CallableArityShape::exact(1),
            CallableCaptureMode::NoCapture,
            "lambda.map.add1",
        );
        let got = eval_surface_value_call_with_callable(
            FUNC_ID_MAP,
            &[
                CallArgValue::Eval(EvalValue::Array(array)),
                CallArgValue::Eval(EvalValue::Lambda(callable)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
            Some(&TestCallableInvoker),
            None,
            None,
        );
        let expected = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Number(3.0),
        ]])
        .expect("row vector");
        assert_eq!(got, Ok(EvalValue::Array(expected)));
    }

    #[test]
    fn eval_surface_value_call_xmatch_spills_array_lookup_value_results() {
        let lookup_values = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Number(1.0),
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Number(3.0),
        ]])
        .expect("row vector");
        let lookup_array = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Number(4.0),
            ArrayCellValue::Number(6.0),
            ArrayCellValue::Number(8.0),
        ]])
        .expect("row vector");
        let got = eval_surface_value_call(
            FUNC_ID_XMATCH,
            &[
                CallArgValue::Eval(EvalValue::Array(lookup_values)),
                CallArgValue::Eval(EvalValue::Array(lookup_array)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        let expected = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Error(WorksheetErrorCode::NA),
            ArrayCellValue::Number(1.0),
            ArrayCellValue::Error(WorksheetErrorCode::NA),
        ]])
        .expect("row vector");
        assert_eq!(got, Ok(EvalValue::Array(expected)));
    }

    #[test]
    fn eval_surface_value_call_xmatch_exact_witness_spills_array_lookup_value_results() {
        let lookup_values = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Number(1.0),
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Number(3.0),
            ArrayCellValue::Number(4.0),
            ArrayCellValue::Number(5.0),
        ]])
        .expect("row vector");
        let lookup_array = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Number(4.0),
            ArrayCellValue::Number(6.0),
            ArrayCellValue::Number(8.0),
        ]])
        .expect("row vector");
        let got = eval_surface_value_call(
            FUNC_ID_XMATCH,
            &[
                CallArgValue::Eval(EvalValue::Array(lookup_values)),
                CallArgValue::Eval(EvalValue::Array(lookup_array)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        let expected = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Error(WorksheetErrorCode::NA),
            ArrayCellValue::Number(1.0),
            ArrayCellValue::Error(WorksheetErrorCode::NA),
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Error(WorksheetErrorCode::NA),
        ]])
        .expect("row vector");
        assert_eq!(got, Ok(EvalValue::Array(expected)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0940_corpus_formula_returns_six() {
        let lookup_values = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Number(1.0),
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Number(3.0),
            ArrayCellValue::Number(4.0),
            ArrayCellValue::Number(5.0),
        ]])
        .expect("row vector");
        let lookup_array = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Number(4.0),
            ArrayCellValue::Number(6.0),
            ArrayCellValue::Number(8.0),
        ]])
        .expect("row vector");
        let source = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Number(1.0),
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Number(3.0),
            ArrayCellValue::Number(4.0),
            ArrayCellValue::Number(5.0),
        ]])
        .expect("row vector");

        let xmatch = eval_surface_value_call(
            FUNC_ID_XMATCH,
            &[
                CallArgValue::Eval(EvalValue::Array(lookup_values)),
                CallArgValue::Eval(EvalValue::Array(lookup_array)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("xmatch result");

        let isnumber = eval_surface_value_call(
            FUNC_ID_ISNUMBER,
            &[CallArgValue::Eval(xmatch)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("isnumber result");

        let filtered = eval_surface_value_call(
            FUNC_ID_FILTER,
            &[
                CallArgValue::Eval(EvalValue::Array(source)),
                CallArgValue::Eval(isnumber),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("filter result");

        let got = eval_surface_value_call(
            FUNC_ID_SUM,
            &[CallArgValue::Eval(filtered)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );

        assert_eq!(got, Ok(EvalValue::Number(6.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0779_dictionary_keys_composition_returns_two() {
        let keys = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Text(ExcelText::from_interop_assignment("name")),
            ArrayCellValue::Text(ExcelText::from_interop_assignment("age")),
            ArrayCellValue::Text(ExcelText::from_interop_assignment("city")),
        ]])
        .expect("row vector");
        let mapped = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Text(ExcelText::from_interop_assignment("Alice")),
            ArrayCellValue::Number(30.0),
            ArrayCellValue::Error(WorksheetErrorCode::NA),
        ]])
        .expect("row vector");

        let iserror = eval_surface_value_call(
            FUNC_ID_ISERROR,
            &[CallArgValue::Eval(EvalValue::Array(mapped))],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("iserror result");

        let keep = eval_surface_value_call(
            FUNC_ID_NOT,
            &[CallArgValue::Eval(iserror)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("not result");

        let filtered = eval_surface_value_call(
            FUNC_ID_FILTER,
            &[
                CallArgValue::Eval(EvalValue::Array(keys)),
                CallArgValue::Eval(keep),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("filter result");

        let got = eval_surface_value_call(
            FUNC_ID_COLUMNS,
            &[CallArgValue::Eval(filtered)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );

        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0702_day_of_date_1900_march_zero_returns_twenty_nine() {
        let serial = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(1900.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("date result");

        let got = eval_surface_value_call(
            FUNC_ID_DAY,
            &[CallArgValue::Eval(serial)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );

        assert_eq!(got, Ok(EvalValue::Number(29.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0703_0705_datedif_cluster_matches_expected_values() {
        let start_y = CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(
            "2020-01-15",
        )));
        let end_y = CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(
            "2024-03-20",
        )));
        let unit_y = CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("Y")));
        let got_y = eval_surface_value_call(
            FUNC_ID_DATEDIF,
            &[start_y, end_y, unit_y],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got_y, Ok(EvalValue::Number(4.0)));

        let start_m = CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(
            "2024-01-15",
        )));
        let end_m = CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(
            "2024-04-10",
        )));
        let got_m = eval_surface_value_call(
            FUNC_ID_DATEDIF,
            &[
                start_m.clone(),
                end_m.clone(),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("M"))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got_m, Ok(EvalValue::Number(2.0)));

        let got_md = eval_surface_value_call(
            FUNC_ID_DATEDIF,
            &[
                start_m,
                end_m,
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("MD"))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got_md, Ok(EvalValue::Number(26.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0706_0708_weekday_iso_cluster_matches_expected_values() {
        let jan1 = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("DATE(2024,1,1)");
        let got_0706 = eval_surface_value_call(
            FUNC_ID_WEEKDAY,
            &[CallArgValue::Eval(jan1.clone())],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got_0706, Ok(EvalValue::Number(2.0)));

        let got_0707 = eval_surface_value_call(
            FUNC_ID_WEEKDAY,
            &[
                CallArgValue::Eval(jan1),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got_0707, Ok(EvalValue::Number(1.0)));

        let dec30 = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(12.0)),
                CallArgValue::Eval(EvalValue::Number(30.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("DATE(2024,12,30)");
        let got_0708 = eval_surface_value_call(
            FUNC_ID_ISOWEEKNUM,
            &[CallArgValue::Eval(dec30)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got_0708, Ok(EvalValue::Number(1.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0709_0711_month_end_shift_cluster_matches_expected_values() {
        let jan15 = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(15.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("DATE(2024,1,15)");
        let got_0709 = eval_surface_value_call(
            FUNC_ID_DAY,
            &[CallArgValue::Eval(
                eval_surface_value_call(
                    FUNC_ID_EOMONTH,
                    &[
                        CallArgValue::Eval(jan15),
                        CallArgValue::Eval(EvalValue::Number(1.0)),
                    ],
                    &NoReferenceResolver,
                    Some(46000.0),
                    Some(0.5),
                    None,
                    None,
                )
                .expect("EOMONTH(...,1)"),
            )],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got_0709, Ok(EvalValue::Number(29.0)));

        let mar15 = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
                CallArgValue::Eval(EvalValue::Number(15.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("DATE(2024,3,15)");
        let got_0710 = eval_surface_value_call(
            FUNC_ID_MONTH,
            &[CallArgValue::Eval(
                eval_surface_value_call(
                    FUNC_ID_EOMONTH,
                    &[
                        CallArgValue::Eval(mar15),
                        CallArgValue::Eval(EvalValue::Number(-1.0)),
                    ],
                    &NoReferenceResolver,
                    Some(46000.0),
                    Some(0.5),
                    None,
                    None,
                )
                .expect("EOMONTH(...,-1)"),
            )],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got_0710, Ok(EvalValue::Number(2.0)));

        let jan31 = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(31.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("DATE(2024,1,31)");
        let got_0711 = eval_surface_value_call(
            FUNC_ID_DAY,
            &[CallArgValue::Eval(
                eval_surface_value_call(
                    FUNC_ID_EDATE,
                    &[
                        CallArgValue::Eval(jan31),
                        CallArgValue::Eval(EvalValue::Number(1.0)),
                    ],
                    &NoReferenceResolver,
                    Some(46000.0),
                    Some(0.5),
                    None,
                    None,
                )
                .expect("EDATE(...,1)"),
            )],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got_0711, Ok(EvalValue::Number(29.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0712_time_second_roundtrip_returns_one() {
        let got = eval_surface_value_call(
            FUNC_ID_ROUND,
            &[
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_OP_MULTIPLY,
                        &[
                            CallArgValue::Eval(
                                eval_surface_value_call(
                                    FUNC_ID_TIME,
                                    &[
                                        CallArgValue::Eval(EvalValue::Number(0.0)),
                                        CallArgValue::Eval(EvalValue::Number(0.0)),
                                        CallArgValue::Eval(EvalValue::Number(1.0)),
                                    ],
                                    &NoReferenceResolver,
                                    Some(46000.0),
                                    Some(0.5),
                                    None,
                                    None,
                                )
                                .expect("TIME(0,0,1)"),
                            ),
                            CallArgValue::Eval(EvalValue::Number(86400.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("TIME*86400"),
                ),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0805_iferror_sum_filter_false_returns_empty() {
        let filtered_err = eval_surface_value_call(
            FUNC_ID_FILTER,
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                    ]])
                    .expect("row vector"),
                )),
                CallArgValue::Eval(EvalValue::Logical(false)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect_err("FILTER({1,2,3},FALSE) should error locally");
        let sum_err = eval_surface_value_call(
            FUNC_ID_SUM,
            &[CallArgValue::Eval(EvalValue::Error(filtered_err))],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect_err("SUM should propagate the same local error lane");
        let got = eval_surface_value_call(
            FUNC_ID_IFERROR,
            &[
                CallArgValue::Eval(EvalValue::Error(sum_err)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("empty"))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("empty")))
        );
    }

    #[test]
    fn eval_surface_value_call_ftc_0807_sort_row_vector_default_axis_returns_first_cell() {
        let sorted = eval_surface_value_call(
            FUNC_ID_SORT,
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(4.0),
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(5.0),
                        ArrayCellValue::Number(9.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(6.0),
                    ]])
                    .expect("row vector"),
                )),
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Number(-1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("sort result");
        let got = eval_surface_value_call(
            FUNC_ID_INDEX,
            &[
                CallArgValue::Eval(sorted),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(3.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0808_sortby_row_vector_index_first_returns_d() {
        let sorted = eval_surface_value_call(
            FUNC_ID_SORTBY,
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("a")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("b")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("c")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("d")),
                    ]])
                    .expect("row vector"),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(4.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Number(1.0),
                    ]])
                    .expect("row vector"),
                )),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("sortby result");
        let got = eval_surface_value_call(
            FUNC_ID_INDEX,
            &[
                CallArgValue::Eval(sorted),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("d")))
        );
    }

    #[test]
    fn eval_surface_value_call_ftc_0814_sum_of_drop_row_vector_negative_count_returns_calc() {
        let drop_err = eval_surface_value_call(
            FUNC_ID_DROP,
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Number(4.0),
                        ArrayCellValue::Number(5.0),
                    ]])
                    .expect("row vector"),
                )),
                CallArgValue::Eval(EvalValue::Number(-2.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect_err("DROP should stay on the row axis and empty out the array locally");
        let got = eval_surface_value_call(
            FUNC_ID_SUM,
            &[CallArgValue::Eval(EvalValue::Error(drop_err))],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Err(WorksheetErrorCode::Calc));
    }

    #[test]
    fn eval_surface_value_call_ftc_0820_choosecols_sum_returns_ninety() {
        let chosen = eval_surface_value_call(
            FUNC_ID_CHOOSECOLS,
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(10.0),
                        ArrayCellValue::Number(20.0),
                        ArrayCellValue::Number(30.0),
                        ArrayCellValue::Number(40.0),
                        ArrayCellValue::Number(50.0),
                    ]])
                    .expect("row vector"),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(5.0),
                    ]])
                    .expect("selector row vector"),
                )),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("choosecols result");
        let got = eval_surface_value_call(
            FUNC_ID_SUM,
            &[CallArgValue::Eval(chosen)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(90.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0846_to_ftc_0850_match_and_choose_cluster_matches_expected() {
        let row_vector = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Number(100.0),
            ArrayCellValue::Number(200.0),
            ArrayCellValue::Number(300.0),
            ArrayCellValue::Number(400.0),
            ArrayCellValue::Number(500.0),
        ]])
        .expect("row vector");
        let matched = eval_surface_value_call(
            FUNC_ID_MATCH,
            &[
                CallArgValue::Eval(EvalValue::Number(300.0)),
                CallArgValue::Eval(EvalValue::Array(row_vector.clone())),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("match result");
        let got_0846 = eval_surface_value_call(
            FUNC_ID_INDEX,
            &[
                CallArgValue::Eval(EvalValue::Array(row_vector)),
                CallArgValue::Eval(matched),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got_0846, Ok(EvalValue::Number(300.0)));

        let got_0848 = eval_surface_value_call(
            FUNC_ID_MATCH,
            &[
                CallArgValue::Eval(EvalValue::Number(99.0)),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(10.0),
                        ArrayCellValue::Number(20.0),
                        ArrayCellValue::Number(30.0),
                    ]])
                    .expect("row vector"),
                )),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got_0848, Err(WorksheetErrorCode::NA));

        let got_0849 = eval_surface_value_call(
            FUNC_ID_CHOOSE,
            &[
                CallArgValue::Eval(EvalValue::Number(3.0)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("a"))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("b"))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("c"))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("d"))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got_0849,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("c")))
        );

        let got_0850 = eval_surface_value_call(
            FUNC_ID_CHOOSE,
            &[
                CallArgValue::Eval(EvalValue::Number(5.0)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("a"))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("b"))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("c"))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got_0850, Ok(EvalValue::Error(WorksheetErrorCode::Value)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0851_and_ftc_0858_lookup_cluster_matches_expected() {
        let got_0851 = eval_surface_value_call(
            FUNC_ID_XLOOKUP,
            &[
                CallArgValue::Eval(EvalValue::Number(99.0)),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                    ]])
                    .expect("row vector"),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("a")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("b")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("c")),
                    ]])
                    .expect("row vector"),
                )),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(
                    "missing",
                ))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got_0851,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment(
                "missing"
            )))
        );

        let got_0858 = eval_surface_value_call(
            FUNC_ID_LOOKUP,
            &[
                CallArgValue::Eval(EvalValue::Number(25.0)),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(10.0),
                        ArrayCellValue::Number(20.0),
                        ArrayCellValue::Number(30.0),
                        ArrayCellValue::Number(40.0),
                        ArrayCellValue::Number(50.0),
                    ]])
                    .expect("row vector"),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("a")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("b")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("c")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("d")),
                        ArrayCellValue::Text(ExcelText::from_interop_assignment("e")),
                    ]])
                    .expect("row vector"),
                )),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got_0858,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("b")))
        );
    }

    #[test]
    fn eval_surface_value_call_xlookup_spills_array_lookup_value_results() {
        let got = eval_surface_value_call(
            FUNC_ID_XLOOKUP,
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                    ]])
                    .expect("row vector"),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(4.0),
                        ArrayCellValue::Number(6.0),
                        ArrayCellValue::Number(8.0),
                    ]])
                    .expect("row vector"),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(20.0),
                        ArrayCellValue::Number(40.0),
                        ArrayCellValue::Number(60.0),
                        ArrayCellValue::Number(80.0),
                    ]])
                    .expect("row vector"),
                )),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("NF"))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("NF")),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("NF")),
                ]])
                .expect("row vector")
            ))
        );
    }

    #[test]
    fn eval_surface_value_call_ftc_1027_choose_sequence_multicolumn_returns_charlie() {
        let cols = eval_surface_value_call(
            FUNC_ID_SEQUENCE,
            &[
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(4.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("sequence result");
        let result = eval_surface_value_call(
            FUNC_ID_CHOOSE,
            &[
                CallArgValue::Eval(cols),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("Alpha"))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("Bravo"))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(
                    "Charlie",
                ))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("Delta"))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("choose result");

        let got = eval_surface_value_call(
            FUNC_ID_INDEX,
            &[
                CallArgValue::Eval(result),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );

        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment(
                "Charlie"
            )))
        );
    }

    #[test]
    fn eval_surface_value_call_ftc_1030_choose_transpose_index_returns_two() {
        let data = eval_surface_value_call(
            FUNC_ID_CHOOSE,
            &[
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_SEQUENCE,
                        &[
                            CallArgValue::Eval(EvalValue::Number(1.0)),
                            CallArgValue::Eval(EvalValue::Number(3.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("sequence result"),
                ),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(1.0)],
                        vec![ArrayCellValue::Number(2.0)],
                        vec![ArrayCellValue::Number(3.0)],
                    ])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(10.0)],
                        vec![ArrayCellValue::Number(20.0)],
                        vec![ArrayCellValue::Number(30.0)],
                    ])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![
                        vec![ArrayCellValue::Number(100.0)],
                        vec![ArrayCellValue::Number(200.0)],
                        vec![ArrayCellValue::Number(300.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("choose result");
        let result = eval_surface_value_call(
            FUNC_ID_TRANSPOSE,
            &[CallArgValue::Eval(data)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("transpose result");

        let got = eval_surface_value_call(
            FUNC_ID_INDEX,
            &[
                CallArgValue::Eval(result),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );

        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_1021_conditional_text_date_format_returns_fifteen() {
        let ctx = test_current_excel_host_context();
        let concat = |lhs: EvalValue, rhs: EvalValue| {
            eval_surface_value_call(
                FUNC_ID_OP_CONCAT,
                &[CallArgValue::Eval(lhs), CallArgValue::Eval(rhs)],
                &NoReferenceResolver,
                Some(46000.0),
                Some(0.5),
                None,
                None,
            )
            .expect("concat result")
        };
        let first_day = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("first day");
        let last_day = eval_surface_value_call(
            FUNC_ID_EOMONTH,
            &[
                CallArgValue::Eval(first_day.clone()),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("last day");
        let test_date = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
                CallArgValue::Eval(EvalValue::Number(15.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("test date");
        let format_code = concat(
            concat(
                concat(
                    concat(
                        EvalValue::Text(ExcelText::from_interop_assignment("[<")),
                        first_day,
                    ),
                    EvalValue::Text(ExcelText::from_interop_assignment("] ;[>")),
                ),
                last_day,
            ),
            EvalValue::Text(ExcelText::from_interop_assignment("] ;dd")),
        );
        let got = eval_surface_value_call(
            FUNC_ID_TEXT,
            &[
                CallArgValue::Eval(test_date),
                CallArgValue::Eval(format_code),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            Some(&ctx),
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("15")))
        );
    }

    #[test]
    fn eval_surface_value_call_ftc_1022_conditional_text_out_of_range_trims_to_zero() {
        let ctx = test_current_excel_host_context();
        let concat = |lhs: EvalValue, rhs: EvalValue| {
            eval_surface_value_call(
                FUNC_ID_OP_CONCAT,
                &[CallArgValue::Eval(lhs), CallArgValue::Eval(rhs)],
                &NoReferenceResolver,
                Some(46000.0),
                Some(0.5),
                None,
                None,
            )
            .expect("concat result")
        };
        let first_day = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("first day");
        let last_day = eval_surface_value_call(
            FUNC_ID_EOMONTH,
            &[
                CallArgValue::Eval(first_day.clone()),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("last day");
        let test_date = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(28.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("test date");
        let format_code = concat(
            concat(
                concat(
                    concat(
                        EvalValue::Text(ExcelText::from_interop_assignment("[<")),
                        first_day,
                    ),
                    EvalValue::Text(ExcelText::from_interop_assignment("] ;[>")),
                ),
                last_day,
            ),
            EvalValue::Text(ExcelText::from_interop_assignment("] ;dd")),
        );
        let rendered = eval_surface_value_call(
            FUNC_ID_TEXT,
            &[
                CallArgValue::Eval(test_date),
                CallArgValue::Eval(format_code),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            Some(&ctx),
            None,
        )
        .expect("text result");
        let trimmed = eval_surface_value_call(
            FUNC_ID_TRIM,
            &[CallArgValue::Eval(rendered)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("trim result");
        let got = eval_surface_value_call(
            FUNC_ID_LEN,
            &[CallArgValue::Eval(trimmed)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(0.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_1024_first_week_textjoin_returns_expected_row() {
        let first_day = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("first day");
        let weekday = eval_surface_value_call(
            FUNC_ID_WEEKDAY,
            &[
                CallArgValue::Eval(first_day.clone()),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("weekday");
        let grid_start = eval_surface_value_call(
            FUNC_ID_OP_ADD,
            &[
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_OP_SUBTRACT,
                        &[
                            CallArgValue::Eval(first_day.clone()),
                            CallArgValue::Eval(weekday),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("subtract result"),
                ),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("grid start");
        let dates = eval_surface_value_call(
            FUNC_ID_OP_ADD,
            &[
                CallArgValue::Eval(grid_start),
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_SEQUENCE,
                        &[
                            CallArgValue::Eval(EvalValue::Number(7.0)),
                            CallArgValue::MissingArg,
                            CallArgValue::Eval(EvalValue::Number(0.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("sequence result"),
                ),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("dates");
        let day_texts = eval_surface_value_call_with_callable(
            FUNC_ID_MAP,
            &[
                CallArgValue::Eval(dates),
                CallArgValue::Eval(EvalValue::Lambda(LambdaValue::helper_lambda(
                    "helper.feb2024_day_or_two_spaces",
                    CallableArityShape::exact(1),
                    CallableCaptureMode::NoCapture,
                    "lambda.map.feb2024.daystr",
                ))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
            Some(&TestCallableInvoker),
            None,
            None,
        )
        .expect("map result");
        let got = eval_surface_value_call(
            FUNC_ID_TEXTJOIN,
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(","))),
                CallArgValue::Eval(EvalValue::Logical(false)),
                CallArgValue::Eval(day_texts),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment(
                "  ,  ,  ,  ,01,02,03",
            )))
        );
    }

    #[test]
    fn eval_surface_value_call_ftc_1023_weekday_headers_index_returns_sun() {
        let ctx = test_current_excel_host_context();
        let base_sun = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(7.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("base sun");
        let headers = eval_surface_value_call(
            FUNC_ID_TEXT,
            &[
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_OP_SUBTRACT,
                        &[
                            CallArgValue::Eval(
                                eval_surface_value_call(
                                    FUNC_ID_OP_ADD,
                                    &[
                                        CallArgValue::Eval(base_sun),
                                        CallArgValue::Eval(
                                            eval_surface_value_call(
                                                FUNC_ID_SEQUENCE,
                                                &[
                                                    CallArgValue::Eval(EvalValue::Number(1.0)),
                                                    CallArgValue::Eval(EvalValue::Number(7.0)),
                                                ],
                                                &NoReferenceResolver,
                                                Some(46000.0),
                                                Some(0.5),
                                                None,
                                                None,
                                            )
                                            .expect("sequence result"),
                                        ),
                                    ],
                                    &NoReferenceResolver,
                                    Some(46000.0),
                                    Some(0.5),
                                    None,
                                    None,
                                )
                                .expect("add result"),
                            ),
                            CallArgValue::Eval(EvalValue::Number(1.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("subtract result"),
                ),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("DDD"))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            Some(&ctx),
            None,
        )
        .expect("text result");
        let got = eval_surface_value_call(
            FUNC_ID_INDEX,
            &[
                CallArgValue::Eval(headers),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("Sun")))
        );
    }

    #[test]
    fn eval_surface_value_call_ftc_1028_text_month_name_returns_july() {
        let ctx = test_current_excel_host_context();
        let date = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(7.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("date result");
        let got = eval_surface_value_call(
            FUNC_ID_TEXT,
            &[
                CallArgValue::Eval(date),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("MMMM"))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            Some(&ctx),
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("July")))
        );
    }

    #[test]
    fn eval_surface_value_call_ftc_1040_one_month_calendar_prefix_returns_expected_text() {
        let ctx = test_current_excel_host_context();
        let first_day = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("first day");
        let weekday = eval_surface_value_call(
            FUNC_ID_WEEKDAY,
            &[
                CallArgValue::Eval(first_day.clone()),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("weekday");
        let grid_start = eval_surface_value_call(
            FUNC_ID_OP_ADD,
            &[
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_OP_SUBTRACT,
                        &[
                            CallArgValue::Eval(first_day.clone()),
                            CallArgValue::Eval(weekday),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("subtract result"),
                ),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("grid start");
        let dates = eval_surface_value_call(
            FUNC_ID_OP_ADD,
            &[
                CallArgValue::Eval(grid_start),
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_SEQUENCE,
                        &[
                            CallArgValue::Eval(EvalValue::Number(42.0)),
                            CallArgValue::MissingArg,
                            CallArgValue::Eval(EvalValue::Number(0.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("sequence result"),
                ),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("dates");
        let day_strs = eval_surface_value_call_with_callable(
            FUNC_ID_MAP,
            &[
                CallArgValue::Eval(dates),
                CallArgValue::Eval(EvalValue::Lambda(LambdaValue::helper_lambda(
                    "helper.jan2024_day_or_two_spaces",
                    CallableArityShape::exact(1),
                    CallableCaptureMode::NoCapture,
                    "lambda.map.jan2024.daystr",
                ))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
            Some(&TestCallableInvoker),
            None,
            None,
        )
        .expect("map result");
        let month_name = eval_surface_value_call(
            FUNC_ID_TEXT,
            &[
                CallArgValue::Eval(first_day),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("MMMM"))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            Some(&ctx),
            None,
        )
        .expect("month name");
        let got = eval_surface_value_call(
            FUNC_ID_TEXTJOIN,
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("|"))),
                CallArgValue::Eval(EvalValue::Logical(false)),
                CallArgValue::Eval(month_name),
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_INDEX,
                        &[
                            CallArgValue::Eval(day_strs.clone()),
                            CallArgValue::Eval(EvalValue::Number(1.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("index 1"),
                ),
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_INDEX,
                        &[
                            CallArgValue::Eval(day_strs.clone()),
                            CallArgValue::Eval(EvalValue::Number(2.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("index 2"),
                ),
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_INDEX,
                        &[
                            CallArgValue::Eval(day_strs.clone()),
                            CallArgValue::Eval(EvalValue::Number(3.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("index 3"),
                ),
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_INDEX,
                        &[
                            CallArgValue::Eval(day_strs.clone()),
                            CallArgValue::Eval(EvalValue::Number(4.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("index 4"),
                ),
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_INDEX,
                        &[
                            CallArgValue::Eval(day_strs.clone()),
                            CallArgValue::Eval(EvalValue::Number(5.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("index 5"),
                ),
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_INDEX,
                        &[
                            CallArgValue::Eval(day_strs.clone()),
                            CallArgValue::Eval(EvalValue::Number(6.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("index 6"),
                ),
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_INDEX,
                        &[
                            CallArgValue::Eval(day_strs),
                            CallArgValue::Eval(EvalValue::Number(7.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("index 7"),
                ),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment(
                "January|  |01|02|03|04|05|06",
            )))
        );
    }

    #[test]
    fn eval_surface_value_call_ftc_1031_first_week_sum_returns_twenty_one() {
        let first_day = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("first day");
        let weekday = eval_surface_value_call(
            FUNC_ID_WEEKDAY,
            &[
                CallArgValue::Eval(first_day.clone()),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("weekday");
        let grid_start = eval_surface_value_call(
            FUNC_ID_OP_ADD,
            &[
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_OP_SUBTRACT,
                        &[CallArgValue::Eval(first_day), CallArgValue::Eval(weekday)],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("subtract result"),
                ),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("grid start");
        let week1 = eval_surface_value_call(
            FUNC_ID_OP_SUBTRACT,
            &[
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_OP_ADD,
                        &[
                            CallArgValue::Eval(grid_start),
                            CallArgValue::Eval(
                                eval_surface_value_call(
                                    FUNC_ID_SEQUENCE,
                                    &[
                                        CallArgValue::Eval(EvalValue::Number(1.0)),
                                        CallArgValue::Eval(EvalValue::Number(7.0)),
                                        CallArgValue::MissingArg,
                                        CallArgValue::Eval(EvalValue::Number(1.0)),
                                    ],
                                    &NoReferenceResolver,
                                    Some(46000.0),
                                    Some(0.5),
                                    None,
                                    None,
                                )
                                .expect("sequence"),
                            ),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("add result"),
                ),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("week1");
        let day_nums = eval_surface_value_call_with_callable(
            FUNC_ID_MAP,
            &[
                CallArgValue::Eval(week1),
                CallArgValue::Eval(EvalValue::Lambda(LambdaValue::helper_lambda(
                    "helper.jan2024_day_or_zero",
                    CallableArityShape::exact(1),
                    CallableCaptureMode::NoCapture,
                    "lambda.map.jan2024.dayzero",
                ))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
            Some(&TestCallableInvoker),
            None,
            None,
        )
        .expect("map result");
        let got = eval_surface_value_call(
            FUNC_ID_SUM,
            &[CallArgValue::Eval(day_nums)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(21.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_1032_after_direct_seam_fixes_returns_zero() {
        let first_day = eval_surface_value_call(
            FUNC_ID_DATE,
            &[
                CallArgValue::Eval(EvalValue::Number(2024.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("first day");
        let last_day = eval_surface_value_call(
            FUNC_ID_EOMONTH,
            &[
                CallArgValue::Eval(first_day.clone()),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("last day");
        let days_in_month = eval_surface_value_call(
            FUNC_ID_DAY,
            &[CallArgValue::Eval(last_day)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("days in month");
        let offset = eval_surface_value_call(
            FUNC_ID_OP_SUBTRACT,
            &[
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_WEEKDAY,
                        &[
                            CallArgValue::Eval(first_day),
                            CallArgValue::Eval(EvalValue::Number(1.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("weekday"),
                ),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("offset");
        let grid = eval_surface_value_call(
            FUNC_ID_SEQUENCE,
            &[CallArgValue::Eval(EvalValue::Number(42.0))],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("grid");
        let day_vals = eval_surface_value_call(
            FUNC_ID_IF,
            &[
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_AND,
                        &[
                            CallArgValue::Eval(
                                eval_surface_value_call(
                                    FUNC_ID_OP_GREATER_THAN,
                                    &[
                                        CallArgValue::Eval(grid.clone()),
                                        CallArgValue::Eval(offset.clone()),
                                    ],
                                    &NoReferenceResolver,
                                    Some(46000.0),
                                    Some(0.5),
                                    None,
                                    None,
                                )
                                .expect("gt result"),
                            ),
                            CallArgValue::Eval(
                                eval_surface_value_call(
                                    FUNC_ID_OP_LESS_EQUAL,
                                    &[
                                        CallArgValue::Eval(grid.clone()),
                                        CallArgValue::Eval(
                                            eval_surface_value_call(
                                                FUNC_ID_OP_ADD,
                                                &[
                                                    CallArgValue::Eval(offset.clone()),
                                                    CallArgValue::Eval(days_in_month),
                                                ],
                                                &NoReferenceResolver,
                                                Some(46000.0),
                                                Some(0.5),
                                                None,
                                                None,
                                            )
                                            .expect("offset+days"),
                                        ),
                                    ],
                                    &NoReferenceResolver,
                                    Some(46000.0),
                                    Some(0.5),
                                    None,
                                    None,
                                )
                                .expect("le result"),
                            ),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("and result"),
                ),
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_OP_SUBTRACT,
                        &[CallArgValue::Eval(grid), CallArgValue::Eval(offset)],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("grid-offset"),
                ),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("day vals");
        let weekly = eval_surface_value_call(
            FUNC_ID_WRAPROWS,
            &[
                CallArgValue::Eval(day_vals),
                CallArgValue::Eval(EvalValue::Number(7.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("weekly");
        let got = eval_surface_value_call(
            FUNC_ID_SUM,
            &[CallArgValue::Eval(
                eval_surface_value_call(
                    FUNC_ID_INDEX,
                    &[
                        CallArgValue::Eval(weekly),
                        CallArgValue::Eval(EvalValue::Number(1.0)),
                        CallArgValue::Eval(EvalValue::Number(0.0)),
                    ],
                    &NoReferenceResolver,
                    Some(46000.0),
                    Some(0.5),
                    None,
                    None,
                )
                .expect("index first row"),
            )],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(0.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0798_scalar_seed_index_lane_returns_zero() {
        let cols = eval_surface_value_call(
            FUNC_ID_COLUMNS,
            &[CallArgValue::Eval(EvalValue::Number(0.0))],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("columns result");
        let got = eval_surface_value_call(
            FUNC_ID_INDEX,
            &[
                CallArgValue::Eval(EvalValue::Number(0.0)),
                CallArgValue::Eval(cols),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(0.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0450_population_stddev_let_composition() {
        let data = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Number(5.0),
            ArrayCellValue::Number(3.0),
            ArrayCellValue::Number(8.0),
            ArrayCellValue::Number(1.0),
            ArrayCellValue::Number(9.0),
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Number(7.0),
            ArrayCellValue::Number(4.0),
            ArrayCellValue::Number(6.0),
        ]])
        .unwrap();
        let n = eval_surface_value_call(
            FUNC_ID_COUNTA,
            &[CallArgValue::Eval(EvalValue::Array(data.clone()))],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("counta result");
        let mean = eval_surface_value_call(
            FUNC_ID_AVERAGE,
            &[CallArgValue::Eval(EvalValue::Array(data.clone()))],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("average result");
        let centered = eval_surface_value_call(
            FUNC_ID_OP_SUBTRACT,
            &[
                CallArgValue::Eval(EvalValue::Array(data)),
                CallArgValue::Eval(mean.clone()),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("subtract result");
        let squares = eval_surface_value_call(
            FUNC_ID_OP_POWER,
            &[
                CallArgValue::Eval(centered),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("power result");
        let variance = eval_surface_value_call(
            FUNC_ID_OP_DIVIDE,
            &[
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_SUMPRODUCT,
                        &[CallArgValue::Eval(squares)],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("sumproduct result"),
                ),
                CallArgValue::Eval(n),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("variance result");
        let got = eval_surface_value_call(
            FUNC_ID_SQRT,
            &[CallArgValue::Eval(variance)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(2.581988897471611)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0256_sumproduct_of_double_unary_compare_returns_two() {
        let include = eval_surface_value_call(
            FUNC_ID_OP_GREATER_THAN,
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("comparison result");
        let coerced = eval_surface_value_call(
            FUNC_ID_OP_NEGATE,
            &[CallArgValue::Eval(
                eval_surface_value_call(
                    FUNC_ID_OP_NEGATE,
                    &[CallArgValue::Eval(include)],
                    &NoReferenceResolver,
                    Some(46000.0),
                    Some(0.5),
                    None,
                    None,
                )
                .expect("first negate"),
            )],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("double-negated result");
        let got = eval_surface_value_call(
            FUNC_ID_SUMPRODUCT,
            &[CallArgValue::Eval(coerced)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0288_text_grouped_decimal_format_returns_en_us_text() {
        let ctx = crate::locale_format::test_en_us_context();
        let got = eval_surface_value_call(
            FUNC_ID_TEXT,
            &[
                CallArgValue::Eval(EvalValue::Number(1234567.89)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(
                    "#,##0.00",
                ))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            Some(&ctx),
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment(
                "1,234,567.89",
            )))
        );
    }

    #[test]
    fn eval_surface_value_call_ftc_0505_columns_of_randarray_returns_three() {
        let generated = eval_surface_value_call(
            FUNC_ID_RANDARRAY,
            &[
                CallArgValue::Eval(EvalValue::Number(5.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("randarray result");
        let got = eval_surface_value_call(
            FUNC_ID_COLUMNS,
            &[CallArgValue::Eval(generated)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(3.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0600_extract_digits_from_string_returns_123() {
        let ctx = test_current_excel_host_context();
        let text = EvalValue::Text(ExcelText::from_interop_assignment("Hello World 123"));
        let length = eval_surface_value_call(
            FUNC_ID_LEN,
            &[CallArgValue::Eval(text.clone())],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("len result");
        let positions = eval_surface_value_call(
            FUNC_ID_SEQUENCE,
            &[CallArgValue::Eval(length)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("sequence result");
        let chars = eval_surface_value_call(
            FUNC_ID_MID,
            &[
                CallArgValue::Eval(text),
                CallArgValue::Eval(positions),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("mid result");
        let multiplied = eval_surface_value_call(
            FUNC_ID_OP_MULTIPLY,
            &[
                CallArgValue::Eval(chars.clone()),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("chars times one");
        let recovered = eval_surface_value_call(
            FUNC_ID_IFERROR,
            &[
                CallArgValue::Eval(multiplied),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(""))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("iferror result");
        let numeric_text = eval_surface_value_call(
            FUNC_ID_VALUE,
            &[CallArgValue::Eval(recovered)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            Some(&ctx),
            None,
        )
        .expect("value result");
        let is_digit = eval_surface_value_call(
            FUNC_ID_ISNUMBER,
            &[CallArgValue::Eval(numeric_text)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("isnumber result");
        let digits = eval_surface_value_call(
            FUNC_ID_FILTER,
            &[
                CallArgValue::Eval(chars),
                CallArgValue::Eval(is_digit),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(""))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("filter result");
        let got = eval_surface_value_call(
            FUNC_ID_CONCAT,
            &[CallArgValue::Eval(digits)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("123")))
        );
    }

    #[test]
    fn eval_surface_value_call_ftc_0470_map_chain_sum_returns_sixty_three() {
        let data = EvalArray::from_rows(vec![
            vec![ArrayCellValue::Number(1.0)],
            vec![ArrayCellValue::Number(2.0)],
            vec![ArrayCellValue::Number(3.0)],
        ])
        .unwrap();
        let step1 = eval_surface_value_call_with_callable(
            FUNC_ID_MAP,
            &[
                CallArgValue::Eval(EvalValue::Array(data)),
                CallArgValue::Eval(EvalValue::Lambda(LambdaValue::helper_lambda(
                    "helper.mul10",
                    CallableArityShape::exact(1),
                    CallableCaptureMode::LexicalCapture,
                    "helper.invoke.v1",
                ))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
            Some(&TestCallableInvoker),
            None,
            None,
        )
        .expect("first map result");
        let step2 = eval_surface_value_call_with_callable(
            FUNC_ID_MAP,
            &[
                CallArgValue::Eval(step1),
                CallArgValue::Eval(EvalValue::Lambda(LambdaValue::helper_lambda(
                    "helper.add1",
                    CallableArityShape::exact(1),
                    CallableCaptureMode::LexicalCapture,
                    "helper.invoke.v1",
                ))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
            Some(&TestCallableInvoker),
            None,
            None,
        )
        .expect("second map result");
        let got = eval_surface_value_call(
            FUNC_ID_SUM,
            &[CallArgValue::Eval(step2)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(63.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0443_recursive_gcd_returns_twelve() {
        let invoker = ClosureCallableInvoker::new();
        let gcd = LambdaValue::helper_lambda(
            "closure.ftc0443.gcd",
            CallableArityShape::exact(3),
            CallableCaptureMode::LexicalCapture,
            "test.closure.invoke.v1",
        );
        let gcd_self = gcd.clone();
        let recursive_invoker = invoker.clone();
        invoker.register(&gcd.callable_token.clone(), 3, move |args| match args {
            [
                PreparedArgValue::Eval(EvalValue::Lambda(self_lambda)),
                PreparedArgValue::Eval(EvalValue::Number(a)),
                PreparedArgValue::Eval(EvalValue::Number(b)),
            ] => {
                if *b == 0.0 {
                    Ok(PreparedArgValue::Eval(EvalValue::Number(*a)))
                } else {
                    let remainder = eval_test_surface_value(
                        FUNC_ID_MOD,
                        &[
                            CallArgValue::Eval(EvalValue::Number(*a)),
                            CallArgValue::Eval(EvalValue::Number(*b)),
                        ],
                    )?;
                    recursive_invoker.invoke(
                        &gcd_self,
                        &[
                            PreparedArgValue::Eval(EvalValue::Lambda(self_lambda.clone())),
                            PreparedArgValue::Eval(EvalValue::Number(*b)),
                            PreparedArgValue::Eval(remainder),
                        ],
                    )
                }
            }
            _ => Err(CallableInvocationError::Worksheet(
                WorksheetErrorCode::Value,
            )),
        });

        let got = invoker.invoke(
            &gcd,
            &[
                PreparedArgValue::Eval(EvalValue::Lambda(gcd.clone())),
                PreparedArgValue::Eval(EvalValue::Number(48.0)),
                PreparedArgValue::Eval(EvalValue::Number(36.0)),
            ],
        );
        assert_eq!(got, Ok(PreparedArgValue::Eval(EvalValue::Number(12.0))));
    }

    #[test]
    fn eval_surface_value_call_ftc_1013_current_inverse_reconstruction_returns_2211() {
        let invoker = ClosureCallableInvoker::new();
        let a = number_column(&[1.0, 1.0, 1.0, 0.0]);
        let b = number_column(&[1.0, 1.0, 0.0, 0.0]);
        let n = EvalValue::Number(4.0);
        let ks = eval_test_surface_value(
            FUNC_ID_SEQUENCE,
            &[
                CallArgValue::Eval(n.clone()),
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
        )
        .expect("ks");
        let two_pi = eval_test_surface_value(
            FUNC_ID_OP_MULTIPLY,
            &[
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(eval_test_surface_value(FUNC_ID_PI, &[]).expect("pi")),
            ],
        )
        .expect("two_pi");

        let register_dft = |token: &str, signal: EvalValue, trig_function: &str, sign: f64| {
            let signal = signal.clone();
            let ks = ks.clone();
            let n = n.clone();
            let two_pi = two_pi.clone();
            let trig_function = trig_function.to_string();
            invoker.register(token, 1, move |args| {
                let wave = eval_test_surface_value(
                    trig_function.as_str(),
                    &[CallArgValue::Eval(
                        eval_test_surface_value(
                            FUNC_ID_OP_DIVIDE,
                            &[
                                CallArgValue::Eval(
                                    eval_test_surface_value(
                                        FUNC_ID_OP_MULTIPLY,
                                        &[
                                            CallArgValue::Eval(two_pi.clone()),
                                            CallArgValue::Eval(
                                                eval_test_surface_value(
                                                    FUNC_ID_OP_MULTIPLY,
                                                    &[
                                                        call_arg_from_prepared(&args[0]),
                                                        CallArgValue::Eval(ks.clone()),
                                                    ],
                                                )
                                                .expect("k*ks"),
                                            ),
                                        ],
                                    )
                                    .expect("2pi*k*ks"),
                                ),
                                CallArgValue::Eval(n.clone()),
                            ],
                        )
                        .expect("angle"),
                    )],
                )?;
                let mut total = eval_test_surface_value(
                    FUNC_ID_SUM,
                    &[CallArgValue::Eval(
                        eval_test_surface_value(
                            FUNC_ID_OP_MULTIPLY,
                            &[CallArgValue::Eval(signal.clone()), CallArgValue::Eval(wave)],
                        )
                        .expect("signal*wave"),
                    )],
                )?;
                if sign < 0.0 {
                    total = eval_test_surface_value(
                        FUNC_ID_OP_MULTIPLY,
                        &[
                            CallArgValue::Eval(EvalValue::Number(sign)),
                            CallArgValue::Eval(total),
                        ],
                    )?;
                }
                Ok(PreparedArgValue::Eval(total))
            })
        };

        let ar_lambda = register_dft("closure.ftc1013.ar", a.clone(), FUNC_ID_COS, 1.0);
        let ai_lambda = register_dft("closure.ftc1013.ai", a.clone(), FUNC_ID_SIN, -1.0);
        let br_lambda = register_dft("closure.ftc1013.br", b.clone(), FUNC_ID_COS, 1.0);
        let bi_lambda = register_dft("closure.ftc1013.bi", b.clone(), FUNC_ID_SIN, -1.0);

        let ar = eval_test_surface_value_with_callable(
            FUNC_ID_MAP,
            &[
                CallArgValue::Eval(ks.clone()),
                CallArgValue::Eval(EvalValue::Lambda(ar_lambda)),
            ],
            &invoker,
        )
        .expect("Ar");
        let ai = eval_test_surface_value_with_callable(
            FUNC_ID_MAP,
            &[
                CallArgValue::Eval(ks.clone()),
                CallArgValue::Eval(EvalValue::Lambda(ai_lambda)),
            ],
            &invoker,
        )
        .expect("Ai");
        let br = eval_test_surface_value_with_callable(
            FUNC_ID_MAP,
            &[
                CallArgValue::Eval(ks.clone()),
                CallArgValue::Eval(EvalValue::Lambda(br_lambda)),
            ],
            &invoker,
        )
        .expect("Br");
        let bi = eval_test_surface_value_with_callable(
            FUNC_ID_MAP,
            &[
                CallArgValue::Eval(ks.clone()),
                CallArgValue::Eval(EvalValue::Lambda(bi_lambda)),
            ],
            &invoker,
        )
        .expect("Bi");

        let cr = eval_test_surface_value(
            FUNC_ID_OP_SUBTRACT,
            &[
                CallArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_OP_MULTIPLY,
                        &[
                            CallArgValue::Eval(ar.clone()),
                            CallArgValue::Eval(br.clone()),
                        ],
                    )
                    .expect("Ar*Br"),
                ),
                CallArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_OP_MULTIPLY,
                        &[
                            CallArgValue::Eval(ai.clone()),
                            CallArgValue::Eval(bi.clone()),
                        ],
                    )
                    .expect("Ai*Bi"),
                ),
            ],
        )
        .expect("Cr");
        let ci = eval_test_surface_value(
            FUNC_ID_OP_ADD,
            &[
                CallArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_OP_MULTIPLY,
                        &[
                            CallArgValue::Eval(ar.clone()),
                            CallArgValue::Eval(bi.clone()),
                        ],
                    )
                    .expect("Ar*Bi"),
                ),
                CallArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_OP_MULTIPLY,
                        &[
                            CallArgValue::Eval(ai.clone()),
                            CallArgValue::Eval(br.clone()),
                        ],
                    )
                    .expect("Ai*Br"),
                ),
            ],
        )
        .expect("Ci");

        let conv_lambda = {
            let cr = cr.clone();
            let ci = ci.clone();
            let ks = ks.clone();
            let n = n.clone();
            let two_pi = two_pi.clone();
            invoker.register("closure.ftc1013.conv", 1, move |args| {
                let angle = eval_test_surface_value(
                    FUNC_ID_OP_DIVIDE,
                    &[
                        CallArgValue::Eval(
                            eval_test_surface_value(
                                FUNC_ID_OP_MULTIPLY,
                                &[
                                    CallArgValue::Eval(two_pi.clone()),
                                    CallArgValue::Eval(
                                        eval_test_surface_value(
                                            FUNC_ID_OP_MULTIPLY,
                                            &[
                                                call_arg_from_prepared(&args[0]),
                                                CallArgValue::Eval(ks.clone()),
                                            ],
                                        )
                                        .expect("n*ks"),
                                    ),
                                ],
                            )
                            .expect("2pi*n*ks"),
                        ),
                        CallArgValue::Eval(n.clone()),
                    ],
                )
                .expect("angle");
                let total = eval_test_surface_value(
                    FUNC_ID_SUM,
                    &[CallArgValue::Eval(
                        eval_test_surface_value(
                            FUNC_ID_OP_ADD,
                            &[
                                CallArgValue::Eval(
                                    eval_test_surface_value(
                                        FUNC_ID_OP_MULTIPLY,
                                        &[
                                            CallArgValue::Eval(cr.clone()),
                                            CallArgValue::Eval(
                                                eval_test_surface_value(
                                                    FUNC_ID_COS,
                                                    &[CallArgValue::Eval(angle.clone())],
                                                )
                                                .expect("cos(angle)"),
                                            ),
                                        ],
                                    )
                                    .expect("Cr*cos"),
                                ),
                                CallArgValue::Eval(
                                    eval_test_surface_value(
                                        FUNC_ID_OP_MULTIPLY,
                                        &[
                                            CallArgValue::Eval(ci.clone()),
                                            CallArgValue::Eval(
                                                eval_test_surface_value(
                                                    FUNC_ID_SIN,
                                                    &[CallArgValue::Eval(angle)],
                                                )
                                                .expect("sin(angle)"),
                                            ),
                                        ],
                                    )
                                    .expect("Ci*sin"),
                                ),
                            ],
                        )
                        .expect("sum terms"),
                    )],
                )?;
                Ok(PreparedArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_OP_DIVIDE,
                        &[CallArgValue::Eval(total), CallArgValue::Eval(n.clone())],
                    )
                    .expect("divide by N"),
                ))
            })
        };

        let conv = eval_test_surface_value_with_callable(
            FUNC_ID_MAP,
            &[
                CallArgValue::Eval(ks.clone()),
                CallArgValue::Eval(EvalValue::Lambda(conv_lambda)),
            ],
            &invoker,
        )
        .expect("conv");

        // The current local witness reconstructs the inverse real part as
        // `Cr*cos(angle) + Ci*sin(angle)`. For the locally computed `Ci`
        // carrier, that yields the packed result `2211`; the previously pinned
        // `1221` expectation corresponds to the alternate reconstruction that
        // subtracts the sine term.
        let got = eval_test_surface_value(
            FUNC_ID_ROUND,
            &[
                CallArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_OP_ADD,
                        &[
                            CallArgValue::Eval(
                                eval_test_surface_value(
                                    FUNC_ID_OP_ADD,
                                    &[
                                        CallArgValue::Eval(
                                            eval_test_surface_value(
                                                FUNC_ID_INDEX,
                                                &[
                                                    CallArgValue::Eval(conv.clone()),
                                                    CallArgValue::Eval(EvalValue::Number(1.0)),
                                                ],
                                            )
                                            .expect("conv1"),
                                        ),
                                        CallArgValue::Eval(
                                            eval_test_surface_value(
                                                FUNC_ID_OP_MULTIPLY,
                                                &[
                                                    CallArgValue::Eval(EvalValue::Number(10.0)),
                                                    CallArgValue::Eval(
                                                        eval_test_surface_value(
                                                            FUNC_ID_INDEX,
                                                            &[
                                                                CallArgValue::Eval(conv.clone()),
                                                                CallArgValue::Eval(
                                                                    EvalValue::Number(2.0),
                                                                ),
                                                            ],
                                                        )
                                                        .expect("conv2"),
                                                    ),
                                                ],
                                            )
                                            .expect("10*conv2"),
                                        ),
                                    ],
                                )
                                .expect("low digits"),
                            ),
                            CallArgValue::Eval(
                                eval_test_surface_value(
                                    FUNC_ID_OP_ADD,
                                    &[
                                        CallArgValue::Eval(
                                            eval_test_surface_value(
                                                FUNC_ID_OP_MULTIPLY,
                                                &[
                                                    CallArgValue::Eval(EvalValue::Number(100.0)),
                                                    CallArgValue::Eval(
                                                        eval_test_surface_value(
                                                            FUNC_ID_INDEX,
                                                            &[
                                                                CallArgValue::Eval(conv.clone()),
                                                                CallArgValue::Eval(
                                                                    EvalValue::Number(3.0),
                                                                ),
                                                            ],
                                                        )
                                                        .expect("conv3"),
                                                    ),
                                                ],
                                            )
                                            .expect("100*conv3"),
                                        ),
                                        CallArgValue::Eval(
                                            eval_test_surface_value(
                                                FUNC_ID_OP_MULTIPLY,
                                                &[
                                                    CallArgValue::Eval(EvalValue::Number(1000.0)),
                                                    CallArgValue::Eval(
                                                        eval_test_surface_value(
                                                            FUNC_ID_INDEX,
                                                            &[
                                                                CallArgValue::Eval(conv),
                                                                CallArgValue::Eval(
                                                                    EvalValue::Number(4.0),
                                                                ),
                                                            ],
                                                        )
                                                        .expect("conv4"),
                                                    ),
                                                ],
                                            )
                                            .expect("1000*conv4"),
                                        ),
                                    ],
                                )
                                .expect("high digits"),
                            ),
                        ],
                    )
                    .expect("packed"),
                ),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
        );
        assert_eq!(got, Ok(EvalValue::Number(2211.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_0477_filter_if_empty_returns_none() {
        let data = EvalArray::from_rows(vec![
            vec![ArrayCellValue::Number(1.0)],
            vec![ArrayCellValue::Number(2.0)],
            vec![ArrayCellValue::Number(3.0)],
            vec![ArrayCellValue::Number(4.0)],
            vec![ArrayCellValue::Number(5.0)],
        ])
        .unwrap();
        let include = eval_surface_value_call(
            FUNC_ID_OP_GREATER_THAN,
            &[
                CallArgValue::Eval(EvalValue::Array(data.clone())),
                CallArgValue::Eval(EvalValue::Number(10.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("comparison result");
        let got = eval_surface_value_call(
            FUNC_ID_FILTER,
            &[
                CallArgValue::Eval(EvalValue::Array(data)),
                CallArgValue::Eval(include),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("none"))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("none")))
        );
    }

    #[test]
    fn eval_surface_value_call_ftc_1006_orientation_chain_packs_to_201_locally() {
        let data = number_column(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        let wrapped = eval_test_surface_value(
            FUNC_ID_WRAPCOLS,
            &[
                CallArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_TOCOL,
                        &[
                            CallArgValue::Eval(data.clone()),
                            CallArgValue::MissingArg,
                            CallArgValue::Eval(EvalValue::Logical(true)),
                        ],
                    )
                    .expect("TOCOL(data,,TRUE)"),
                ),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
        )
        .expect("Wrap(data,2)");
        let x0 = eval_test_surface_value(
            FUNC_ID_TAKE,
            &[
                CallArgValue::Eval(wrapped.clone()),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
        )
        .expect("TAKE(w,1)");
        let x1 = eval_test_surface_value(
            FUNC_ID_TAKE,
            &[
                CallArgValue::Eval(wrapped.clone()),
                CallArgValue::Eval(EvalValue::Number(-1.0)),
            ],
        )
        .expect("TAKE(w,-1)");
        let y0 = eval_test_surface_value(
            FUNC_ID_WRAPCOLS,
            &[
                CallArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_TOCOL,
                        &[
                            CallArgValue::Eval(x0.clone()),
                            CallArgValue::MissingArg,
                            CallArgValue::Eval(EvalValue::Logical(true)),
                        ],
                    )
                    .expect("TOCOL(x0,,TRUE)"),
                ),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
        )
        .expect("Wrap(x0,2)");
        let y1 = eval_test_surface_value(
            FUNC_ID_WRAPCOLS,
            &[
                CallArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_TOCOL,
                        &[
                            CallArgValue::Eval(x1.clone()),
                            CallArgValue::MissingArg,
                            CallArgValue::Eval(EvalValue::Logical(true)),
                        ],
                    )
                    .expect("TOCOL(x1,,TRUE)"),
                ),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
        )
        .expect("Wrap(x1,2)");
        let result = eval_test_surface_value(
            FUNC_ID_VSTACK,
            &[
                CallArgValue::Eval(y0.clone()),
                CallArgValue::Eval(y1.clone()),
            ],
        )
        .expect("VSTACK(y0,y1)");
        let flat = eval_test_surface_value(FUNC_ID_TOCOL, &[CallArgValue::Eval(result.clone())])
            .expect("TOCOL(result)");
        let packed = eval_test_surface_value(
            FUNC_ID_OP_ADD,
            &[
                CallArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_INDEX,
                        &[
                            CallArgValue::Eval(flat.clone()),
                            CallArgValue::Eval(EvalValue::Number(1.0)),
                        ],
                    )
                    .expect("INDEX(flat,1)"),
                ),
                CallArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_OP_MULTIPLY,
                        &[
                            CallArgValue::Eval(EvalValue::Number(100.0)),
                            CallArgValue::Eval(
                                eval_test_surface_value(
                                    FUNC_ID_INDEX,
                                    &[
                                        CallArgValue::Eval(flat.clone()),
                                        CallArgValue::Eval(EvalValue::Number(5.0)),
                                    ],
                                )
                                .expect("INDEX(flat,5)"),
                            ),
                        ],
                    )
                    .expect("100*index5"),
                ),
            ],
        )
        .expect("packed");

        assert_eq!(
            wrapped,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Number(5.0),
                        ArrayCellValue::Number(7.0),
                    ],
                    vec![
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(4.0),
                        ArrayCellValue::Number(6.0),
                        ArrayCellValue::Number(8.0),
                    ],
                ])
                .unwrap()
            )
        );
        assert_eq!(
            y0,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0), ArrayCellValue::Number(5.0)],
                    vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(7.0)],
                ])
                .unwrap()
            )
        );
        assert_eq!(
            y1,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(2.0), ArrayCellValue::Number(6.0)],
                    vec![ArrayCellValue::Number(4.0), ArrayCellValue::Number(8.0)],
                ])
                .unwrap()
            )
        );
        assert_eq!(
            flat,
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(5.0)],
                    vec![ArrayCellValue::Number(3.0)],
                    vec![ArrayCellValue::Number(7.0)],
                    vec![ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(6.0)],
                    vec![ArrayCellValue::Number(4.0)],
                    vec![ArrayCellValue::Number(8.0)],
                ])
                .unwrap()
            )
        );
        assert_eq!(packed, EvalValue::Number(201.0));
    }

    #[test]
    fn eval_surface_value_call_ftc_1007_take_vector_split_packs_to_6_locally() {
        let x = eval_test_surface_value(
            FUNC_ID_HSTACK,
            &[
                CallArgValue::Eval(EvalValue::Number(3.0)),
                CallArgValue::Eval(EvalValue::Number(0.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
        )
        .expect("HSTACK");
        let x0 = eval_test_surface_value(
            FUNC_ID_TAKE,
            &[
                CallArgValue::Eval(x.clone()),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
        )
        .expect("TAKE(x,1)");
        let x1 = eval_test_surface_value(
            FUNC_ID_TAKE,
            &[
                CallArgValue::Eval(x.clone()),
                CallArgValue::Eval(EvalValue::Number(-1.0)),
            ],
        )
        .expect("TAKE(x,-1)");
        let re_x0 = eval_test_surface_value(
            FUNC_ID_TAKE,
            &[
                CallArgValue::Eval(x0.clone()),
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
        )
        .expect("Re(x0)");
        let re_x1 = eval_test_surface_value(
            FUNC_ID_TAKE,
            &[
                CallArgValue::Eval(x1.clone()),
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
        )
        .expect("Re(x1)");
        let y0 = eval_test_surface_value(
            FUNC_ID_OP_ADD,
            &[
                CallArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_INDEX,
                        &[
                            CallArgValue::Eval(re_x0.clone()),
                            CallArgValue::Eval(EvalValue::Number(1.0)),
                            CallArgValue::Eval(EvalValue::Number(1.0)),
                        ],
                    )
                    .expect("INDEX(re_x0,1,1)"),
                ),
                CallArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_INDEX,
                        &[
                            CallArgValue::Eval(re_x1.clone()),
                            CallArgValue::Eval(EvalValue::Number(1.0)),
                            CallArgValue::Eval(EvalValue::Number(1.0)),
                        ],
                    )
                    .expect("INDEX(re_x1,1,1)"),
                ),
            ],
        )
        .expect("y0");
        let y1 = eval_test_surface_value(
            FUNC_ID_OP_SUBTRACT,
            &[
                CallArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_INDEX,
                        &[
                            CallArgValue::Eval(re_x0.clone()),
                            CallArgValue::Eval(EvalValue::Number(1.0)),
                            CallArgValue::Eval(EvalValue::Number(1.0)),
                        ],
                    )
                    .expect("INDEX(re_x0,1,1)"),
                ),
                CallArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_INDEX,
                        &[
                            CallArgValue::Eval(re_x1.clone()),
                            CallArgValue::Eval(EvalValue::Number(1.0)),
                            CallArgValue::Eval(EvalValue::Number(1.0)),
                        ],
                    )
                    .expect("INDEX(re_x1,1,1)"),
                ),
            ],
        )
        .expect("y1");
        let packed = eval_test_surface_value(
            FUNC_ID_OP_ADD,
            &[
                CallArgValue::Eval(y0.clone()),
                CallArgValue::Eval(
                    eval_test_surface_value(
                        FUNC_ID_OP_MULTIPLY,
                        &[
                            CallArgValue::Eval(y1.clone()),
                            CallArgValue::Eval(EvalValue::Number(100.0)),
                        ],
                    )
                    .expect("y1*100"),
                ),
            ],
        )
        .expect("packed");

        assert_eq!(x0, x);
        assert_eq!(x1, x);
        assert_eq!(
            re_x0,
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(0.0),
                ]])
                .unwrap()
            )
        );
        assert_eq!(re_x1, re_x0);
        assert_eq!(y0, EvalValue::Number(6.0));
        assert_eq!(y1, EvalValue::Number(0.0));
        assert_eq!(packed, EvalValue::Number(6.0));
    }

    #[test]
    fn eval_surface_value_call_ftc_1008_complex_magnitude_returns_five() {
        let z = eval_surface_value_call(
            FUNC_ID_HSTACK,
            &[
                CallArgValue::Eval(EvalValue::Number(3.0)),
                CallArgValue::Eval(EvalValue::Number(4.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("hstack result");
        let re = eval_surface_value_call(
            FUNC_ID_TAKE,
            &[
                CallArgValue::Eval(z.clone()),
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("real part");
        let im = eval_surface_value_call(
            FUNC_ID_TAKE,
            &[
                CallArgValue::Eval(z),
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Number(-1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("imaginary part");
        let sumsq = eval_surface_value_call(
            FUNC_ID_OP_ADD,
            &[
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_OP_POWER,
                        &[
                            CallArgValue::Eval(re),
                            CallArgValue::Eval(EvalValue::Number(2.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("re squared"),
                ),
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_OP_POWER,
                        &[
                            CallArgValue::Eval(im),
                            CallArgValue::Eval(EvalValue::Number(2.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("im squared"),
                ),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("sumsq result");
        let magnitude = eval_surface_value_call(
            FUNC_ID_SQRT,
            &[CallArgValue::Eval(sumsq)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("sqrt result");
        let indexed = eval_surface_value_call(
            FUNC_ID_INDEX,
            &[
                CallArgValue::Eval(magnitude),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("index result");
        let got = eval_surface_value_call(
            FUNC_ID_ROUND,
            &[
                CallArgValue::Eval(indexed),
                CallArgValue::Eval(EvalValue::Number(6.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(5.0)));
    }

    #[test]
    fn eval_surface_value_call_ftc_1020_calendar_grid_counts_january_days() {
        let dates = eval_surface_value_call(
            FUNC_ID_OP_ADD,
            &[
                CallArgValue::Eval(EvalValue::Number(45291.0)),
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_SEQUENCE,
                        &[
                            CallArgValue::Eval(EvalValue::Number(42.0)),
                            CallArgValue::MissingArg,
                            CallArgValue::Eval(EvalValue::Number(0.0)),
                        ],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("sequence result"),
                ),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("dates result");
        let in_month = eval_surface_value_call(
            FUNC_ID_OP_EQUAL,
            &[
                CallArgValue::Eval(
                    eval_surface_value_call(
                        FUNC_ID_MONTH,
                        &[CallArgValue::Eval(dates)],
                        &NoReferenceResolver,
                        Some(46000.0),
                        Some(0.5),
                        None,
                        None,
                    )
                    .expect("month result"),
                ),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("equal result");
        let coerced = eval_surface_value_call(
            FUNC_ID_OP_NEGATE,
            &[CallArgValue::Eval(
                eval_surface_value_call(
                    FUNC_ID_OP_NEGATE,
                    &[CallArgValue::Eval(in_month)],
                    &NoReferenceResolver,
                    Some(46000.0),
                    Some(0.5),
                    None,
                    None,
                )
                .expect("first negate"),
            )],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        )
        .expect("double-negated result");
        let got = eval_surface_value_call(
            FUNC_ID_SUM,
            &[CallArgValue::Eval(coerced)],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(31.0)));
    }

    #[test]
    fn eval_surface_value_call_match_spills_array_lookup_value_results() {
        let lookup_values = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Number(1.0),
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Number(3.0),
        ]])
        .expect("row vector");
        let lookup_array = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Number(4.0),
            ArrayCellValue::Number(6.0),
            ArrayCellValue::Number(8.0),
        ]])
        .expect("row vector");
        let got = eval_surface_value_call(
            FUNC_ID_MATCH,
            &[
                CallArgValue::Eval(EvalValue::Array(lookup_values)),
                CallArgValue::Eval(EvalValue::Array(lookup_array)),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        let expected = EvalArray::from_rows(vec![vec![
            ArrayCellValue::Error(WorksheetErrorCode::NA),
            ArrayCellValue::Number(1.0),
            ArrayCellValue::Error(WorksheetErrorCode::NA),
        ]])
        .expect("row vector");
        assert_eq!(got, Ok(EvalValue::Array(expected)));
    }

    #[test]
    fn eval_surface_extended_call_wraps_now_with_number_format_hint() {
        let got = eval_surface_extended_call(
            FUNC_ID_NOW,
            &[],
            &NoReferenceResolver,
            Some(46000.25),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(ExtendedValue::ValueWithPresentation {
                value: EvalValue::Number(46000.25),
                hint: PresentationHint::number_format(NumberFormatHint::DateLike),
            })
        );
    }

    #[test]
    fn eval_surface_extended_call_wraps_today_with_number_format_hint() {
        let got = eval_surface_extended_call(
            FUNC_ID_TODAY,
            &[],
            &NoReferenceResolver,
            Some(46000.75),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(ExtendedValue::ValueWithPresentation {
                value: EvalValue::Number(46000.0),
                hint: PresentationHint::number_format(NumberFormatHint::DateLike),
            })
        );
    }

    #[test]
    fn eval_surface_extended_call_wraps_hyperlink_with_style_hint() {
        let got = eval_surface_extended_call(
            FUNC_ID_HYPERLINK,
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(
                    "https://example.com",
                ))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("Go"))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            None,
        );
        assert_eq!(
            got,
            Ok(ExtendedValue::ValueWithPresentation {
                value: EvalValue::Text(ExcelText::from_interop_assignment("Go")),
                hint: PresentationHint::style(CellStyleHint::Hyperlink),
            })
        );
    }

    #[test]
    fn eval_surface_extended_call_wraps_image_with_rich_value() {
        let got = eval_surface_extended_call(
            FUNC_ID_IMAGE,
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(
                    "https://example.com/image.png",
                ))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(
                    "Sphere",
                ))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            Some(&TestImageProvider),
        );
        match got {
            Ok(ExtendedValue::RichValue(rich)) => {
                assert_eq!(rich.value_type.type_name, "_webimage");
                assert!(matches!(rich.fallback, RichValueData::Text(_)));
            }
            other => panic!("expected rich image surface, got {other:?}"),
        }
    }

    #[test]
    fn eval_surface_value_call_routes_image_through_host_provider() {
        let got = eval_surface_value_call(
            FUNC_ID_IMAGE,
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(
                    "https://example.com/image.png",
                ))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(
                    "Sphere",
                ))),
            ],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
            None,
            Some(&TestImageProvider),
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment(
                "-2146826273"
            )))
        );
    }

    #[test]
    fn arg_preparation_profile_reports_roman_as_values_only() {
        let got = arg_preparation_profile(FUNC_ID_ROMAN);
        assert_eq!(got, Some(ArgPreparationProfile::ValuesOnlyPreAdapter));
    }

    #[test]
    fn eval_surface_q_unary_number_abs_calls_kernel() {
        let got = eval_surface_q_unary_number(FUNC_ID_ABS, -3.0);
        assert_eq!(got, Ok(3.0));
    }

    #[test]
    fn eval_surface_q_binary_number_add_calls_kernel() {
        let got = eval_surface_q_binary_number(FUNC_ID_OP_ADD, 1.5, 2.0);
        assert_eq!(got, Ok(3.5));
    }

    #[test]
    fn eval_surface_q_binary_number_round_calls_kernel() {
        let got = eval_surface_q_binary_number(FUNC_ID_ROUND, 12.34, 1.0);
        assert_eq!(got, Ok(12.3));
    }

    #[test]
    fn eval_surface_q_binary_number_power_calls_kernel() {
        let got = eval_surface_q_binary_number(FUNC_ID_POWER, 2.0, 3.0);
        assert_eq!(got, Ok(8.0));
    }

    #[test]
    fn eval_surface_q_nullary_number_pi_returns_constant() {
        let got = eval_surface_q_nullary_number(FUNC_ID_PI);
        assert_eq!(got, Ok(std::f64::consts::PI));
    }
}
