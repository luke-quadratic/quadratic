use std::collections::HashMap;

use super::A1Error;
use crate::grid::SheetId;

/// Map from sheet name to ID.
///
/// Note, we cannot use a reverse map because TS does
/// not support structs as keys.
///
/// Sheet names should be case-folded using [`crate::util::case_fold()`].
pub type SheetNameIdMap = HashMap<String, SheetId>;

/// Parses the sheet name from an A1 cell reference string. If the string does
/// not contain `!`, then this function returns `Ok((None, _))`. If the string
/// does contain `!`, then the left side is parsed as a sheet name that may be
/// quoted or unquoted.
pub(crate) fn parse_optional_sheet_name(a1: &str) -> Result<(Option<String>, &str), A1Error> {
    let Some((sheet_name, rest)) = a1.rsplit_once('!') else {
        return Ok((None, a1));
    };

    let sheet_name = if let Some(inner) = Some(sheet_name)
        .and_then(|s| s.strip_prefix('\''))
        .and_then(|s| s.strip_suffix('\''))
    {
        inner // single quotes removed
    } else if sheet_name.contains(' ') || sheet_name.contains('!') {
        return Err(A1Error::InvalidSheetNameMissingQuotes(
            sheet_name.to_string(),
        ));
    } else if sheet_name.contains('\'') {
        return Err(A1Error::MismatchedQuotes(sheet_name.to_string()));
    } else {
        sheet_name
    };

    Ok((Some(sheet_name.to_string()), rest))
}

pub(crate) fn parse_optional_sheet_name_to_id<'a>(
    a1: &'a str,
    default_sheet_id: &SheetId,
    sheet_map: &SheetNameIdMap,
) -> Result<(SheetId, &'a str), A1Error> {
    let (sheet_name, rest) = parse_optional_sheet_name(a1)?;
    let sheet_id = match sheet_name {
        Some(sheet_name) => {
            let folded_name = sheet_name.to_lowercase();
            **sheet_map
                .iter()
                .map(|(k, v)| (k.to_lowercase(), v))
                .collect::<HashMap<_, _>>()
                .get(&folded_name)
                .ok_or(A1Error::InvalidSheetName(sheet_name))?
        }
        None => default_sheet_id.to_owned(),
    };
    Ok((sheet_id, rest))
}

/// Returns a sheet name, quoted if necessary.
pub(crate) fn quote_sheet_name(sheet_name: &str) -> String {
    if sheet_name_must_be_quoted(sheet_name) {
        format!("'{}'", sheet_name.replace("'", "''"))
    } else {
        sheet_name.to_string()
    }
}

/// Returns whether a sheet name must be quoted.
fn sheet_name_must_be_quoted(sheet_name: &str) -> bool {
    sheet_name.starts_with(|c: char| c.is_ascii_digit())
        || sheet_name
            .chars()
            .any(|c| c.is_whitespace() || c.is_ascii_punctuation())
}

#[cfg(test)]
#[serial_test::parallel]
mod tests {
    use crate::grid::Sheet;

    use super::*;

    #[test]
    fn test_parse_optional_sheet_name() {
        assert_eq!(
            parse_optional_sheet_name("Sheet1!A1"),
            Ok((Some("Sheet1".to_string()), "A1"))
        );
        assert_eq!(
            parse_optional_sheet_name("sheet1!A1"),
            Ok((Some("sheet1".to_string()), "A1"))
        );
        assert_eq!(
            parse_optional_sheet_name("'Sheet 1'!A1"),
            Ok((Some("Sheet 1".to_string()), "A1"))
        );
        assert_eq!(parse_optional_sheet_name("A1"), Ok((None, "A1")));
        assert_eq!(
            parse_optional_sheet_name("'Sheet with ! mark'!A1"),
            Ok((Some("Sheet with ! mark".to_string()), "A1"))
        );
        assert_eq!(
            parse_optional_sheet_name("Sheet 1!A1"),
            Err(A1Error::InvalidSheetNameMissingQuotes(
                "Sheet 1".to_string()
            ))
        );
        assert_eq!(
            parse_optional_sheet_name("Sheet1!Sheet2!A1"),
            Err(A1Error::InvalidSheetNameMissingQuotes(
                "Sheet1!Sheet2".to_string()
            ))
        );
        assert_eq!(
            parse_optional_sheet_name("'Sheet1'!A1"),
            Ok((Some("Sheet1".to_string()), "A1"))
        );
        assert_eq!(
            parse_optional_sheet_name("'Sheet1!A1"),
            Err(A1Error::MismatchedQuotes("'Sheet1".to_string()))
        );
    }

    #[test]
    fn test_parse_optional_sheet_id() {
        let sheet_1 = SheetId::new();
        let sheet_2 = SheetId::new();
        let map = HashMap::from([
            ("sheet1".to_string(), sheet_1),
            ("sheet 2".to_string(), sheet_2),
        ]);
        assert_eq!(
            parse_optional_sheet_name_to_id("SHEET1!A1", &sheet_1, &map),
            Ok((sheet_1, "A1"))
        );
        assert_eq!(
            parse_optional_sheet_name_to_id("'SHEET 2'!A1", &sheet_1, &map),
            Ok((sheet_2, "A1"))
        );
        assert_eq!(
            parse_optional_sheet_name_to_id("A1", &sheet_1, &map),
            Ok((sheet_1, "A1"))
        );
        assert_eq!(
            parse_optional_sheet_name_to_id("Sheet1!A1:B2", &sheet_1, &map),
            Ok((sheet_1, "A1:B2"))
        );
        assert_eq!(
            parse_optional_sheet_name_to_id("Sheet1!Sheet2A1", &sheet_1, &map),
            Ok((sheet_1, "Sheet2A1"))
        );
        assert_eq!(
            parse_optional_sheet_name_to_id("Sheet 1!A1", &sheet_1, &map),
            Err(A1Error::InvalidSheetNameMissingQuotes(
                "Sheet 1".to_string()
            ))
        );
        assert_eq!(
            parse_optional_sheet_name_to_id("sheet1!A1", &sheet_1, &map),
            Ok((sheet_1, "A1"))
        );
    }

    #[test]
    fn test_parse_long_sheet_name() {
        let mut sheet = Sheet::test();
        sheet.name = "Types: sequences, mapping, sets".to_string();
        let map = HashMap::from([("Types: sequences, mapping, sets".to_string(), sheet.id)]);
        assert_eq!(
            parse_optional_sheet_name_to_id(
                "'Types: sequences, mapping, sets'!A1:B2",
                &sheet.id,
                &map
            ),
            Ok((sheet.id, "A1:B2"))
        );
    }
}