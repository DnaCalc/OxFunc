// Auto-generated from docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2*.json.
// Do not hand-edit parameter data here; update the OxFunc witness source artifacts and regenerate.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ParameterSeed {
    pub name: &'static str,
    pub optional: bool,
    pub repeats: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SignatureSeed {
    pub function_id: &'static str,
    pub signature_display: &'static str,
    pub parameters: &'static [ParameterSeed],
    pub trailing_repeats: bool,
}

pub(crate) const SIGNATURE_SEEDS: &[SignatureSeed] = &[
    SignatureSeed {
        function_id: "FUNC.ABS",
        signature_display: "ABS(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ACCRINT",
        signature_display: "ACCRINT(issue, first_interest, settlement, rate, par, frequency, [basis], [calc_method])",
        parameters: &[
            ParameterSeed {
                name: "issue",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "first_interest",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "settlement",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "rate",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "par",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "frequency",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "basis",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "calc_method",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ACCRINTM",
        signature_display: "ACCRINTM(issue, settlement, rate, par, [basis])",
        parameters: &[
            ParameterSeed {
                name: "issue",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "settlement",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "rate",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "par",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "basis",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ACOS",
        signature_display: "ACOS(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ACOSH",
        signature_display: "ACOSH(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ACOT",
        signature_display: "ACOT(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ACOTH",
        signature_display: "ACOTH(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ADDRESS",
        signature_display: "ADDRESS(row_num, column_num, [abs_num], [a1], [sheet_text])",
        parameters: &[
            ParameterSeed {
                name: "row_num",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "column_num",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "abs_num",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "a1",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "sheet_text",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.AGGREGATE",
        signature_display: "AGGREGATE(function_num, options, ref1, [ref2], ...)",
        parameters: &[
            ParameterSeed {
                name: "function_num",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "options",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "ref1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "ref2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.AMORDEGRC",
        signature_display: "AMORDEGRC(cost, date_purchased, first_period, salvage, period, rate, [basis])",
        parameters: &[
            ParameterSeed {
                name: "cost",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "date_purchased",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "first_period",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "salvage",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "period",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "rate",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "basis",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.AMORLINC",
        signature_display: "AMORLINC(cost, date_purchased, first_period, salvage, period, rate, [basis])",
        parameters: &[
            ParameterSeed {
                name: "cost",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "date_purchased",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "first_period",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "salvage",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "period",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "rate",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "basis",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.AND",
        signature_display: "AND(logical1, [logical2], ...)",
        parameters: &[
            ParameterSeed {
                name: "logical1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "logical2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ARABIC",
        signature_display: "ARABIC(text)",
        parameters: &[ParameterSeed {
            name: "text",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.AREAS",
        signature_display: "AREAS(reference)",
        parameters: &[ParameterSeed {
            name: "reference",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ARRAYTOTEXT",
        signature_display: "ARRAYTOTEXT(array, [format])",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "format",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ASC",
        signature_display: "ASC(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ASIN",
        signature_display: "ASIN(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ASINH",
        signature_display: "ASINH(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ATAN",
        signature_display: "ATAN(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ATAN2",
        signature_display: "ATAN2(x_num, y_num)",
        parameters: &[
            ParameterSeed {
                name: "x_num",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "y_num",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ATANH",
        signature_display: "ATANH(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.AVEDEV",
        signature_display: "AVEDEV(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.AVERAGE",
        signature_display: "AVERAGE(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.AVERAGEA",
        signature_display: "AVERAGEA(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.AVERAGEIF",
        signature_display: "AVERAGEIF(range, criteria, [average_range])",
        parameters: &[
            ParameterSeed {
                name: "range",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "criteria",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "average_range",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.AVERAGEIFS",
        signature_display: "AVERAGEIFS(average_range, criteria_range1, criteria1, ...)",
        parameters: &[
            ParameterSeed {
                name: "average_range",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "criteria_range1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "criteria1",
                optional: false,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.BAHTTEXT",
        signature_display: "BAHTTEXT(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BASE",
        signature_display: "BASE(number, radix, [min_length])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "radix",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "min_length",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BESSELI",
        signature_display: "BESSELI(x, n)",
        parameters: &[
            ParameterSeed {
                name: "x",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "n",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BESSELJ",
        signature_display: "BESSELJ(x, n)",
        parameters: &[
            ParameterSeed {
                name: "x",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "n",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BESSELK",
        signature_display: "BESSELK(x, n)",
        parameters: &[
            ParameterSeed {
                name: "x",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "n",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BESSELY",
        signature_display: "BESSELY(x, n)",
        parameters: &[
            ParameterSeed {
                name: "x",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "n",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BETA.DIST",
        signature_display: "BETA.DIST(x, alpha, beta, cumulative, [A], [B])",
        parameters: &[
            ParameterSeed {
                name: "x",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "alpha",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "beta",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "cumulative",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "A",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BETA.INV",
        signature_display: "BETA.INV(probability, alpha, beta, [A], [B])",
        parameters: &[
            ParameterSeed {
                name: "probability",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "alpha",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "beta",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "A",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BETADIST",
        signature_display: "BETADIST(x, alpha, beta, [A], [B])",
        parameters: &[
            ParameterSeed {
                name: "x",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "alpha",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "beta",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "A",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BETAINV",
        signature_display: "BETAINV(probability, alpha, beta, [A], [B])",
        parameters: &[
            ParameterSeed {
                name: "probability",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "alpha",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "beta",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "A",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BIN2DEC",
        signature_display: "BIN2DEC(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BIN2HEX",
        signature_display: "BIN2HEX(number, [places])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "places",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BIN2OCT",
        signature_display: "BIN2OCT(number, [places])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "places",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BINOM.DIST",
        signature_display: "BINOM.DIST(number_s, trials, probability_s, cumulative)",
        parameters: &[
            ParameterSeed {
                name: "number_s",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "trials",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "probability_s",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "cumulative",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BINOM.DIST.RANGE",
        signature_display: "BINOM.DIST.RANGE(trials, probability_s, number_s, [number_s2])",
        parameters: &[
            ParameterSeed {
                name: "trials",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "probability_s",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number_s",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number_s2",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BINOM.INV",
        signature_display: "BINOM.INV(trials, probability_s, alpha)",
        parameters: &[
            ParameterSeed {
                name: "trials",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "probability_s",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "alpha",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BINOMDIST",
        signature_display: "BINOMDIST(number_s, trials, probability_s, cumulative)",
        parameters: &[
            ParameterSeed {
                name: "number_s",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "trials",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "probability_s",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "cumulative",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BITAND",
        signature_display: "BITAND(number1, number2)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BITLSHIFT",
        signature_display: "BITLSHIFT(number, shift_amount)",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "shift_amount",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BITOR",
        signature_display: "BITOR(number1, number2)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BITRSHIFT",
        signature_display: "BITRSHIFT(number, shift_amount)",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "shift_amount",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BITXOR",
        signature_display: "BITXOR(number1, number2)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BYCOL",
        signature_display: "BYCOL(array, lambda)",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "lambda",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.BYROW",
        signature_display: "BYROW(array, lambda)",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "lambda",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CALL",
        signature_display: "CALL(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.CEILING",
        signature_display: "CEILING(number, significance)",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "significance",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CEILING.MATH",
        signature_display: "CEILING.MATH(number, [significance], [mode])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "significance",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "mode",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CEILING.PRECISE",
        signature_display: "CEILING.PRECISE(number, [significance])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "significance",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CELL",
        signature_display: "CELL(info_type, [reference])",
        parameters: &[
            ParameterSeed {
                name: "info_type",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "reference",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CHAR",
        signature_display: "CHAR(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CHIDIST",
        signature_display: "CHIDIST(x, deg_freedom)",
        parameters: &[
            ParameterSeed {
                name: "x",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "deg_freedom",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CHIINV",
        signature_display: "CHIINV(probability, deg_freedom)",
        parameters: &[
            ParameterSeed {
                name: "probability",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "deg_freedom",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CHISQ.DIST",
        signature_display: "CHISQ.DIST(x, deg_freedom, cumulative)",
        parameters: &[
            ParameterSeed {
                name: "x",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "deg_freedom",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "cumulative",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CHISQ.DIST.RT",
        signature_display: "CHISQ.DIST.RT(x, deg_freedom)",
        parameters: &[
            ParameterSeed {
                name: "x",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "deg_freedom",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CHISQ.INV",
        signature_display: "CHISQ.INV(probability, deg_freedom)",
        parameters: &[
            ParameterSeed {
                name: "probability",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "deg_freedom",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CHISQ.INV.RT",
        signature_display: "CHISQ.INV.RT(probability, deg_freedom)",
        parameters: &[
            ParameterSeed {
                name: "probability",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "deg_freedom",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CHISQ.TEST",
        signature_display: "CHISQ.TEST(actual_range, expected_range)",
        parameters: &[
            ParameterSeed {
                name: "actual_range",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "expected_range",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CHITEST",
        signature_display: "CHITEST(actual_range, expected_range)",
        parameters: &[
            ParameterSeed {
                name: "actual_range",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "expected_range",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CHOOSE",
        signature_display: "CHOOSE(index_num, value1, [value2], ...)",
        parameters: &[
            ParameterSeed {
                name: "index_num",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "value1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "value2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.CHOOSECOLS",
        signature_display: "CHOOSECOLS(array, col_num1, [col_num2], ...)",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "col_num1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "col_num2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.CHOOSEROWS",
        signature_display: "CHOOSEROWS(array, row_num1, [row_num2], ...)",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "row_num1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "row_num2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.CLEAN",
        signature_display: "CLEAN(text)",
        parameters: &[ParameterSeed {
            name: "text",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CODE",
        signature_display: "CODE(text)",
        parameters: &[ParameterSeed {
            name: "text",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.COLUMN",
        signature_display: "COLUMN(reference)",
        parameters: &[ParameterSeed {
            name: "reference",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.COLUMNS",
        signature_display: "COLUMNS(array)",
        parameters: &[ParameterSeed {
            name: "array",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.COMBIN",
        signature_display: "COMBIN(number, number_chosen)",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number_chosen",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.COMBINA",
        signature_display: "COMBINA(number, number_chosen)",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number_chosen",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.COMPLEX",
        signature_display: "COMPLEX(real_num, i_num, [suffix])",
        parameters: &[
            ParameterSeed {
                name: "real_num",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "i_num",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "suffix",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CONCAT",
        signature_display: "CONCAT(text1, [text2], ...)",
        parameters: &[
            ParameterSeed {
                name: "text1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "text2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.CONCATENATE",
        signature_display: "CONCATENATE(text1, [text2], ...)",
        parameters: &[
            ParameterSeed {
                name: "text1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "text2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.CONFIDENCE",
        signature_display: "CONFIDENCE(alpha, standard_dev, size)",
        parameters: &[
            ParameterSeed {
                name: "alpha",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "standard_dev",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "size",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CONFIDENCE.NORM",
        signature_display: "CONFIDENCE.NORM(alpha, standard_dev, size)",
        parameters: &[
            ParameterSeed {
                name: "alpha",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "standard_dev",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "size",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CONFIDENCE.T",
        signature_display: "CONFIDENCE.T(alpha, standard_dev, size)",
        parameters: &[
            ParameterSeed {
                name: "alpha",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "standard_dev",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "size",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CONVERT",
        signature_display: "CONVERT(number, from_unit, to_unit)",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "from_unit",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "to_unit",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CORREL",
        signature_display: "CORREL(array1, array2)",
        parameters: &[
            ParameterSeed {
                name: "array1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "array2",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.COS",
        signature_display: "COS(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.COSH",
        signature_display: "COSH(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.COT",
        signature_display: "COT(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.COTH",
        signature_display: "COTH(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.COUNT",
        signature_display: "COUNT(value1, [value2], ...)",
        parameters: &[
            ParameterSeed {
                name: "value1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "value2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.COUNTA",
        signature_display: "COUNTA(value1, [value2], ...)",
        parameters: &[
            ParameterSeed {
                name: "value1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "value2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.COUNTBLANK",
        signature_display: "COUNTBLANK(range)",
        parameters: &[ParameterSeed {
            name: "range",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.COUNTIF",
        signature_display: "COUNTIF(range, criteria)",
        parameters: &[
            ParameterSeed {
                name: "range",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "criteria",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.COUNTIFS",
        signature_display: "COUNTIFS(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.COUPDAYBS",
        signature_display: "COUPDAYBS(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.COUPDAYS",
        signature_display: "COUPDAYS(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.COUPDAYSNC",
        signature_display: "COUPDAYSNC(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.COUPNCD",
        signature_display: "COUPNCD(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.COUPNUM",
        signature_display: "COUPNUM(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.COUPPCD",
        signature_display: "COUPPCD(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.COVAR",
        signature_display: "COVAR(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.COVARIANCE.P",
        signature_display: "COVARIANCE.P(array1, array2)",
        parameters: &[
            ParameterSeed {
                name: "array1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "array2",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.COVARIANCE.S",
        signature_display: "COVARIANCE.S(array1, array2)",
        parameters: &[
            ParameterSeed {
                name: "array1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "array2",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CRITBINOM",
        signature_display: "CRITBINOM(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.CSC",
        signature_display: "CSC(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CSCH",
        signature_display: "CSCH(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.CUMIPMT",
        signature_display: "CUMIPMT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.CUMPRINC",
        signature_display: "CUMPRINC(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DATE",
        signature_display: "DATE(year, month, day)",
        parameters: &[
            ParameterSeed {
                name: "year",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "month",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "day",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.DATEDIF",
        signature_display: "DATEDIF(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DATEVALUE",
        signature_display: "DATEVALUE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DAVERAGE",
        signature_display: "DAVERAGE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DAY",
        signature_display: "DAY(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DAYS",
        signature_display: "DAYS(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DAYS360",
        signature_display: "DAYS360(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DB",
        signature_display: "DB(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DBCS",
        signature_display: "DBCS(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DCOUNT",
        signature_display: "DCOUNT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DCOUNTA",
        signature_display: "DCOUNTA(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DDB",
        signature_display: "DDB(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DEC2BIN",
        signature_display: "DEC2BIN(number, [places])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "places",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.DEC2HEX",
        signature_display: "DEC2HEX(number, [places])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "places",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.DEC2OCT",
        signature_display: "DEC2OCT(number, [places])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "places",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.DECIMAL",
        signature_display: "DECIMAL(text, radix)",
        parameters: &[
            ParameterSeed {
                name: "text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "radix",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.DEGREES",
        signature_display: "DEGREES(angle)",
        parameters: &[ParameterSeed {
            name: "angle",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.DELTA",
        signature_display: "DELTA(number1, number2)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.DEVSQ",
        signature_display: "DEVSQ(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DGET",
        signature_display: "DGET(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DISC",
        signature_display: "DISC(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DMAX",
        signature_display: "DMAX(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DMIN",
        signature_display: "DMIN(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DOLLAR",
        signature_display: "DOLLAR(number, [decimals])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "decimals",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.DOLLARDE",
        signature_display: "DOLLARDE(fractional_dollar, fraction)",
        parameters: &[
            ParameterSeed {
                name: "fractional_dollar",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "fraction",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.DOLLARFR",
        signature_display: "DOLLARFR(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DPRODUCT",
        signature_display: "DPRODUCT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DROP",
        signature_display: "DROP(array, rows)",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "rows",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.DSTDEV",
        signature_display: "DSTDEV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DSTDEVP",
        signature_display: "DSTDEVP(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DSUM",
        signature_display: "DSUM(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DURATION",
        signature_display: "DURATION(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DVAR",
        signature_display: "DVAR(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.DVARP",
        signature_display: "DVARP(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.EDATE",
        signature_display: "EDATE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.EFFECT",
        signature_display: "EFFECT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.EOMONTH",
        signature_display: "EOMONTH(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ERF",
        signature_display: "ERF(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ERF.PRECISE",
        signature_display: "ERF.PRECISE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ERFC",
        signature_display: "ERFC(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ERFC.PRECISE",
        signature_display: "ERFC.PRECISE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ERROR.TYPE",
        signature_display: "ERROR.TYPE(error_val)",
        parameters: &[ParameterSeed {
            name: "error_val",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.EVEN",
        signature_display: "EVEN(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.EXACT",
        signature_display: "EXACT(text1, text2)",
        parameters: &[
            ParameterSeed {
                name: "text1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "text2",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.EXP",
        signature_display: "EXP(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.EXPAND",
        signature_display: "EXPAND(array, rows, [columns], [pad_with])",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "rows",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "columns",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "pad_with",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.EXPON.DIST",
        signature_display: "EXPON.DIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.EXPONDIST",
        signature_display: "EXPONDIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.F.DIST",
        signature_display: "F.DIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.F.DIST.RT",
        signature_display: "F.DIST.RT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.F.INV",
        signature_display: "F.INV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.F.INV.RT",
        signature_display: "F.INV.RT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.F.TEST",
        signature_display: "F.TEST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.FACT",
        signature_display: "FACT(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.FACTDOUBLE",
        signature_display: "FACTDOUBLE(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.FALSE",
        signature_display: "FALSE()",
        parameters: &[],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.FDIST",
        signature_display: "FDIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.FILTER",
        signature_display: "FILTER(array, include, [if_empty])",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "include",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "if_empty",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.FINV",
        signature_display: "FINV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.FISHER",
        signature_display: "FISHER(x)",
        parameters: &[ParameterSeed {
            name: "x",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.FISHERINV",
        signature_display: "FISHERINV(y)",
        parameters: &[ParameterSeed {
            name: "y",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.FIXED",
        signature_display: "FIXED(number, [decimals])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "decimals",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.FLOOR",
        signature_display: "FLOOR(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.FLOOR.MATH",
        signature_display: "FLOOR.MATH(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.FLOOR.PRECISE",
        signature_display: "FLOOR.PRECISE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.FORECAST",
        signature_display: "FORECAST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.FORECAST.LINEAR",
        signature_display: "FORECAST.LINEAR(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.FORMULATEXT",
        signature_display: "FORMULATEXT(reference)",
        parameters: &[ParameterSeed {
            name: "reference",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.FREQUENCY",
        signature_display: "FREQUENCY(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.FTEST",
        signature_display: "FTEST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.FV",
        signature_display: "FV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.FVSCHEDULE",
        signature_display: "FVSCHEDULE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.GAMMA",
        signature_display: "GAMMA(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.GAMMA.DIST",
        signature_display: "GAMMA.DIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.GAMMA.INV",
        signature_display: "GAMMA.INV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.GAMMADIST",
        signature_display: "GAMMADIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.GAMMAINV",
        signature_display: "GAMMAINV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.GAMMALN",
        signature_display: "GAMMALN(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.GAMMALN.PRECISE",
        signature_display: "GAMMALN.PRECISE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.GAUSS",
        signature_display: "GAUSS(z)",
        parameters: &[ParameterSeed {
            name: "z",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.GCD",
        signature_display: "GCD(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.GEOMEAN",
        signature_display: "GEOMEAN(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.GESTEP",
        signature_display: "GESTEP(number, [step])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "step",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.GROUPBY",
        signature_display: "GROUPBY(row_fields, values, function, [field_headers], [total_depth], [sort_order], [filter_array], [field_relationship])",
        parameters: &[
            ParameterSeed {
                name: "row_fields",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "values",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "function",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.GROWTH",
        signature_display: "GROWTH(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.HARMEAN",
        signature_display: "HARMEAN(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.HEX2BIN",
        signature_display: "HEX2BIN(number, [places])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "places",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.HEX2DEC",
        signature_display: "HEX2DEC(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.HEX2OCT",
        signature_display: "HEX2OCT(number, [places])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "places",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.HLOOKUP",
        signature_display: "HLOOKUP(lookup_value, table_array, row_index_num, [range_lookup])",
        parameters: &[
            ParameterSeed {
                name: "lookup_value",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "table_array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "row_index_num",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "range_lookup",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.HOUR",
        signature_display: "HOUR(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.HSTACK",
        signature_display: "HSTACK(array1, [array2], ...)",
        parameters: &[
            ParameterSeed {
                name: "array1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "array2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.HYPERLINK",
        signature_display: "HYPERLINK(link_location, [friendly_name])",
        parameters: &[
            ParameterSeed {
                name: "link_location",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "friendly_name",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.HYPGEOM.DIST",
        signature_display: "HYPGEOM.DIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.HYPGEOMDIST",
        signature_display: "HYPGEOMDIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IF",
        signature_display: "IF(logical_test, value_if_true, [value_if_false])",
        parameters: &[
            ParameterSeed {
                name: "logical_test",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "value_if_true",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "value_if_false",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.IFERROR",
        signature_display: "IFERROR(value, value_if_error)",
        parameters: &[
            ParameterSeed {
                name: "value",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "value_if_error",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.IFNA",
        signature_display: "IFNA(value, value_if_na)",
        parameters: &[
            ParameterSeed {
                name: "value",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "value_if_na",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.IFS",
        signature_display: "IFS(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMABS",
        signature_display: "IMABS(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMAGE",
        signature_display: "IMAGE(source, [alt_text], [sizing], [height], [width])",
        parameters: &[
            ParameterSeed {
                name: "source",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "alt_text",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "sizing",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "height",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "width",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.IMAGINARY",
        signature_display: "IMAGINARY(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMARGUMENT",
        signature_display: "IMARGUMENT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMCONJUGATE",
        signature_display: "IMCONJUGATE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMCOS",
        signature_display: "IMCOS(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMCOSH",
        signature_display: "IMCOSH(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMCOT",
        signature_display: "IMCOT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMCSC",
        signature_display: "IMCSC(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMCSCH",
        signature_display: "IMCSCH(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMDIV",
        signature_display: "IMDIV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMEXP",
        signature_display: "IMEXP(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMLN",
        signature_display: "IMLN(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMLOG10",
        signature_display: "IMLOG10(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMLOG2",
        signature_display: "IMLOG2(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMPOWER",
        signature_display: "IMPOWER(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMPRODUCT",
        signature_display: "IMPRODUCT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMREAL",
        signature_display: "IMREAL(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMSEC",
        signature_display: "IMSEC(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMSECH",
        signature_display: "IMSECH(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMSIN",
        signature_display: "IMSIN(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMSINH",
        signature_display: "IMSINH(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMSQRT",
        signature_display: "IMSQRT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMSUB",
        signature_display: "IMSUB(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMSUM",
        signature_display: "IMSUM(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IMTAN",
        signature_display: "IMTAN(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.INDEX",
        signature_display: "INDEX(array, row_num, [column_num], [area_num])",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "row_num",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "column_num",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "area_num",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.INDIRECT",
        signature_display: "INDIRECT(ref_text, [a1])",
        parameters: &[
            ParameterSeed {
                name: "ref_text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "a1",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.INFO",
        signature_display: "INFO(type_num)",
        parameters: &[ParameterSeed {
            name: "type_num",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.INT",
        signature_display: "INT(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.INTERCEPT",
        signature_display: "INTERCEPT(known_y's, known_x's)",
        parameters: &[
            ParameterSeed {
                name: "known_y's",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "known_x's",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.INTRATE",
        signature_display: "INTRATE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IPMT",
        signature_display: "IPMT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.IRR",
        signature_display: "IRR(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ISBLANK",
        signature_display: "ISBLANK(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ISERR",
        signature_display: "ISERR(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ISERROR",
        signature_display: "ISERROR(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ISEVEN",
        signature_display: "ISEVEN(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ISFORMULA",
        signature_display: "ISFORMULA(reference)",
        parameters: &[ParameterSeed {
            name: "reference",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ISLOGICAL",
        signature_display: "ISLOGICAL(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ISNA",
        signature_display: "ISNA(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ISNONTEXT",
        signature_display: "ISNONTEXT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ISNUMBER",
        signature_display: "ISNUMBER(value)",
        parameters: &[ParameterSeed {
            name: "value",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ISO.CEILING",
        signature_display: "ISO.CEILING(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ISODD",
        signature_display: "ISODD(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ISOMITTED",
        signature_display: "ISOMITTED(value)",
        parameters: &[ParameterSeed {
            name: "value",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ISOWEEKNUM",
        signature_display: "ISOWEEKNUM(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ISPMT",
        signature_display: "ISPMT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ISREF",
        signature_display: "ISREF(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ISTEXT",
        signature_display: "ISTEXT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.JIS",
        signature_display: "JIS(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.KURT",
        signature_display: "KURT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.LAMBDA",
        signature_display: "LAMBDA(parameter1, [parameter2, ...], calculation)",
        parameters: &[
            ParameterSeed {
                name: "parameter1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "parameter2",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "calculation",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.LARGE",
        signature_display: "LARGE(array, k)",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "k",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.LCM",
        signature_display: "LCM(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.LET",
        signature_display: "LET(name1, value1, calculation_or_name2, [value2], ...)",
        parameters: &[
            ParameterSeed {
                name: "name1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "value1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "calculation_or_name2",
                optional: false,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.LINEST",
        signature_display: "LINEST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.LN",
        signature_display: "LN(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.LOG",
        signature_display: "LOG(number, [base])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "base",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.LOG10",
        signature_display: "LOG10(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.LOGEST",
        signature_display: "LOGEST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.LOGINV",
        signature_display: "LOGINV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.LOGNORM.DIST",
        signature_display: "LOGNORM.DIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.LOGNORM.INV",
        signature_display: "LOGNORM.INV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.LOGNORMDIST",
        signature_display: "LOGNORMDIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.LOOKUP",
        signature_display: "LOOKUP(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.LOWER",
        signature_display: "LOWER(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MAKEARRAY",
        signature_display: "MAKEARRAY(rows, cols, lambda)",
        parameters: &[
            ParameterSeed {
                name: "rows",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "cols",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "lambda",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.MAP",
        signature_display: "MAP(array1, [array2, ...], lambda)",
        parameters: &[
            ParameterSeed {
                name: "array1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "array2",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "lambda",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.MATCH",
        signature_display: "MATCH(lookup_value, lookup_array, [match_type])",
        parameters: &[
            ParameterSeed {
                name: "lookup_value",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "lookup_array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "match_type",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.MAX",
        signature_display: "MAX(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MAXA",
        signature_display: "MAXA(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MAXIFS",
        signature_display: "MAXIFS(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MDETERM",
        signature_display: "MDETERM(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MDURATION",
        signature_display: "MDURATION(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MEDIAN",
        signature_display: "MEDIAN(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MIN",
        signature_display: "MIN(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MINA",
        signature_display: "MINA(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MINIFS",
        signature_display: "MINIFS(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MINUTE",
        signature_display: "MINUTE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MINVERSE",
        signature_display: "MINVERSE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MIRR",
        signature_display: "MIRR(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MMULT",
        signature_display: "MMULT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MOD",
        signature_display: "MOD(number, divisor)",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "divisor",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.MODE",
        signature_display: "MODE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MODE.MULT",
        signature_display: "MODE.MULT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MODE.SNGL",
        signature_display: "MODE.SNGL(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MONTH",
        signature_display: "MONTH(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MROUND",
        signature_display: "MROUND(number, multiple)",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "multiple",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.MULTINOMIAL",
        signature_display: "MULTINOMIAL(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.MUNIT",
        signature_display: "MUNIT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.N",
        signature_display: "N(value)",
        parameters: &[ParameterSeed {
            name: "value",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.NA",
        signature_display: "NA()",
        parameters: &[],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.NEGBINOM.DIST",
        signature_display: "NEGBINOM.DIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.NEGBINOMDIST",
        signature_display: "NEGBINOMDIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.NETWORKDAYS",
        signature_display: "NETWORKDAYS(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.NETWORKDAYS.INTL",
        signature_display: "NETWORKDAYS.INTL(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.NOMINAL",
        signature_display: "NOMINAL(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.NORM.DIST",
        signature_display: "NORM.DIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.NORM.INV",
        signature_display: "NORM.INV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.NORM.S.DIST",
        signature_display: "NORM.S.DIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.NORM.S.INV",
        signature_display: "NORM.S.INV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.NORMDIST",
        signature_display: "NORMDIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.NORMINV",
        signature_display: "NORMINV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.NORMSDIST",
        signature_display: "NORMSDIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.NORMSINV",
        signature_display: "NORMSINV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.NOT",
        signature_display: "NOT(logical)",
        parameters: &[ParameterSeed {
            name: "logical",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.NOW",
        signature_display: "NOW()",
        parameters: &[],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.NPER",
        signature_display: "NPER(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.NPV",
        signature_display: "NPV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.NUMBERVALUE",
        signature_display: "NUMBERVALUE(text, [decimal_separator], [group_separator])",
        parameters: &[
            ParameterSeed {
                name: "text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "decimal_separator",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "group_separator",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OCT2BIN",
        signature_display: "OCT2BIN(number, [places])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "places",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OCT2DEC",
        signature_display: "OCT2DEC(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OCT2HEX",
        signature_display: "OCT2HEX(number, [places])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "places",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ODD",
        signature_display: "ODD(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ODDFPRICE",
        signature_display: "ODDFPRICE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ODDFYIELD",
        signature_display: "ODDFYIELD(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ODDLPRICE",
        signature_display: "ODDLPRICE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ODDLYIELD",
        signature_display: "ODDLYIELD(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.OFFSET",
        signature_display: "OFFSET(reference, rows, cols, [height], [width])",
        parameters: &[
            ParameterSeed {
                name: "reference",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "rows",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "cols",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "height",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "width",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_ADD",
        signature_display: "A + B",
        parameters: &[
            ParameterSeed {
                name: "A",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_CONCAT",
        signature_display: "A & B",
        parameters: &[
            ParameterSeed {
                name: "A",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_DIVIDE",
        signature_display: "A / B",
        parameters: &[
            ParameterSeed {
                name: "A",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_EQUAL",
        signature_display: "A = B",
        parameters: &[
            ParameterSeed {
                name: "A",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_GREATER_EQUAL",
        signature_display: "A >= B",
        parameters: &[
            ParameterSeed {
                name: "A",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_GREATER_THAN",
        signature_display: "A > B",
        parameters: &[
            ParameterSeed {
                name: "A",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_IMPLICIT_INTERSECTION",
        signature_display: "@ reference_or_array",
        parameters: &[ParameterSeed {
            name: "reference_or_array",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_INTERSECTION_REF",
        signature_display: "A B",
        parameters: &[
            ParameterSeed {
                name: "A",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_LESS_EQUAL",
        signature_display: "A <= B",
        parameters: &[
            ParameterSeed {
                name: "A",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_LESS_THAN",
        signature_display: "A < B",
        parameters: &[
            ParameterSeed {
                name: "A",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_MULTIPLY",
        signature_display: "A * B",
        parameters: &[
            ParameterSeed {
                name: "A",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_NEGATE",
        signature_display: "-A",
        parameters: &[],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_NOT_EQUAL",
        signature_display: "A <> B",
        parameters: &[
            ParameterSeed {
                name: "A",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_PERCENT",
        signature_display: "A%",
        parameters: &[],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_POWER",
        signature_display: "A ^ B",
        parameters: &[
            ParameterSeed {
                name: "A",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_RANGE_REF",
        signature_display: "A:B",
        parameters: &[],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_SPILL_REF",
        signature_display: "A#",
        parameters: &[],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_SUBTRACT",
        signature_display: "A - B",
        parameters: &[
            ParameterSeed {
                name: "A",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_TRIM_REF_BOTH",
        signature_display: "A @? B",
        parameters: &[
            ParameterSeed {
                name: "A",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "B",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_TRIM_REF_LEADING",
        signature_display: "@A",
        parameters: &[],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_TRIM_REF_TRAILING",
        signature_display: "A@",
        parameters: &[],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_UNARY_PLUS",
        signature_display: "+A",
        parameters: &[],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OP_UNION_REF",
        signature_display: "A, B",
        parameters: &[],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.OR",
        signature_display: "OR(logical1, [logical2], ...)",
        parameters: &[
            ParameterSeed {
                name: "logical1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "logical2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.PDURATION",
        signature_display: "PDURATION(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.PEARSON",
        signature_display: "PEARSON(array1, array2)",
        parameters: &[
            ParameterSeed {
                name: "array1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "array2",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.PERCENTILE",
        signature_display: "PERCENTILE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.PERCENTILE.EXC",
        signature_display: "PERCENTILE.EXC(array, k)",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "k",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.PERCENTILE.INC",
        signature_display: "PERCENTILE.INC(array, k)",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "k",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.PERCENTOF",
        signature_display: "PERCENTOF(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.PERCENTRANK",
        signature_display: "PERCENTRANK(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.PERCENTRANK.EXC",
        signature_display: "PERCENTRANK.EXC(array, x, [significance])",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "x",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "significance",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.PERCENTRANK.INC",
        signature_display: "PERCENTRANK.INC(array, x, [significance])",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "x",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "significance",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.PERMUT",
        signature_display: "PERMUT(number, number_chosen)",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number_chosen",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.PERMUTATIONA",
        signature_display: "PERMUTATIONA(n, k)",
        parameters: &[
            ParameterSeed {
                name: "n",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "k",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.PHI",
        signature_display: "PHI(x)",
        parameters: &[ParameterSeed {
            name: "x",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.PI",
        signature_display: "PI()",
        parameters: &[],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.PIVOTBY",
        signature_display: "PIVOTBY(row_fields, column_fields, values, array_agg, ...)",
        parameters: &[
            ParameterSeed {
                name: "row_fields",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "column_fields",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "values",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "array_agg",
                optional: false,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.PMT",
        signature_display: "PMT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.POISSON",
        signature_display: "POISSON(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.POISSON.DIST",
        signature_display: "POISSON.DIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.POWER",
        signature_display: "POWER(number, power)",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "power",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.PPMT",
        signature_display: "PPMT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.PRICE",
        signature_display: "PRICE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.PRICEDISC",
        signature_display: "PRICEDISC(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.PRICEMAT",
        signature_display: "PRICEMAT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.PROB",
        signature_display: "PROB(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.PRODUCT",
        signature_display: "PRODUCT(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.PROPER",
        signature_display: "PROPER(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.PV",
        signature_display: "PV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.QUARTILE",
        signature_display: "QUARTILE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.QUARTILE.EXC",
        signature_display: "QUARTILE.EXC(array, quart)",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "quart",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.QUARTILE.INC",
        signature_display: "QUARTILE.INC(array, quart)",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "quart",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.QUOTIENT",
        signature_display: "QUOTIENT(numerator, denominator)",
        parameters: &[
            ParameterSeed {
                name: "numerator",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "denominator",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.RADIANS",
        signature_display: "RADIANS(angle)",
        parameters: &[ParameterSeed {
            name: "angle",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.RAND",
        signature_display: "RAND()",
        parameters: &[],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.RANDARRAY",
        signature_display: "RANDARRAY(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.RANDBETWEEN",
        signature_display: "RANDBETWEEN(bottom, top)",
        parameters: &[
            ParameterSeed {
                name: "bottom",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "top",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.RANK",
        signature_display: "RANK(number, ref, [order])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "ref",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "order",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.RANK.AVG",
        signature_display: "RANK.AVG(number, ref, [order])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "ref",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "order",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.RANK.EQ",
        signature_display: "RANK.EQ(number, ref, [order])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "ref",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "order",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.RATE",
        signature_display: "RATE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.RECEIVED",
        signature_display: "RECEIVED(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.REDUCE",
        signature_display: "REDUCE(initial_value, array, lambda)",
        parameters: &[
            ParameterSeed {
                name: "initial_value",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "lambda",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.REGEXEXTRACT",
        signature_display: "REGEXEXTRACT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.REGEXREPLACE",
        signature_display: "REGEXREPLACE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.REGEXTEST",
        signature_display: "REGEXTEST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.REGISTER.ID",
        signature_display: "REGISTER.ID(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.REPT",
        signature_display: "REPT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.ROMAN",
        signature_display: "ROMAN(number, [form])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "form",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ROUND",
        signature_display: "ROUND(number, num_digits)",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "num_digits",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ROUNDDOWN",
        signature_display: "ROUNDDOWN(number, num_digits)",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "num_digits",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ROUNDUP",
        signature_display: "ROUNDUP(number, num_digits)",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "num_digits",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ROW",
        signature_display: "ROW([reference])",
        parameters: &[ParameterSeed {
            name: "reference",
            optional: true,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ROWS",
        signature_display: "ROWS(array)",
        parameters: &[ParameterSeed {
            name: "array",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.RRI",
        signature_display: "RRI(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.RSQ",
        signature_display: "RSQ(known_y's, known_x's)",
        parameters: &[
            ParameterSeed {
                name: "known_y's",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "known_x's",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.RTD",
        signature_display: "RTD(prog_id, server, topic1, [topic2], ...)",
        parameters: &[
            ParameterSeed {
                name: "prog_id",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "server",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "topic1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "topic2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SCAN",
        signature_display: "SCAN(initial_value, array, lambda)",
        parameters: &[
            ParameterSeed {
                name: "initial_value",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "lambda",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SEC",
        signature_display: "SEC(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SECH",
        signature_display: "SECH(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SECOND",
        signature_display: "SECOND(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SEQUENCE",
        signature_display: "SEQUENCE(rows, [columns], [start], [step])",
        parameters: &[
            ParameterSeed {
                name: "rows",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "columns",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "start",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "step",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SERIESSUM",
        signature_display: "SERIESSUM(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SHEET",
        signature_display: "SHEET([reference])",
        parameters: &[ParameterSeed {
            name: "reference",
            optional: true,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SHEETS",
        signature_display: "SHEETS([reference])",
        parameters: &[ParameterSeed {
            name: "reference",
            optional: true,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SIGN",
        signature_display: "SIGN(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SIN",
        signature_display: "SIN(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SINH",
        signature_display: "SINH(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SKEW",
        signature_display: "SKEW(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SKEW.P",
        signature_display: "SKEW.P(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SLN",
        signature_display: "SLN(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SLOPE",
        signature_display: "SLOPE(known_y's, known_x's)",
        parameters: &[
            ParameterSeed {
                name: "known_y's",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "known_x's",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SMALL",
        signature_display: "SMALL(array, k)",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "k",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SORT",
        signature_display: "SORT(array, [sort_index], [sort_order], [by_col])",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "sort_index",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "sort_order",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "by_col",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SORTBY",
        signature_display: "SORTBY(array, by_array1, [sort_order1], [by_array2], ...)",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "by_array1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "sort_order1",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "by_array2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SQRT",
        signature_display: "SQRT(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SQRTPI",
        signature_display: "SQRTPI(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.STANDARDIZE",
        signature_display: "STANDARDIZE(x, mean, standard_dev)",
        parameters: &[
            ParameterSeed {
                name: "x",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "mean",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "standard_dev",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.STDEV",
        signature_display: "STDEV(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.STDEV.P",
        signature_display: "STDEV.P(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.STDEV.S",
        signature_display: "STDEV.S(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.STDEVA",
        signature_display: "STDEVA(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.STDEVP",
        signature_display: "STDEVP(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.STDEVPA",
        signature_display: "STDEVPA(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.STEYX",
        signature_display: "STEYX(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SUBSTITUTE",
        signature_display: "SUBSTITUTE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SUBTOTAL",
        signature_display: "SUBTOTAL(function_num, ref1, [ref2], ...)",
        parameters: &[
            ParameterSeed {
                name: "function_num",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "ref1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "ref2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SUM",
        signature_display: "SUM(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SUMIF",
        signature_display: "SUMIF(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SUMIFS",
        signature_display: "SUMIFS(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SUMPRODUCT",
        signature_display: "SUMPRODUCT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SUMSQ",
        signature_display: "SUMSQ(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SUMX2MY2",
        signature_display: "SUMX2MY2(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SUMX2PY2",
        signature_display: "SUMX2PY2(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SUMXMY2",
        signature_display: "SUMXMY2(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.SWITCH",
        signature_display: "SWITCH(expression, value1, result1, [value2, result2], [default])",
        parameters: &[
            ParameterSeed {
                name: "expression",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "value1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "result1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "value2",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "result2",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "default",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SYD",
        signature_display: "SYD(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.T",
        signature_display: "T(value)",
        parameters: &[ParameterSeed {
            name: "value",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.T.DIST",
        signature_display: "T.DIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.T.DIST.2T",
        signature_display: "T.DIST.2T(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.T.DIST.RT",
        signature_display: "T.DIST.RT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.T.INV",
        signature_display: "T.INV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.T.INV.2T",
        signature_display: "T.INV.2T(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.T.TEST",
        signature_display: "T.TEST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TAKE",
        signature_display: "TAKE(array, rows, [columns])",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "rows",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "columns",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.TAN",
        signature_display: "TAN(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.TANH",
        signature_display: "TANH(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.TBILLEQ",
        signature_display: "TBILLEQ(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TBILLPRICE",
        signature_display: "TBILLPRICE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TBILLYIELD",
        signature_display: "TBILLYIELD(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TDIST",
        signature_display: "TDIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TEXT",
        signature_display: "TEXT(value, format_text)",
        parameters: &[
            ParameterSeed {
                name: "value",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "format_text",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.TEXTAFTER",
        signature_display: "TEXTAFTER(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TEXTBEFORE",
        signature_display: "TEXTBEFORE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TEXTJOIN",
        signature_display: "TEXTJOIN(delimiter, ignore_empty, text1, [text2], ...)",
        parameters: &[
            ParameterSeed {
                name: "delimiter",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "ignore_empty",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "text1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "text2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TEXTSPLIT",
        signature_display: "TEXTSPLIT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TIME",
        signature_display: "TIME(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TIMEVALUE",
        signature_display: "TIMEVALUE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TINV",
        signature_display: "TINV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TOCOL",
        signature_display: "TOCOL(array, [ignore], [scan_by_column])",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "ignore",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "scan_by_column",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.TODAY",
        signature_display: "TODAY()",
        parameters: &[],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.TOROW",
        signature_display: "TOROW(array, [ignore], [scan_by_column])",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "ignore",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "scan_by_column",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.TRANSPOSE",
        signature_display: "TRANSPOSE(array)",
        parameters: &[ParameterSeed {
            name: "array",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.TREND",
        signature_display: "TREND(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TRIM",
        signature_display: "TRIM(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TRIMMEAN",
        signature_display: "TRIMMEAN(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TRIMRANGE",
        signature_display: "TRIMRANGE(array, [trim_rows], [trim_cols], [headers])",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "trim_rows",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "trim_cols",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "headers",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.TRUE",
        signature_display: "TRUE()",
        parameters: &[],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.TRUNC",
        signature_display: "TRUNC(number, [num_digits])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "num_digits",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.TTEST",
        signature_display: "TTEST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.TYPE",
        signature_display: "TYPE(value)",
        parameters: &[ParameterSeed {
            name: "value",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.UNICHAR",
        signature_display: "UNICHAR(number)",
        parameters: &[ParameterSeed {
            name: "number",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.UNICODE",
        signature_display: "UNICODE(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.UNIQUE",
        signature_display: "UNIQUE(array, [by_col], [exactly_once])",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "by_col",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "exactly_once",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.UPPER",
        signature_display: "UPPER(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.VALUE",
        signature_display: "VALUE(text)",
        parameters: &[ParameterSeed {
            name: "text",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.VALUETOTEXT",
        signature_display: "VALUETOTEXT(value, [format])",
        parameters: &[
            ParameterSeed {
                name: "value",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "format",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.VAR",
        signature_display: "VAR(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.VAR.P",
        signature_display: "VAR.P(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.VAR.S",
        signature_display: "VAR.S(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.VARA",
        signature_display: "VARA(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.VARP",
        signature_display: "VARP(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.VARPA",
        signature_display: "VARPA(number1, [number2], ...)",
        parameters: &[
            ParameterSeed {
                name: "number1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "number2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.VDB",
        signature_display: "VDB(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.VLOOKUP",
        signature_display: "VLOOKUP(lookup_value, table_array, col_index_num, [range_lookup])",
        parameters: &[
            ParameterSeed {
                name: "lookup_value",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "table_array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "col_index_num",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "range_lookup",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.VSTACK",
        signature_display: "VSTACK(array1, [array2], ...)",
        parameters: &[
            ParameterSeed {
                name: "array1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "array2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.WEEKDAY",
        signature_display: "WEEKDAY(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.WEEKNUM",
        signature_display: "WEEKNUM(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.WEIBULL",
        signature_display: "WEIBULL(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.WEIBULL.DIST",
        signature_display: "WEIBULL.DIST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.WORKDAY",
        signature_display: "WORKDAY(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.WORKDAY.INTL",
        signature_display: "WORKDAY.INTL(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.WRAPCOLS",
        signature_display: "WRAPCOLS(array, wrap_count, [pad_with])",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "wrap_count",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "pad_with",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.WRAPROWS",
        signature_display: "WRAPROWS(array, wrap_count, [pad_with])",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "wrap_count",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "pad_with",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.XIRR",
        signature_display: "XIRR(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.XLOOKUP",
        signature_display: "XLOOKUP(lookup_value, lookup_array, return_array, [if_not_found], [match_mode], [search_mode])",
        parameters: &[
            ParameterSeed {
                name: "lookup_value",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "lookup_array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "return_array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "if_not_found",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "match_mode",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "search_mode",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.XMATCH",
        signature_display: "XMATCH(lookup_value, lookup_array, [match_mode], [search_mode])",
        parameters: &[
            ParameterSeed {
                name: "lookup_value",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "lookup_array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "match_mode",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "search_mode",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.XNPV",
        signature_display: "XNPV(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.XOR",
        signature_display: "XOR(logical1, [logical2], ...)",
        parameters: &[
            ParameterSeed {
                name: "logical1",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "logical2",
                optional: true,
                repeats: true,
            },
        ],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.YEAR",
        signature_display: "YEAR(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.YEARFRAC",
        signature_display: "YEARFRAC(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.YIELD",
        signature_display: "YIELD(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.YIELDDISC",
        signature_display: "YIELDDISC(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.YIELDMAT",
        signature_display: "YIELDMAT(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    SignatureSeed {
        function_id: "FUNC.Z.TEST",
        signature_display: "Z.TEST(array, x, [sigma])",
        parameters: &[
            ParameterSeed {
                name: "array",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "x",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "sigma",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.ZTEST",
        signature_display: "ZTEST(...)",
        parameters: &[],
        trailing_repeats: true,
    },
    // Manual W091 seeds for linked built-ins not yet present in the W71 witness-signature JSON set.
    SignatureSeed {
        function_id: "FUNC.ENCODEURL",
        signature_display: "ENCODEURL(text)",
        parameters: &[ParameterSeed {
            name: "text",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.EUROCONVERT",
        signature_display: "EUROCONVERT(number, source, target, [full_precision], [triangulation_precision])",
        parameters: &[
            ParameterSeed {
                name: "number",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "source",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "target",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "full_precision",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "triangulation_precision",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.FILTERXML",
        signature_display: "FILTERXML(xml, xpath)",
        parameters: &[
            ParameterSeed {
                name: "xml",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "xpath",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.FIND",
        signature_display: "FIND(find_text, within_text, [start_num])",
        parameters: &[
            ParameterSeed {
                name: "find_text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "within_text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "start_num",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.FINDB",
        signature_display: "FINDB(find_text, within_text, [start_num])",
        parameters: &[
            ParameterSeed {
                name: "find_text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "within_text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "start_num",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.LEFT",
        signature_display: "LEFT(text, [num_chars])",
        parameters: &[
            ParameterSeed {
                name: "text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "num_chars",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.LEFTB",
        signature_display: "LEFTB(text, [num_bytes])",
        parameters: &[
            ParameterSeed {
                name: "text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "num_bytes",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.LEN",
        signature_display: "LEN(text)",
        parameters: &[ParameterSeed {
            name: "text",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.LENB",
        signature_display: "LENB(text)",
        parameters: &[ParameterSeed {
            name: "text",
            optional: false,
            repeats: false,
        }],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.MID",
        signature_display: "MID(text, start_num, num_chars)",
        parameters: &[
            ParameterSeed {
                name: "text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "start_num",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "num_chars",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.MIDB",
        signature_display: "MIDB(text, start_num, num_bytes)",
        parameters: &[
            ParameterSeed {
                name: "text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "start_num",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "num_bytes",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.REPLACE",
        signature_display: "REPLACE(old_text, start_num, num_chars, new_text)",
        parameters: &[
            ParameterSeed {
                name: "old_text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "start_num",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "num_chars",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "new_text",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.REPLACEB",
        signature_display: "REPLACEB(old_text, start_num, num_bytes, new_text)",
        parameters: &[
            ParameterSeed {
                name: "old_text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "start_num",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "num_bytes",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "new_text",
                optional: false,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.RIGHT",
        signature_display: "RIGHT(text, [num_chars])",
        parameters: &[
            ParameterSeed {
                name: "text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "num_chars",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.RIGHTB",
        signature_display: "RIGHTB(text, [num_bytes])",
        parameters: &[
            ParameterSeed {
                name: "text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "num_bytes",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SEARCH",
        signature_display: "SEARCH(find_text, within_text, [start_num])",
        parameters: &[
            ParameterSeed {
                name: "find_text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "within_text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "start_num",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.SEARCHB",
        signature_display: "SEARCHB(find_text, within_text, [start_num])",
        parameters: &[
            ParameterSeed {
                name: "find_text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "within_text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "start_num",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
    SignatureSeed {
        function_id: "FUNC.TRANSLATE",
        signature_display: "TRANSLATE(text, [source_language], [target_language])",
        parameters: &[
            ParameterSeed {
                name: "text",
                optional: false,
                repeats: false,
            },
            ParameterSeed {
                name: "source_language",
                optional: true,
                repeats: false,
            },
            ParameterSeed {
                name: "target_language",
                optional: true,
                repeats: false,
            },
        ],
        trailing_repeats: false,
    },
];

pub(crate) fn signature_seed_for_id(function_id: &str) -> Option<&'static SignatureSeed> {
    SIGNATURE_SEEDS
        .iter()
        .find(|seed| seed.function_id.eq_ignore_ascii_case(function_id))
}
