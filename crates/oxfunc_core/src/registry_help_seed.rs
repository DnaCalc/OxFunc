// Auto-generated from current OxFunc function catalog and rich V2 witness seeds.
// Do not hand-edit help data here; update the source artifacts and rerun tools/generate-registry-help-seed.ps1.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ParameterHelpSeed {
    pub index: usize,
    pub name: &'static str,
    pub short_description: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RegistryHelpSeed {
    pub function_id: &'static str,
    pub short_description: Option<&'static str>,
    pub long_description: Option<&'static str>,
    pub parameters: &'static [ParameterHelpSeed],
}

pub(crate) const REGISTRY_HELP_SEEDS: &[RegistryHelpSeed] = &[
    RegistryHelpSeed {
        function_id: "FUNC.ABS",
        short_description: Some("Returns the absolute value of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ACCRINT",
        short_description: Some(
            "Returns the accrued interest for a security that pays periodic interest",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ACCRINTM",
        short_description: Some(
            "Returns the accrued interest for a security that pays interest at maturity",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ACOS",
        short_description: Some("Returns the arccosine of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ACOSH",
        short_description: Some("Returns the inverse hyperbolic cosine of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ACOT",
        short_description: Some("Returns the arccotangent of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ACOTH",
        short_description: Some("Returns the hyperbolic arccotangent of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ADDRESS",
        short_description: Some("Returns a reference as text to a single cell in a worksheet"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.AGGREGATE",
        short_description: Some("Returns an aggregate in a list or database"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.AMORDEGRC",
        short_description: Some(
            "Returns the depreciation for each accounting period by using a depreciation coefficient",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.AMORLINC",
        short_description: Some("Returns the depreciation for each accounting period"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.AND",
        short_description: Some("Returns TRUE if all of its arguments are TRUE"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ARABIC",
        short_description: Some("Converts a Roman number to Arabic, as a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.AREAS",
        short_description: Some("Returns the number of areas in a reference"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ARRAYTOTEXT",
        short_description: Some("Returns an array of text values from any specified range"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ASC",
        short_description: Some(
            "Changes full-width (double-byte) English letters or katakana within a character string to half-width (single-byte) characters",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ASIN",
        short_description: Some("Returns the arcsine of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ASINH",
        short_description: Some("Returns the inverse hyperbolic sine of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ATAN",
        short_description: Some("Returns the arctangent of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ATAN2",
        short_description: Some("Returns the arctangent from x- and y-coordinates"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ATANH",
        short_description: Some("Returns the inverse hyperbolic tangent of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.AVEDEV",
        short_description: Some(
            "Returns the average of the absolute deviations of data points from their mean",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.AVERAGE",
        short_description: Some("Returns the average of its arguments"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.AVERAGEA",
        short_description: Some(
            "Returns the average of its arguments, including numbers, text, and logical values",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.AVERAGEIF",
        short_description: Some(
            "Returns the average (arithmetic mean) of all the cells in a range that meet a given criteria",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.AVERAGEIFS",
        short_description: Some(
            "Returns the average (arithmetic mean) of all cells that meet multiple criteria",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BAHTTEXT",
        short_description: Some("Converts a number to text, using the ? (baht) currency format"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BASE",
        short_description: Some(
            "Converts a number into a text representation with the given radix (base)",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BESSELI",
        short_description: Some("Returns the modified Bessel function In(x)"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BESSELJ",
        short_description: Some("Returns the Bessel function Jn(x)"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BESSELK",
        short_description: Some("Returns the modified Bessel function Kn(x)"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BESSELY",
        short_description: Some("Returns the Bessel function Yn(x)"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BETA.DIST",
        short_description: Some("Returns the beta cumulative distribution function"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BETA.INV",
        short_description: Some(
            "Returns the inverse of the cumulative distribution function for a specified beta distribution",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BETADIST",
        short_description: Some("Returns the beta cumulative distribution function"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BETAINV",
        short_description: Some(
            "Returns the inverse of the cumulative distribution function for a specified beta distribution",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BIN2DEC",
        short_description: Some("Converts a binary number to decimal"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BIN2HEX",
        short_description: Some("Converts a binary number to hexadecimal"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BIN2OCT",
        short_description: Some("Converts a binary number to octal"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BINOM.DIST",
        short_description: Some("Returns the individual term binomial distribution probability"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BINOM.DIST.RANGE",
        short_description: Some(
            "Returns the probability of a trial result using a binomial distribution",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BINOM.INV",
        short_description: Some(
            "Returns the smallest value for which the cumulative binomial distribution is less than or equal to a criterion value",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BINOMDIST",
        short_description: Some("Returns the individual term binomial distribution probability"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BITAND",
        short_description: Some("Returns a 'Bitwise And' of two numbers"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BITLSHIFT",
        short_description: Some("Returns a value number shifted left by shift_amount bits"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BITOR",
        short_description: Some("Returns a bitwise OR of 2 numbers"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BITRSHIFT",
        short_description: Some("Returns a value number shifted right by shift_amount bits"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BITXOR",
        short_description: Some("Returns a bitwise 'Exclusive Or' of two numbers"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BYCOL",
        short_description: Some(
            "Applies a LAMBDA to each column and returns an array of the results",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.BYROW",
        short_description: Some("Applies a LAMBDA to each row and returns an array of the results"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CALL",
        short_description: Some("Calls a procedure in a dynamic link library or code resource"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CEILING",
        short_description: Some(
            "Rounds a number to the nearest integer or to the nearest multiple of significance",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CEILING.MATH",
        short_description: Some(
            "Rounds a number up, to the nearest integer or to the nearest multiple of significance",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CEILING.PRECISE",
        short_description: Some(
            "Rounds a number the nearest integer or to the nearest multiple of significance. Regardless of the sign of the number, the number is rounded up.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CELL",
        short_description: Some(
            "Returns information about the formatting, location, or contents of a cell",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CHAR",
        short_description: Some("Returns the character specified by the code number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CHIDIST",
        short_description: Some(
            "Returns the one-tailed probability of the chi-squared distribution",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CHIINV",
        short_description: Some(
            "Returns the inverse of the one-tailed probability of the chi-squared distribution",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CHISQ.DIST",
        short_description: Some("Returns the cumulative beta probability density function"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CHISQ.DIST.RT",
        short_description: Some(
            "Returns the one-tailed probability of the chi-squared distribution",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CHISQ.INV",
        short_description: Some("Returns the cumulative beta probability density function"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CHISQ.INV.RT",
        short_description: Some(
            "Returns the inverse of the one-tailed probability of the chi-squared distribution",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CHISQ.TEST",
        short_description: Some("Returns the test for independence"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CHITEST",
        short_description: Some("Returns the test for independence"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CHOOSE",
        short_description: Some("Chooses a value from a list of values"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CHOOSECOLS",
        short_description: Some("Returns the specified columns from an array"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CHOOSEROWS",
        short_description: Some("Returns the specified rows from an array"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CLEAN",
        short_description: Some("Removes all nonprintable characters from text"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CODE",
        short_description: Some("Returns a numeric code for the first character in a text string"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COLUMN",
        short_description: Some("Returns the column number of a reference"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COLUMNS",
        short_description: Some("Returns the number of columns in a reference"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COMBIN",
        short_description: Some("Returns the number of combinations for a given number of objects"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COMBINA",
        short_description: Some(
            "Returns the number of combinations with repetitions for a given number of items",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COMPLEX",
        short_description: Some("Converts real and imaginary coefficients into a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CONCAT",
        short_description: Some(
            "Combines the text from multiple ranges and/or strings, but it doesn't provide the delimiter or IgnoreEmpty arguments.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CONCATENATE",
        short_description: Some("Joins two or more text strings into one string"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CONFIDENCE",
        short_description: Some("Returns the confidence interval for a population mean"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CONFIDENCE.NORM",
        short_description: Some("Returns the confidence interval for a population mean"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CONFIDENCE.T",
        short_description: Some(
            "Returns the confidence interval for a population mean, using a Student's t distribution",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CONVERT",
        short_description: Some("Converts a number from one measurement system to another"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CORREL",
        short_description: Some("Returns the correlation coefficient between two data sets"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COS",
        short_description: Some("Returns the cosine of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COSH",
        short_description: Some("Returns the hyperbolic cosine of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COT",
        short_description: Some("Returns the cotangent of an angle"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COTH",
        short_description: Some("Returns the hyperbolic cotangent of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COUNT",
        short_description: Some(
            "Use this function to count how many numbers are in the list of arguments. You can use COUNTA to count how many values are in the list of arguments.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COUNTA",
        short_description: Some("Counts how many values are in the list of arguments"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COUNTBLANK",
        short_description: Some("Counts the number of blank cells within a range"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COUNTIF",
        short_description: Some(
            "Counts the number of cells within a range that meet the given criteria",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COUNTIFS",
        short_description: Some(
            "Use this function to count the number of cells within a range that meet multiple criteria.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COUPDAYBS",
        short_description: Some(
            "Returns the number of days from the beginning of the coupon period to the settlement date",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COUPDAYS",
        short_description: Some(
            "Returns the number of days in the coupon period that contains the settlement date",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COUPDAYSNC",
        short_description: Some(
            "Returns the number of days from the settlement date to the next coupon date",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COUPNCD",
        short_description: Some("Returns the next coupon date after the settlement date"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COUPNUM",
        short_description: Some(
            "Returns the number of coupons payable between the settlement date and maturity date",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COUPPCD",
        short_description: Some("Returns the previous coupon date before the settlement date"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COVAR",
        short_description: Some(
            "Returns covariance, the average of the products of paired deviations",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COVARIANCE.P",
        short_description: Some(
            "Returns covariance, the average of the products of paired deviations",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.COVARIANCE.S",
        short_description: Some(
            "Returns the sample covariance, the average of the products deviations for each data point pair in two data sets",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CRITBINOM",
        short_description: Some(
            "Returns the smallest value for which the cumulative binomial distribution is less than or equal to a criterion value",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CSC",
        short_description: Some("Returns the cosecant of an angle"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CSCH",
        short_description: Some("Returns the hyperbolic cosecant of an angle"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CUBEKPIMEMBER",
        short_description: Some(
            "Returns a key performance indicator (KPI) property and displays the KPI name in the cell. A KPI is a quantifiable measurement, such as monthly gross profit or quarterly employee turnover, that is used to monitor an organization's performance.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CUBEMEMBER",
        short_description: Some(
            "Returns a member or tuple from the cube. Use to validate that the member or tuple exists in the cube.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CUBEMEMBERPROPERTY",
        short_description: Some(
            "Returns the value of a member property from the cube. Use to validate that a member name exists within the cube and to return the specified property for this member.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CUBERANKEDMEMBER",
        short_description: Some(
            "Returns the nth, or ranked, member in a set. Use to return one or more elements in a set, such as the top sales performer or the top 10 students.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CUBESET",
        short_description: Some(
            "Defines a calculated set of members or tuples by sending a set expression to the cube on the server, which creates the set, and then returns that set to Microsoft Excel.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CUBESETCOUNT",
        short_description: Some("Returns the number of items in a set."),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CUBEVALUE",
        short_description: Some("Returns an aggregated value from the cube."),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CUMIPMT",
        short_description: Some("Returns the cumulative interest paid between two periods"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.CUMPRINC",
        short_description: Some(
            "Returns the cumulative principal paid on a loan between two periods",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DATE",
        short_description: Some("Returns the serial number of a particular date"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DATEDIF",
        short_description: Some(
            "Calculates the number of days, months, or years between two dates. This function is useful in formulas where you need to calculate an age.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DATEVALUE",
        short_description: Some("Converts a date in the form of text to a serial number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DAVERAGE",
        short_description: Some("Returns the average of selected database entries"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DAY",
        short_description: Some("Converts a serial number to a day of the month"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DAYS",
        short_description: Some("Returns the number of days between two dates"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DAYS360",
        short_description: Some(
            "Calculates the number of days between two dates based on a 360-day year",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DB",
        short_description: Some(
            "Returns the depreciation of an asset for a specified period by using the fixed-declining balance method",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DBCS",
        short_description: Some(
            "Changes half-width (single-byte) English letters or katakana within a character string to full-width (double-byte) characters",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DCOUNT",
        short_description: Some("Counts the cells that contain numbers in a database"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DCOUNTA",
        short_description: Some("Counts nonblank cells in a database"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DDB",
        short_description: Some(
            "Returns the depreciation of an asset for a specified period by using the double-declining balance method or some other method that you specify",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DEC2BIN",
        short_description: Some("Converts a decimal number to binary"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DEC2HEX",
        short_description: Some("Converts a decimal number to hexadecimal"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DEC2OCT",
        short_description: Some("Converts a decimal number to octal"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DECIMAL",
        short_description: Some(
            "Converts a text representation of a number in a given base into a decimal number",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DEGREES",
        short_description: Some("Converts radians to degrees"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DELTA",
        short_description: Some("Tests whether two values are equal"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DETECTLANGUAGE",
        short_description: Some("Identifies the language of a specified text"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DEVSQ",
        short_description: Some("Returns the sum of squares of deviations"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DGET",
        short_description: Some(
            "Extracts from a database a single record that matches the specified criteria",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DISC",
        short_description: Some("Returns the discount rate for a security"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DMAX",
        short_description: Some("Returns the maximum value from selected database entries"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DMIN",
        short_description: Some("Returns the minimum value from selected database entries"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DOLLAR",
        short_description: Some("Converts a number to text, using the $ (dollar) currency format"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DOLLARDE",
        short_description: Some(
            "Converts a dollar price, expressed as a fraction, into a dollar price, expressed as a decimal number",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DOLLARFR",
        short_description: Some(
            "Converts a dollar price, expressed as a decimal number, into a dollar price, expressed as a fraction",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DPRODUCT",
        short_description: Some(
            "Multiplies the values in a particular field of records that match the criteria in a database",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DROP",
        short_description: Some(
            "Excludes a specified number of rows or columns from the start or end of an array",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DSTDEV",
        short_description: Some(
            "Estimates the standard deviation based on a sample of selected database entries",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DSTDEVP",
        short_description: Some(
            "Calculates the standard deviation based on the entire population of selected database entries",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DSUM",
        short_description: Some(
            "Adds the numbers in the field column of records in the database that match the criteria",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DURATION",
        short_description: Some(
            "Returns the annual duration of a security with periodic interest payments",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DVAR",
        short_description: Some(
            "Estimates variance based on a sample from selected database entries",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.DVARP",
        short_description: Some(
            "Calculates variance based on the entire population of selected database entries",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.EDATE",
        short_description: Some(
            "Returns the serial number of the date that is the indicated number of months before or after the start date",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.EFFECT",
        short_description: Some("Returns the effective annual interest rate"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ENCODEURL",
        short_description: Some("Returns a URL-encoded string"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.EOMONTH",
        short_description: Some(
            "Returns the serial number of the last day of the month before or after a specified number of months",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ERF",
        short_description: Some("Returns the error function"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ERF.PRECISE",
        short_description: Some("Returns the error function"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ERFC",
        short_description: Some("Returns the complementary error function"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ERFC.PRECISE",
        short_description: Some(
            "Returns the complementary ERF function integrated between x and infinity",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ERROR.TYPE",
        short_description: Some("Returns a number corresponding to an error type"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.EUROCONVERT",
        short_description: Some(
            "Converts a number to euros, converts a number from euros to a euro member currency, or converts a number from one euro member currency to another by using the euro as an intermediary (triangulation)",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.EVEN",
        short_description: Some("Rounds a number up to the nearest even integer"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.EXACT",
        short_description: Some("Checks to see if two text values are identical"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.EXP",
        short_description: Some("Returns e raised to the power of a given number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.EXPAND",
        short_description: Some("Expands or pads an array to specified row and column dimensions"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.EXPON.DIST",
        short_description: Some("Returns the exponential distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.EXPONDIST",
        short_description: Some("Returns the exponential distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.F.DIST",
        short_description: Some("Returns the F probability distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.F.DIST.RT",
        short_description: Some("Returns the F probability distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.F.INV",
        short_description: Some("Returns the inverse of the F probability distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.F.INV.RT",
        short_description: Some("Returns the inverse of the F probability distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.F.TEST",
        short_description: Some("Returns the result of an F-test"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FACT",
        short_description: Some("Returns the factorial of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FACTDOUBLE",
        short_description: Some("Returns the double factorial of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FALSE",
        short_description: Some("Returns the logical value FALSE"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FDIST",
        short_description: Some("Returns the F probability distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FILTER",
        short_description: Some(
            "Use this function to filter a range of data based on criteria you define.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FILTERXML",
        short_description: Some(
            "Returns specific data from the XML content by using the specified XPath",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FIND, FINDB",
        short_description: Some("Finds one text value within another (case-sensitive)"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FINV",
        short_description: Some("Returns the inverse of the F probability distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FISHER",
        short_description: Some("Returns the Fisher transformation"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FISHERINV",
        short_description: Some("Returns the inverse of the Fisher transformation"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FIXED",
        short_description: Some("Formats a number as text with a fixed number of decimals"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FLOOR",
        short_description: Some("Rounds a number down, toward zero"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FLOOR.MATH",
        short_description: Some(
            "Rounds a number down, to the nearest integer or to the nearest multiple of significance",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FLOOR.PRECISE",
        short_description: Some(
            "Rounds a number down to the nearest integer or to the nearest multiple of significance. Regardless of the sign of the number, the number is rounded down.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FORECAST",
        short_description: Some("Returns a value along a linear trend"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FORECAST.LINEAR",
        short_description: Some("Returns a value along a linear trend"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FORMULATEXT",
        short_description: Some("Returns the formula at the given reference as text"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FREQUENCY",
        short_description: Some("Returns a frequency distribution as a vertical array"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FTEST",
        short_description: Some("Returns the result of an F-test"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FV",
        short_description: Some("Returns the future value of an investment"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.FVSCHEDULE",
        short_description: Some(
            "Returns the future value of an initial principal after applying a series of compound interest rates",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.GAMMA",
        short_description: Some("Returns the Gamma function value"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.GAMMA.DIST",
        short_description: Some("Returns the gamma distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.GAMMA.INV",
        short_description: Some("Returns the inverse of the gamma cumulative distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.GAMMADIST",
        short_description: Some("Returns the gamma distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.GAMMAINV",
        short_description: Some("Returns the inverse of the gamma cumulative distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.GAMMALN",
        short_description: Some("Returns the natural logarithm of the gamma function, ?(x)"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.GAMMALN.PRECISE",
        short_description: Some("Returns the natural logarithm of the gamma function, ?(x)"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.GAUSS",
        short_description: Some(
            "Returns 0.5 less than the standard normal cumulative distribution",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.GCD",
        short_description: Some("Returns the greatest common divisor"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.GEOMEAN",
        short_description: Some("Returns the geometric mean"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.GESTEP",
        short_description: Some("Tests whether a number is greater than a threshold value"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.GETPIVOTDATA",
        short_description: Some("Returns data stored in a PivotTable report"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.GROUPBY",
        short_description: Some(
            "Helps a user group, aggregate, sort, and filter data based on the fields you specify",
        ),
        long_description: Some(
            "The first widened tranche uses GROUPBY as the grouped-aggregation representative for witness rollout.",
        ),
        parameters: &[
            ParameterHelpSeed {
                index: 0,
                name: "row_fields",
                short_description: Some("Grouping keys used to partition the input rows."),
            },
            ParameterHelpSeed {
                index: 1,
                name: "values",
                short_description: Some("Value vector or matrix aggregated within each group."),
            },
            ParameterHelpSeed {
                index: 2,
                name: "function",
                short_description: Some(
                    "Current parked baseline supports the admitted grouped-aggregation callable/aggregate slice.",
                ),
            },
        ],
    },
    RegistryHelpSeed {
        function_id: "FUNC.GROWTH",
        short_description: Some("Returns values along an exponential trend"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.HARMEAN",
        short_description: Some("Returns the harmonic mean"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.HEX2BIN",
        short_description: Some("Converts a hexadecimal number to binary"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.HEX2DEC",
        short_description: Some("Converts a hexadecimal number to decimal"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.HEX2OCT",
        short_description: Some("Converts a hexadecimal number to octal"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.HLOOKUP",
        short_description: Some(
            "Looks in the top row of an array and returns the value of the indicated cell",
        ),
        long_description: Some(
            "Current-baseline OxFunc support covers exact and approximate numeric lookup, wildcard text lookup on the exact lane, logical lookup values, and row-index validation.",
        ),
        parameters: &[
            ParameterHelpSeed {
                index: 1,
                name: "table_array",
                short_description: Some(
                    "Reference and array structure stays visible through the adapter seam.",
                ),
            },
            ParameterHelpSeed {
                index: 0,
                name: "lookup_value",
                short_description: Some(
                    "Supports exact and approximate lookup, including the current-baseline wildcard text lane.",
                ),
            },
            ParameterHelpSeed {
                index: 3,
                name: "range_lookup",
                short_description: Some("Omitted means approximate-match mode."),
            },
            ParameterHelpSeed {
                index: 2,
                name: "row_index_num",
                short_description: Some("Truncates toward zero before validation."),
            },
        ],
    },
    RegistryHelpSeed {
        function_id: "FUNC.HOUR",
        short_description: Some("Converts a serial number to an hour"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.HSTACK",
        short_description: Some(
            "Appends arrays horizontally and in sequence to return a larger array",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.HYPERLINK",
        short_description: Some(
            "Creates a shortcut or jump that opens a document stored on a network server, an intranet, or the Internet",
        ),
        long_description: Some(
            "The first widened tranche uses HYPERLINK as the presentation-aware value representative.",
        ),
        parameters: &[
            ParameterHelpSeed {
                index: 0,
                name: "link_location",
                short_description: Some("Target URL or workbook-local hyperlink location."),
            },
            ParameterHelpSeed {
                index: 1,
                name: "friendly_name",
                short_description: Some(
                    "Optional display value carried alongside hyperlink formatting hints.",
                ),
            },
        ],
    },
    RegistryHelpSeed {
        function_id: "FUNC.HYPGEOM.DIST",
        short_description: Some("Returns the hypergeometric distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.HYPGEOMDIST",
        short_description: Some("Returns the hypergeometric distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IF",
        short_description: Some(
            "Use this function to return one value if a condition is true and another value if it's false. Here's a video about using the IF function .",
        ),
        long_description: Some(
            "The first widened tranche uses IF as the control-flow anchor for mixed witness rollout.",
        ),
        parameters: &[
            ParameterHelpSeed {
                index: 0,
                name: "logical_test",
                short_description: Some("Condition evaluation drives the branch selection."),
            },
            ParameterHelpSeed {
                index: 1,
                name: "value_if_true",
                short_description: Some("Returned when the logical test evaluates truthy."),
            },
            ParameterHelpSeed {
                index: 2,
                name: "value_if_false",
                short_description: Some(
                    "Optional false branch; omission follows the current baseline IF defaulting rules.",
                ),
            },
        ],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IFERROR",
        short_description: Some(
            "Returns a value you specify if a formula evaluates to an error; otherwise, returns the result of the formula",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IFNA",
        short_description: Some(
            "Returns the value you specify if the expression resolves to #N/A, otherwise returns the result of the expression",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IFS",
        short_description: Some(
            "Checks whether one or more conditions are met and returns a value that corresponds to the first TRUE condition.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMABS",
        short_description: Some("Returns the absolute value (modulus) of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMAGE",
        short_description: Some("Returns an image from a given source"),
        long_description: Some(
            "The first widened tranche uses IMAGE as the rich-value publication representative rather than an ordinary scalar-return function.",
        ),
        parameters: &[
            ParameterHelpSeed {
                index: 1,
                name: "alt_text",
                short_description: Some("Optional alternate text for presentation-aware hosts."),
            },
            ParameterHelpSeed {
                index: 0,
                name: "source",
                short_description: Some("Image source or provider-backed locator string."),
            },
            ParameterHelpSeed {
                index: 2,
                name: "sizing",
                short_description: Some("Optional sizing mode selection."),
            },
            ParameterHelpSeed {
                index: 4,
                name: "width",
                short_description: Some("Optional explicit width input."),
            },
            ParameterHelpSeed {
                index: 3,
                name: "height",
                short_description: Some("Optional explicit height input."),
            },
        ],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMAGINARY",
        short_description: Some("Returns the imaginary coefficient of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMARGUMENT",
        short_description: Some("Returns the argument theta, an angle expressed in radians"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMCONJUGATE",
        short_description: Some("Returns the complex conjugate of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMCOS",
        short_description: Some("Returns the cosine of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMCOSH",
        short_description: Some("Returns the hyperbolic cosine of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMCOT",
        short_description: Some("Returns the cotangent of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMCSC",
        short_description: Some("Returns the cosecant of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMCSCH",
        short_description: Some("Returns the hyperbolic cosecant of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMDIV",
        short_description: Some("Returns the quotient of two complex numbers"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMEXP",
        short_description: Some("Returns the exponential of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMLN",
        short_description: Some("Returns the natural logarithm of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMLOG10",
        short_description: Some("Returns the base-10 logarithm of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMLOG2",
        short_description: Some("Returns the base-2 logarithm of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMPOWER",
        short_description: Some("Returns a complex number raised to an integer power"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMPRODUCT",
        short_description: Some("Returns the product of from 2 to 255 complex numbers"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMREAL",
        short_description: Some("Returns the real coefficient of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMSEC",
        short_description: Some("Returns the secant of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMSECH",
        short_description: Some("Returns the hyperbolic secant of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMSIN",
        short_description: Some("Returns the sine of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMSINH",
        short_description: Some("Returns the hyperbolic sine of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMSQRT",
        short_description: Some("Returns the square root of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMSUB",
        short_description: Some("Returns the difference between two complex numbers"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMSUM",
        short_description: Some("Returns the sum of complex numbers"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IMTAN",
        short_description: Some("Returns the tangent of a complex number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.INDEX",
        short_description: Some("Uses an index to choose a value from a reference or array"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.INDIRECT",
        short_description: Some("Returns a reference indicated by a text value"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.INFO",
        short_description: Some("Returns information about the current operating environment"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.INT",
        short_description: Some("Rounds a number down to the nearest integer"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.INTERCEPT",
        short_description: Some("Returns the intercept of the linear regression line"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.INTRATE",
        short_description: Some("Returns the interest rate for a fully invested security"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IPMT",
        short_description: Some(
            "Returns the interest payment for an investment for a given period",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.IRR",
        short_description: Some("Returns the internal rate of return for a series of cash flows"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISBLANK",
        short_description: Some("Returns TRUE if the value is blank"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISERR",
        short_description: Some("Returns TRUE if the value is any error value except #N/A"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISERROR",
        short_description: Some("Returns TRUE if the value is any error value"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISEVEN",
        short_description: Some("Returns TRUE if the number is even"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISFORMULA",
        short_description: Some(
            "Returns TRUE if there is a reference to a cell that contains a formula",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISLOGICAL",
        short_description: Some("Returns TRUE if the value is a logical value"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISNA",
        short_description: Some("Returns TRUE if the value is the #N/A error value"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISNONTEXT",
        short_description: Some("Returns TRUE if the value is not text"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISNUMBER",
        short_description: Some("Returns TRUE if the value is a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISO.CEILING",
        short_description: Some(
            "Returns a number that is rounded up to the nearest integer or to the nearest multiple of significance",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISODD",
        short_description: Some("Returns TRUE if the number is odd"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISOMITTED",
        short_description: Some(
            "Checks whether the value in a LAMBDA is missing and returns TRUE or FALSE",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISOWEEKNUM",
        short_description: Some(
            "Returns the number of the ISO week number of the year for a given date",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISPMT",
        short_description: Some(
            "Calculates the interest paid during a specific period of an investment",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISREF",
        short_description: Some("Returns TRUE if the value is a reference"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ISTEXT",
        short_description: Some("Returns TRUE if the value is text"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.KURT",
        short_description: Some("Returns the kurtosis of a data set"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LAMBDA",
        short_description: Some(
            "Create custom, reusable functions and call them by a friendly name",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LARGE",
        short_description: Some("Returns the k-th largest value in a data set"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LCM",
        short_description: Some("Returns the least common multiple"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LEFT, LEFTB",
        short_description: Some("Returns the leftmost characters from a text value"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LEN, LENB",
        short_description: Some("Returns the number of characters in a text string"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LET",
        short_description: Some("Use this function to assign names to calculation results."),
        long_description: Some(
            "The first widened tranche uses LET as the callable/helper formation representative without reopening the cross-repo callable freeze.",
        ),
        parameters: &[
            ParameterHelpSeed {
                index: 0,
                name: "name1",
                short_description: Some("Introduces the first lexical binding."),
            },
            ParameterHelpSeed {
                index: 1,
                name: "value1",
                short_description: Some("Supplies the first bound value or expression result."),
            },
            ParameterHelpSeed {
                index: 2,
                name: "calculation_or_name2",
                short_description: Some(
                    "Either begins the final calculation or continues the binding chain.",
                ),
            },
        ],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LINEST",
        short_description: Some("Returns the parameters of a linear trend"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LN",
        short_description: Some("Returns the natural logarithm of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LOG",
        short_description: Some("Returns the logarithm of a number to a specified base"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LOG10",
        short_description: Some("Returns the base-10 logarithm of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LOGEST",
        short_description: Some("Returns the parameters of an exponential trend"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LOGINV",
        short_description: Some(
            "Returns the inverse of the lognormal cumulative distribution function",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LOGNORM.DIST",
        short_description: Some("Returns the cumulative lognormal distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LOGNORM.INV",
        short_description: Some("Returns the inverse of the lognormal cumulative distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LOGNORMDIST",
        short_description: Some("Returns the cumulative lognormal distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LOOKUP",
        short_description: Some("Looks up values in a vector or array"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.LOWER",
        short_description: Some("Converts text to lowercase"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MAKEARRAY",
        short_description: Some(
            "Returns a calculated array of a specified row and column size, by applying a LAMBDA",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MAP",
        short_description: Some(
            "Returns an array formed by mapping each value in the array(s) to a new value by applying a LAMBDA to create a new value",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MATCH",
        short_description: Some("Looks up values in a reference or array"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MAX",
        short_description: Some("Returns the maximum value in a list of arguments"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MAXA",
        short_description: Some(
            "Returns the maximum value in a list of arguments, including numbers, text, and logical values",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MAXIFS",
        short_description: Some(
            "Returns the maximum value among cells specified by a given set of conditions or criteria",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MDETERM",
        short_description: Some("Returns the matrix determinant of an array"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MDURATION",
        short_description: Some(
            "Returns the Macauley modified duration for a security with an assumed par value of $100",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MEDIAN",
        short_description: Some("Returns the median of the given numbers"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MID, MIDB",
        short_description: Some(
            "Returns a specific number of characters from a text string starting at the position you specify",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MIN",
        short_description: Some("Returns the minimum value in a list of arguments"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MINA",
        short_description: Some(
            "Returns the smallest value in a list of arguments, including numbers, text, and logical values",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MINIFS",
        short_description: Some(
            "Returns the minimum value among cells specified by a given set of conditions or criteria.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MINUTE",
        short_description: Some("Converts a serial number to a minute"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MINVERSE",
        short_description: Some("Returns the matrix inverse of an array"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MIRR",
        short_description: Some(
            "Returns the internal rate of return where positive and negative cash flows are financed at different rates",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MMULT",
        short_description: Some("Returns the matrix product of two arrays"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MOD",
        short_description: Some("Returns the remainder from division"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MODE",
        short_description: Some("Returns the most common value in a data set"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MODE.MULT",
        short_description: Some(
            "Returns a vertical array of the most frequently occurring, or repetitive values in an array or range of data",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MODE.SNGL",
        short_description: Some("Returns the most common value in a data set"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MONTH",
        short_description: Some("Converts a serial number to a month"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MROUND",
        short_description: Some("Returns a number rounded to the desired multiple"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MULTINOMIAL",
        short_description: Some("Returns the multinomial of a set of numbers"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.MUNIT",
        short_description: Some("Returns the unit matrix or the specified dimension"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.N",
        short_description: Some("Returns a value converted to a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NA",
        short_description: Some("Returns the error value #N/A"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NEGBINOM.DIST",
        short_description: Some("Returns the negative binomial distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NEGBINOMDIST",
        short_description: Some("Returns the negative binomial distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NETWORKDAYS",
        short_description: Some("Returns the number of whole workdays between two dates"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NETWORKDAYS.INTL",
        short_description: Some(
            "Returns the number of whole workdays between two dates using parameters to indicate which and how many days are weekend days",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NOMINAL",
        short_description: Some("Returns the annual nominal interest rate"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NORM.DIST",
        short_description: Some("Returns the normal cumulative distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NORM.INV",
        short_description: Some("Returns the inverse of the normal cumulative distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NORM.S.DIST",
        short_description: Some("Returns the standard normal cumulative distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NORM.S.INV",
        short_description: Some(
            "Returns the inverse of the standard normal cumulative distribution",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NORMDIST",
        short_description: Some("Returns the normal cumulative distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NORMINV",
        short_description: Some("Returns the inverse of the normal cumulative distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NORMSDIST",
        short_description: Some("Returns the standard normal cumulative distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NORMSINV",
        short_description: Some(
            "Returns the inverse of the standard normal cumulative distribution",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NOT",
        short_description: Some("Reverses the logic of its argument"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NOW",
        short_description: Some("Returns the serial number of the current date and time"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NPER",
        short_description: Some("Returns the number of periods for an investment"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NPV",
        short_description: Some(
            "Returns the net present value of an investment based on a series of periodic cash flows and a discount rate",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.NUMBERVALUE",
        short_description: Some("Converts text to number in a locale-independent manner"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.OCT2BIN",
        short_description: Some("Converts an octal number to binary"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.OCT2DEC",
        short_description: Some("Converts an octal number to decimal"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.OCT2HEX",
        short_description: Some("Converts an octal number to hexadecimal"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ODD",
        short_description: Some("Rounds a number up to the nearest odd integer"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ODDFPRICE",
        short_description: Some(
            "Returns the price per $100 face value of a security with an odd first period",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ODDFYIELD",
        short_description: Some("Returns the yield of a security with an odd first period"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ODDLPRICE",
        short_description: Some(
            "Returns the price per $100 face value of a security with an odd last period",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ODDLYIELD",
        short_description: Some("Returns the yield of a security with an odd last period"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.OFFSET",
        short_description: Some("Returns a reference offset from a given reference"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.OP_IMPLICIT_INTERSECTION",
        short_description: Some(
            "Projects the implicit-intersection result for a reference or array in the current formula context.",
        ),
        long_description: Some(
            "The first widened tranche uses OP_IMPLICIT_INTERSECTION as the modeled operator representative.",
        ),
        parameters: &[ParameterHelpSeed {
            index: 0,
            name: "reference_or_array",
            short_description: Some(
                "The operator projects an implicit-intersection scalar from the current context.",
            ),
        }],
    },
    RegistryHelpSeed {
        function_id: "FUNC.OR",
        short_description: Some("Returns TRUE if any argument is TRUE"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PDURATION",
        short_description: Some(
            "Returns the number of periods required by an investment to reach a specified value",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PEARSON",
        short_description: Some("Returns the Pearson product moment correlation coefficient"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PERCENTILE",
        short_description: Some("Returns the k-th percentile of values in a range"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PERCENTILE.EXC",
        short_description: Some(
            "Returns the k-th percentile of values in a range, where k is in the range 0..1, exclusive",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PERCENTILE.INC",
        short_description: Some("Returns the k-th percentile of values in a range"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PERCENTOF",
        short_description: Some("Sums the values in the subset and divides it by all the values"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PERCENTRANK",
        short_description: Some("Returns the percentage rank of a value in a data set"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PERCENTRANK.EXC",
        short_description: Some(
            "Returns the rank of a value in a data set as a percentage (0..1, exclusive) of the data set",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PERCENTRANK.INC",
        short_description: Some("Returns the percentage rank of a value in a data set"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PERMUT",
        short_description: Some("Returns the number of permutations for a given number of objects"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PERMUTATIONA",
        short_description: Some(
            "Returns the number of permutations for a given number of objects (with repetitions) that can be selected from the total objects",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PHI",
        short_description: Some(
            "Returns the value of the density function for a standard normal distribution",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PHONETIC",
        short_description: Some("Extracts the phonetic (furigana) characters from a text string"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PI",
        short_description: Some("Returns the value of pi"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PIVOTBY",
        short_description: Some(
            "Helps a user group, aggregate, sort, and filter data based on the row and column fields that you specify",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PMT",
        short_description: Some("Returns the periodic payment for an annuity"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.POISSON",
        short_description: Some("Returns the Poisson distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.POISSON.DIST",
        short_description: Some("Returns the Poisson distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.POWER",
        short_description: Some("Returns the result of a number raised to a power"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PPMT",
        short_description: Some(
            "Returns the payment on the principal for an investment for a given period",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PRICE",
        short_description: Some(
            "Returns the price per $100 face value of a security that pays periodic interest",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PRICEDISC",
        short_description: Some("Returns the price per $100 face value of a discounted security"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PRICEMAT",
        short_description: Some(
            "Returns the price per $100 face value of a security that pays interest at maturity",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PROB",
        short_description: Some(
            "Returns the probability that values in a range are between two limits",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PRODUCT",
        short_description: Some("Multiplies its arguments"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PROPER",
        short_description: Some("Capitalizes the first letter in each word of a text value"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.PV",
        short_description: Some("Returns the present value of an investment"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.QUARTILE",
        short_description: Some("Returns the quartile of a data set"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.QUARTILE.EXC",
        short_description: Some(
            "Returns the quartile of the data set, based on percentile values from 0..1, exclusive",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.QUARTILE.INC",
        short_description: Some("Returns the quartile of a data set"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.QUOTIENT",
        short_description: Some("Returns the integer portion of a division"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.RADIANS",
        short_description: Some("Converts degrees to radians"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.RAND",
        short_description: Some("Returns a random number between 0 and 1"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.RANDARRAY",
        short_description: Some(
            "Returns an array of random numbers between 0 and 1. However, you can specify the number of rows and columns to fill, minimum and maximum values, and whether to return whole numbers or decimal values.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.RANDBETWEEN",
        short_description: Some("Returns a random number between the numbers you specify"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.RANK",
        short_description: Some("Returns the rank of a number in a list of numbers"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.RANK.AVG",
        short_description: Some("Returns the rank of a number in a list of numbers"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.RANK.EQ",
        short_description: Some("Returns the rank of a number in a list of numbers"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.RATE",
        short_description: Some("Returns the interest rate per period of an annuity"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.RECEIVED",
        short_description: Some(
            "Returns the amount received at maturity for a fully invested security",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.REDUCE",
        short_description: Some(
            "Reduces an array to an accumulated value by applying a LAMBDA to each value and returning the total value in the accumulator",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.REGEXEXTRACT",
        short_description: Some(
            "Extracts strings within the provided text that matches the pattern",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.REGEXREPLACE",
        short_description: Some(
            "Replaces strings within the provided text that matches the pattern with replacement",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.REGEXTEST",
        short_description: Some("Determines whether any part of text matches the pattern"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.REGISTER.ID",
        short_description: Some(
            "Returns the register ID of the specified dynamic link library (DLL) or code resource that has been previously registered",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.REPLACE, REPLACEB",
        short_description: Some("Replaces characters within text"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.REPT",
        short_description: Some("Repeats text a given number of times"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.RIGHT, RIGHTB",
        short_description: Some("Returns the rightmost characters from a text value"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ROMAN",
        short_description: Some("Converts an Arabic numeral to Roman, as text"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ROUND",
        short_description: Some("Rounds a number to a specified number of digits"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ROUNDDOWN",
        short_description: Some("Rounds a number down, toward zero"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ROUNDUP",
        short_description: Some("Rounds a number up, away from zero"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ROW",
        short_description: Some("Returns the row number of a reference"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ROWS",
        short_description: Some("Returns the number of rows in a reference"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.RRI",
        short_description: Some(
            "Returns an equivalent interest rate for the growth of an investment",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.RSQ",
        short_description: Some(
            "Returns the square of the Pearson product moment correlation coefficient",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.RTD",
        short_description: Some(
            "Retrieves real-time data from a program that supports COM automation",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SCAN",
        short_description: Some(
            "Scans an array by applying a LAMBDA to each value and returns an array that has each intermediate value",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SEARCH, SEARCHB",
        short_description: Some("Finds one text value within another (not case-sensitive)"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SEC",
        short_description: Some("Returns the secant of an angle"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SECH",
        short_description: Some("Returns the hyperbolic secant of an angle"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SECOND",
        short_description: Some("Converts a serial number to a second"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SEQUENCE",
        short_description: Some(
            "Generates a list of sequential numbers in an array, such as 1, 2, 3, 4",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SERIESSUM",
        short_description: Some("Returns the sum of a power series based on the formula"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SHEET",
        short_description: Some("Returns the sheet number of the referenced sheet"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SHEETS",
        short_description: Some("Returns the number of sheets in a reference"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SIGN",
        short_description: Some("Returns the sign of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SIN",
        short_description: Some("Returns the sine of the given angle"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SINH",
        short_description: Some("Returns the hyperbolic sine of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SKEW",
        short_description: Some("Returns the skewness of a distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SKEW.P",
        short_description: Some(
            "Returns the skewness of a distribution based on a population: a characterization of the degree of asymmetry of a distribution around its mean",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SLN",
        short_description: Some(
            "Returns the straight-line depreciation of an asset for one period",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SLOPE",
        short_description: Some("Returns the slope of the linear regression line"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SMALL",
        short_description: Some("Returns the k-th smallest value in a data set"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SORT",
        short_description: Some("Sorts the contents of a range or array"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SORTBY",
        short_description: Some(
            "Sorts the contents of a range or array based on the values in a corresponding range or array",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SQRT",
        short_description: Some("Returns a positive square root"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SQRTPI",
        short_description: Some("Returns the square root of (number * pi)"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.STANDARDIZE",
        short_description: Some("Returns a normalized value"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.STDEV",
        short_description: Some("Estimates standard deviation based on a sample"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.STDEV.P",
        short_description: Some("Calculates standard deviation based on the entire population"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.STDEV.S",
        short_description: Some("Estimates standard deviation based on a sample"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.STDEVA",
        short_description: Some(
            "Estimates standard deviation based on a sample, including numbers, text, and logical values",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.STDEVP",
        short_description: Some("Calculates standard deviation based on the entire population"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.STDEVPA",
        short_description: Some(
            "Calculates standard deviation based on the entire population, including numbers, text, and logical values",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.STEYX",
        short_description: Some(
            "Returns the standard error of the predicted y-value for each x in the regression",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.STOCKHISTORY",
        short_description: Some("Retrieves historical data about a financial instrument"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SUBSTITUTE",
        short_description: Some("Substitutes new text for old text in a text string"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SUBTOTAL",
        short_description: Some("Returns a subtotal in a list or database"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SUM",
        short_description: Some("Use this function to add the values in cells."),
        long_description: Some(
            "The first mixed tranche treats SUM as the ordinary scalar/aggregate anchor for downstream witness rendering.",
        ),
        parameters: &[
            ParameterHelpSeed {
                index: 0,
                name: "number1",
                short_description: Some(
                    "Starts the additive fold and admits ordinary scalar and reference-fed lanes.",
                ),
            },
            ParameterHelpSeed {
                index: 1,
                name: "number2",
                short_description: Some(
                    "Represents the repeated optional additive arguments for the bounded witness slice.",
                ),
            },
        ],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SUMIF",
        short_description: Some("Adds the cells specified by a given criteria"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SUMIFS",
        short_description: Some(
            "Use this function when you need to add the cells in a range that meet multiple criteria.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SUMPRODUCT",
        short_description: Some(
            "Returns the sum of the products of corresponding array components",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SUMSQ",
        short_description: Some("Returns the sum of the squares of the arguments"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SUMX2MY2",
        short_description: Some(
            "Returns the sum of the difference of squares of corresponding values in two arrays",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SUMX2PY2",
        short_description: Some(
            "Returns the sum of the sum of squares of corresponding values in two arrays",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SUMXMY2",
        short_description: Some(
            "Returns the sum of squares of differences of corresponding values in two arrays",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SWITCH",
        short_description: Some(
            "Evaluates an expression against a list of values and returns the result corresponding to the first matching value. If there is no match, an optional default value may be returned.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.SYD",
        short_description: Some(
            "Returns the sum-of-years' digits depreciation of an asset for a specified period",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.T",
        short_description: Some("Converts its arguments to text"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.T.DIST",
        short_description: Some(
            "Returns the Percentage Points (probability) for the Student t-distribution",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.T.DIST.2T",
        short_description: Some(
            "Returns the Percentage Points (probability) for the Student t-distribution",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.T.DIST.RT",
        short_description: Some("Returns the Student's t-distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.T.INV",
        short_description: Some(
            "Returns the t-value of the Student's t-distribution as a function of the probability and the degrees of freedom",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.T.INV.2T",
        short_description: Some("Returns the inverse of the Student's t-distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.T.TEST",
        short_description: Some("Returns the probability associated with a Student's t-test"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TAKE",
        short_description: Some(
            "Returns a specified number of contiguous rows or columns from the start or end of an array",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TAN",
        short_description: Some("Returns the tangent of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TANH",
        short_description: Some("Returns the hyperbolic tangent of a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TBILLEQ",
        short_description: Some("Returns the bond-equivalent yield for a Treasury bill"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TBILLPRICE",
        short_description: Some("Returns the price per $100 face value for a Treasury bill"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TBILLYIELD",
        short_description: Some("Returns the yield for a Treasury bill"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TDIST",
        short_description: Some("Returns the Student's t-distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TEXT",
        short_description: Some("Formats a number and converts it to text"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TEXTAFTER",
        short_description: Some("Returns text that occurs after given character or string"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TEXTBEFORE",
        short_description: Some(
            "Use this function to return text that occurs before a given character or string. You can use TEXTAFTER to return text that occurs after a given character or string.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TEXTJOIN",
        short_description: Some("Text: Combines the text from multiple ranges and/or strings"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TEXTSPLIT",
        short_description: Some("Splits text strings by using column and row delimiters"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TIME",
        short_description: Some("Returns the serial number of a particular time"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TIMEVALUE",
        short_description: Some("Converts a time in the form of text to a serial number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TINV",
        short_description: Some("Returns the inverse of the Student's t-distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TOCOL",
        short_description: Some("Returns the array in a single column"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TODAY",
        short_description: Some("Returns the serial number of today's date"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TOROW",
        short_description: Some("Returns the array in a single row"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TRANSLATE",
        short_description: Some("Translates a text from one language to another"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TRANSPOSE",
        short_description: Some("Returns the transpose of an array"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TREND",
        short_description: Some("Returns values along a linear trend"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TRIM",
        short_description: Some("Removes spaces from text"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TRIMMEAN",
        short_description: Some("Returns the mean of the interior of a data set"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TRIMRANGE",
        short_description: Some(
            "Scans in from the edges of a range or array until it finds a non-blank cell (or value), it then excludes those blank rows or columns",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TRUE",
        short_description: Some("Returns the logical value TRUE"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TRUNC",
        short_description: Some("Truncates a number to an integer"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TTEST",
        short_description: Some("Returns the probability associated with a Student's t-test"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.TYPE",
        short_description: Some("Returns a number indicating the data type of a value"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.UNICHAR",
        short_description: Some(
            "Returns the Unicode character that is references by the given numeric value",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.UNICODE",
        short_description: Some(
            "Returns the number (code point) that corresponds to the first character of the text",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.UNIQUE",
        short_description: Some(
            "Use this function to return a list of unique values in a list or range.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.UPPER",
        short_description: Some("Converts text to uppercase"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.VALUE",
        short_description: Some("Converts a text argument to a number"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.VALUETOTEXT",
        short_description: Some("Returns text from any specified value"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.VAR",
        short_description: Some("Estimates variance based on a sample"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.VAR.P",
        short_description: Some("Calculates variance based on the entire population"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.VAR.S",
        short_description: Some("Estimates variance based on a sample"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.VARA",
        short_description: Some(
            "Estimates variance based on a sample, including numbers, text, and logical values",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.VARP",
        short_description: Some("Calculates variance based on the entire population"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.VARPA",
        short_description: Some(
            "Calculates variance based on the entire population, including numbers, text, and logical values",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.VDB",
        short_description: Some(
            "Returns the depreciation of an asset for a specified or partial period by using a declining balance method",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.VLOOKUP",
        short_description: Some(
            "Looks in the first column of an array and moves across the row to return the value of a cell",
        ),
        long_description: Some(
            "Current-baseline OxFunc support covers exact and approximate numeric lookup, wildcard text lookup on the exact lane, logical lookup values, and column-index validation.",
        ),
        parameters: &[
            ParameterHelpSeed {
                index: 1,
                name: "table_array",
                short_description: Some(
                    "Reference and array structure stays visible through the adapter seam.",
                ),
            },
            ParameterHelpSeed {
                index: 0,
                name: "lookup_value",
                short_description: Some(
                    "Supports exact and approximate lookup, including the current-baseline wildcard text lane.",
                ),
            },
            ParameterHelpSeed {
                index: 3,
                name: "range_lookup",
                short_description: Some("Omitted means approximate-match mode."),
            },
            ParameterHelpSeed {
                index: 2,
                name: "col_index_num",
                short_description: Some("Truncates toward zero before validation."),
            },
        ],
    },
    RegistryHelpSeed {
        function_id: "FUNC.VSTACK",
        short_description: Some(
            "Appends arrays vertically and in sequence to return a larger array",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.WEBSERVICE",
        short_description: Some("Returns data from a web service"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.WEEKDAY",
        short_description: Some("Converts a serial number to a day of the week"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.WEEKNUM",
        short_description: Some(
            "Converts a serial number to a number representing where the week falls numerically with a year",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.WEIBULL",
        short_description: Some("Returns the Weibull distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.WEIBULL.DIST",
        short_description: Some("Returns the Weibull distribution"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.WORKDAY",
        short_description: Some(
            "Returns the serial number of the date before or after a specified number of workdays",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.WORKDAY.INTL",
        short_description: Some(
            "Returns the serial number of the date before or after a specified number of workdays using parameters to indicate which and how many days are weekend days",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.WRAPCOLS",
        short_description: Some(
            "Wraps the provided row or column of values by columns after a specified number of elements",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.WRAPROWS",
        short_description: Some(
            "Wraps the provided row or column of values by rows after a specified number of elements",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.XIRR",
        short_description: Some(
            "Returns the internal rate of return for a schedule of cash flows that is not necessarily periodic",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.XLOOKUP",
        short_description: Some(
            "Use this function when you need to search a range or an array, and return an item corresponding to the first match it finds. If a match doesn't exist, then XLOOKUP can return the closest (approximate) match.",
        ),
        long_description: Some(
            "The first widened tranche uses XLOOKUP as the modern lookup representative beyond the bounded HVLOOKUP family seed.",
        ),
        parameters: &[
            ParameterHelpSeed {
                index: 2,
                name: "return_array",
                short_description: Some("Return vector or area paired to the lookup array."),
            },
            ParameterHelpSeed {
                index: 1,
                name: "lookup_array",
                short_description: Some("Lookup vector used for match search."),
            },
            ParameterHelpSeed {
                index: 0,
                name: "lookup_value",
                short_description: Some(
                    "Current mixed tranche carries the modern lookup-value entry point.",
                ),
            },
            ParameterHelpSeed {
                index: 5,
                name: "search_mode",
                short_description: Some(
                    "Controls forward, reverse, and binary-style search direction.",
                ),
            },
            ParameterHelpSeed {
                index: 4,
                name: "match_mode",
                short_description: Some("Controls exact, approximate, and wildcard matching."),
            },
            ParameterHelpSeed {
                index: 3,
                name: "if_not_found",
                short_description: Some("Optional not-found projection lane."),
            },
        ],
    },
    RegistryHelpSeed {
        function_id: "FUNC.XMATCH",
        short_description: Some(
            "Returns the relative position of an item in an array or range of cells.",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.XNPV",
        short_description: Some(
            "Returns the net present value for a schedule of cash flows that is not necessarily periodic",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.XOR",
        short_description: Some("Returns a logical exclusive OR of all arguments"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.YEAR",
        short_description: Some("Converts a serial number to a year"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.YEARFRAC",
        short_description: Some(
            "Returns the year fraction representing the number of whole days between start_date and end_date",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.YIELD",
        short_description: Some("Returns the yield on a security that pays periodic interest"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.YIELDDISC",
        short_description: Some(
            "Returns the annual yield for a discounted security; for example, a Treasury bill",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.YIELDMAT",
        short_description: Some(
            "Returns the annual yield of a security that pays interest at maturity",
        ),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.Z.TEST",
        short_description: Some("Returns the one-tailed probability-value of a z-test"),
        long_description: None,
        parameters: &[],
    },
    RegistryHelpSeed {
        function_id: "FUNC.ZTEST",
        short_description: Some("Returns the one-tailed probability-value of a z-test"),
        long_description: None,
        parameters: &[],
    },
];

pub(crate) fn registry_help_seed_for_id(function_id: &str) -> Option<&'static RegistryHelpSeed> {
    REGISTRY_HELP_SEEDS
        .iter()
        .find(|seed| seed.function_id.eq_ignore_ascii_case(function_id))
}
