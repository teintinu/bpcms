//! Pending comments: TODO, FIXME, REVIEW, OPTIMIZE and HACK.
//!
//! Syntax: `// KEYWORD [YYYY-MM-DD] message`. The keyword is case-insensitive
//! in source and canonicalized to uppercase in the AST and by the formatter.
//! The date is the **due date**: missing or past dates are errors, and due
//! dates within 15 days are warnings.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::ast::Pending;
use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::token::{Token, TokenKind};

pub const KEYWORDS: [&str; 5] = ["TODO", "FIXME", "REVIEW", "OPTIMIZE", "HACK"];
const SOON_DAYS: i64 = 15;

/// A calendar date used by pending-date checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        if !(1..=12).contains(&month) || day == 0 {
            return None;
        }
        let days = days_in_month(year, month);
        (day <= days).then_some(Self { year, month, day })
    }

    pub fn parse(text: &str) -> Option<Self> {
        if text.len() != 10 {
            return None;
        }
        let bytes = text.as_bytes();
        if bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        if !bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
        {
            return None;
        }
        let year = text[0..4].parse().ok()?;
        let month = text[5..7].parse().ok()?;
        let day = text[8..10].parse().ok()?;
        Self::new(year, month, day)
    }

    fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

pub fn today() -> Date {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| (duration.as_secs() / 86_400) as i64)
        .unwrap_or(0);
    civil_from_days(days)
}

pub fn days_between(from: Date, to: Date) -> i64 {
    days_from_civil(to) - days_from_civil(from)
}

/// Typed node kind for one pending keyword.
pub fn pending_kind(keyword: &str) -> &'static str {
    match keyword {
        "FIXME" => "PendingFixme",
        "REVIEW" => "PendingReview",
        "OPTIMIZE" => "PendingOptimize",
        "HACK" => "PendingHack",
        _ => "PendingTodo",
    }
}

/// Collects pending comments from the lexer tokens (comments included).
pub fn collect(source: &str, tokens: &[Token], path: &str) -> Vec<Pending> {
    let mut pending = Vec::new();
    for token in tokens
        .iter()
        .filter(|token| token.kind == TokenKind::Comment)
    {
        let Some(text) = source.get(token.span.start..token.span.end) else {
            continue;
        };
        let body = text.strip_prefix("//").unwrap_or(text);
        let leading = body.len() - body.trim_start().len();
        let body = body.trim_start();
        if body.is_empty() {
            continue;
        }
        let keyword_end = body.find(char::is_whitespace).unwrap_or(body.len());
        let keyword = &body[..keyword_end];
        let Some(canonical) = KEYWORDS
            .iter()
            .find(|candidate| candidate.eq_ignore_ascii_case(keyword))
            .copied()
        else {
            continue;
        };
        let keyword_start = token.span.start + 2 + leading;
        let keyword_span = Span::new(keyword_start, keyword_start + keyword.len());
        let mut rest = body[keyword_end..].trim_start();

        let mut date_text = None;
        let mut due_date = None;
        if let Some(after) = rest.strip_prefix('[') {
            if let Some(end) = after.find(']') {
                let raw = after[..end].trim().to_string();
                date_text = Some(raw.clone());
                due_date = Date::parse(&raw).map(|date| date.iso());
                rest = after[end + 1..].trim_start();
            }
        }

        pending.push(Pending {
            kind: pending_kind(canonical),
            id: format!("pending:{path}:{}", token.span.start),
            due_date,
            date_text,
            message: rest.to_string(),
            span: token.span,
            keyword_span,
        });
    }
    pending
}

/// Validates due dates. Missing dates and past dates are errors; dates within
/// 15 days are warnings.
pub fn validate(pending: &Pending, today: Date) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let Some(due) = pending.due_date.as_deref().and_then(Date::parse) else {
        if pending.date_text.is_some() {
            diagnostics.push(Diagnostic::error(
                "E_PENDING_DATE",
                format!(
                    "{} has an invalid due date `{}` (expected YYYY-MM-DD)",
                    pending.keyword(),
                    pending.date_text.as_deref().unwrap_or("")
                ),
                pending.span,
            ));
        } else {
            diagnostics.push(Diagnostic::error(
                "E_PENDING_NO_DATE",
                format!("{} has no due date; add [YYYY-MM-DD]", pending.keyword()),
                pending.span,
            ));
        }
        return diagnostics;
    };

    let remaining = days_between(today, due);
    if remaining < 0 {
        diagnostics.push(Diagnostic::error(
            "E_PENDING_EXPIRED",
            format!(
                "{} expired on {} ({} day(s) ago)",
                pending.keyword(),
                due.iso(),
                -remaining
            ),
            pending.span,
        ));
    } else if remaining <= SOON_DAYS {
        diagnostics.push(Diagnostic::warning(
            "W_PENDING_SOON",
            format!(
                "{} is due in {} day(s) on {}",
                pending.keyword(),
                remaining,
                due.iso()
            ),
            pending.span,
        ));
    }
    diagnostics
}

