use std::{fmt, ops::Add, str::FromStr};

use chrono::{Datelike, Month, NaiveDate};
use strum::EnumString;
use syntax::bibtex::Value;

use super::text::TextFieldData;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum DateField {
    Date,
    EventDate,
    Month,
    UrlDate,
    Year,
}

impl DateField {
    pub fn parse(input: &str) -> Option<Self> {
        Self::from_str(input).ok()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum DateFieldData {
    Date(NaiveDate),
    Year(i32),
    Month(Month),
    Other(String),
}

impl Add for DateFieldData {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (date @ Self::Year(_), Self::Year(_))
            | (date @ Self::Month(_), Self::Month(_))
            | (date @ Self::Date(_), Self::Date(_))
            | (Self::Other(_), date)
            | (date, Self::Other(_)) => date,
            (Self::Year(year), Self::Month(month)) | (Self::Month(month), Self::Year(year)) => {
                let new_date = NaiveDate::from_ymd_opt(year, month.number_from_month(), 1).unwrap();
                Self::Date(new_date)
            }
            (Self::Year(year), Self::Date(date)) | (Self::Date(date), Self::Year(year)) => {
                let new_date = date.with_year(year).unwrap_or(date);
                Self::Date(new_date)
            }
            (Self::Date(date), Self::Month(month)) | (Self::Month(month), Self::Date(date)) => {
                let new_date = date.with_month(month.number_from_month()).unwrap_or(date);
                Self::Date(new_date)
            }
        }
    }
}

impl fmt::Display for DateFieldData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Year(year) => write!(f, "{}", year),
            Self::Date(date) => write!(f, "{}", date.format("%b. %Y")),
            Self::Month(month) => write!(f, "{}", month.name()),
            Self::Other(text) => write!(f, "{}", text),
        }
    }
}

impl DateFieldData {
    pub fn parse(value: &Value) -> Option<Self> {
        let TextFieldData { text } = TextFieldData::parse(value)?;
        NaiveDate::from_str(&text)
            .ok()
            .map(Self::Date)
            .or_else(|| text.parse().ok().map(Self::Year))
            .or_else(|| text.parse().ok().map(Self::Month))
            .or(Some(Self::Other(text)))
    }
}
