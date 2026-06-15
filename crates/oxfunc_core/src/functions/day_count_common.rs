use crate::locale_format::{WorkbookDateSystem, ymd_from_excel_serial};
use crate::value::WorksheetErrorCode;

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            if leap { 29 } else { 28 }
        }
        _ => 30,
    }
}

/// NASD US (30/360) day count between two Excel serial dates.
///
/// Excel's basis-0 rule rolls the end date to the 1st of the following month
/// when the end day is 31 and the (already adjusted) start day is below 30,
/// rather than collapsing the end day to 30 in place. The month roll keeps the
/// count non-negative for forward ranges; the in-place form produces short and
/// even negative counts for same-month spans.
pub fn us_30_360(start: i64, end: i64) -> Result<f64, WorksheetErrorCode> {
    let (sy, sm, mut sd) = ymd_from_excel_serial(WorkbookDateSystem::System1900, start as f64)
        .ok_or(WorksheetErrorCode::Value)?;
    let (ey, em, mut ed) = ymd_from_excel_serial(WorkbookDateSystem::System1900, end as f64)
        .ok_or(WorksheetErrorCode::Value)?;

    let start_last_feb = sm == 2 && sd == days_in_month(sy, sm);
    let end_last_feb = em == 2 && ed == days_in_month(ey, em);

    if sd == 31 || start_last_feb {
        sd = 30;
    }
    if ed == 31 {
        if sd < 30 {
            let (ny, nm) = if em == 12 { (ey + 1, 1) } else { (ey, em + 1) };
            return Ok((ny - sy) as f64 * 360.0 + (nm - sm) as f64 * 30.0 + (1 - sd) as f64);
        }
        ed = 30;
    }
    if end_last_feb && start_last_feb {
        ed = 30;
    }

    Ok(((ey - sy) * 360 + (em - sm) * 30 + (ed - sd)) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale_format::excel_serial_from_ymd;

    fn serial(year: i64, month: i64, day: i64) -> i64 {
        excel_serial_from_ymd(WorkbookDateSystem::System1900, year, month, day).unwrap() as i64
    }

    // Equivalent to the in-place ed==31 && sd>=30 -> ed=30 form used by
    // amor_depreciation_family::days360_us, kept here only to assert agreement.
    fn amor_form(start: i64, end: i64) -> f64 {
        let (sy, sm, mut sd) =
            ymd_from_excel_serial(WorkbookDateSystem::System1900, start as f64).unwrap();
        let (ey, em, mut ed) =
            ymd_from_excel_serial(WorkbookDateSystem::System1900, end as f64).unwrap();
        let start_last_feb = sm == 2 && sd == days_in_month(sy, sm);
        let end_last_feb = em == 2 && ed == days_in_month(ey, em);
        if sd == 31 || start_last_feb {
            sd = 30;
        }
        if ed == 31 && sd >= 30 {
            ed = 30;
        }
        if end_last_feb && start_last_feb {
            ed = 30;
        }
        ((ey - sy) * 360 + (em - sm) * 30 + (ed - sd)) as f64
    }

    #[test]
    fn end_day_31_rolls_into_next_month() {
        assert_eq!(
            us_30_360(serial(2023, 11, 15), serial(2024, 1, 31)),
            Ok(76.0)
        );
    }

    #[test]
    fn same_month_forward_range_is_non_negative() {
        assert_eq!(
            us_30_360(serial(2024, 1, 15), serial(2024, 1, 31)),
            Ok(16.0)
        );
    }

    #[test]
    fn start_at_least_30_matches_amor_in_place_form() {
        for (s, e) in [
            (serial(2024, 1, 30), serial(2024, 3, 31)),
            (serial(2024, 1, 31), serial(2024, 5, 31)),
            (serial(2023, 12, 31), serial(2024, 3, 31)),
        ] {
            assert_eq!(
                us_30_360(s, e),
                Ok(amor_form(s, e)),
                "start>=30 case {s}->{e}"
            );
        }
    }

    #[test]
    fn december_31_end_rolls_across_year_boundary() {
        assert_eq!(
            us_30_360(serial(2023, 11, 15), serial(2023, 12, 31)),
            Ok(46.0)
        );
    }
}