/// Format edits that uppercase pending keywords (comments only).
pub fn format_edits(source: &str, pending: &[Pending]) -> Vec<(Span, String)> {
    pending
        .iter()
        .filter_map(|pending| {
            let current = source.get(pending.keyword_span.start..pending.keyword_span.end)?;
            (current != pending.keyword())
                .then(|| (pending.keyword_span, pending.keyword().to_string()))
        })
        .collect()
}

/// Prefer double quotes when the body has no quote/escape to preserve.
pub fn quote_edits(source: &str) -> Vec<(Span, String)> {
    crate::lexer::lex(source)
        .tokens
        .into_iter()
        .filter(|token| token.kind == TokenKind::String)
        .filter_map(|token| {
            let raw = source.get(token.span.start..token.span.end)?;
            let body = raw.strip_prefix('\'')?.strip_suffix('\'')?;
            if body.contains('"') || body.contains('\\') {
                return None;
            }
            Some((token.span, format!("\"{body}\"")))
        })
        .collect()
}

/// Applies pending and string-quote formatter edits to `source`.
pub fn format_source(source: &str, pending: &[Pending]) -> String {
    let mut edits = format_edits(source, pending);
    edits.extend(quote_edits(source));
    edits.sort_by_key(|(span, _)| span.start);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (span, replacement) in edits {
        if span.start < cursor {
            continue;
        }
        output.push_str(&source[cursor..span.start]);
        output.push_str(&replacement);
        cursor = span.end;
    }
    output.push_str(&source[cursor..]);
    output
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

// Howard Hinnant's civil-date algorithms (proleptic Gregorian, UTC).
fn days_from_civil(date: Date) -> i64 {
    let mut year = date.year as i64;
    let month = date.month as i64;
    let day = date.day as i64;
    year -= i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> Date {
    let zero_day = days + 719_468;
    let era = if zero_day >= 0 {
        zero_day
    } else {
        zero_day - 146_096
    } / 146_097;
    let day_of_era = zero_day - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_piece = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_piece + 2) / 5 + 1;
    let month = month_piece + if month_piece < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    Date {
        year: year as i32,
        month: month as u32,
        day: day as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    fn parse_pending(source: &str) -> Vec<Pending> {
        let lexed = lex(source);
        collect(source, &lexed.tokens, "test.archol")
    }

    #[test]
    fn recognizes_keywords_case_insensitively_and_keeps_message() {
        let items =
            parse_pending("// todo implementar criptografia\n// FixMe [2027-03-12] corrigir\n");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].kind, "PendingTodo");
        assert_eq!(items[0].keyword(), "TODO");
        assert_eq!(items[0].due_date, None);
        assert_eq!(items[0].message, "implementar criptografia");
        assert_eq!(items[1].kind, "PendingFixme");
        assert_eq!(items[1].keyword(), "FIXME");
        assert_eq!(items[1].due_date.as_deref(), Some("2027-03-12"));
        assert_eq!(items[1].message, "corrigir");
    }

    #[test]
    fn ignores_comments_without_a_pending_keyword() {
        assert!(parse_pending("// just a comment\n// NOTA keyword is ignored\n").is_empty());
    }

    #[test]
    fn missing_and_past_dates_are_errors() {
        let today = Date::new(2026, 11, 18).unwrap();
        let items = parse_pending("// TODO no date\n// FIXME [2026-11-17] late\n");
        assert_eq!(validate(&items[0], today)[0].code, "E_PENDING_NO_DATE");
        assert_eq!(validate(&items[1], today)[0].code, "E_PENDING_EXPIRED");
    }

    #[test]
    fn due_within_fifteen_days_is_a_warning_and_later_is_clean() {
        let today = Date::new(2026, 11, 18).unwrap();
        let soon = parse_pending("// REVIEW [2026-12-03] within 15 days\n");
        assert_eq!(validate(&soon[0], today)[0].code, "W_PENDING_SOON");
        let later = parse_pending("// REVIEW [2026-12-04] after 15 days\n");
        assert!(validate(&later[0], today).is_empty());
    }

    #[test]
    fn invalid_dates_are_reported() {
        let today = Date::new(2026, 11, 18).unwrap();
        let items = parse_pending("// HACK [2026-02-30] impossible\n");
        assert_eq!(items[0].due_date, None);
        assert_eq!(items[0].date_text.as_deref(), Some("2026-02-30"));
        assert_eq!(validate(&items[0], today)[0].code, "E_PENDING_DATE");
    }

    #[test]
    fn formatter_uppercases_only_pending_keywords() {
        let source = "// todo lower\n// note stays\n";
        let items = parse_pending(source);
        assert_eq!(
            format_source(source, &items),
            "// TODO lower\n// note stays\n"
        );
    }

    #[test]
    fn formatter_prefers_double_quotes_when_possible() {
        let source = "document A {\n  title: 'A'\n  note: 'has \"double\" quotes'\n}\n";
        let formatted = format_source(source, &[]);
        assert!(formatted.contains("title: \"A\""), "{formatted}");
        assert!(
            formatted.contains("note: 'has \"double\" quotes'"),
            "{formatted}"
        );
    }
}
