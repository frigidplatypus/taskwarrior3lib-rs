//! Date synonym definitions
//!
//! This module contains the enumeration of all supported date synonyms.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Comprehensive enumeration of date synonyms supported by Taskwarrior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DateSynonym {
    // Absolute time references
    Now,
    Today,
    Yesterday,
    Tomorrow,

    // Weekdays (full names)
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,

    // Weekday abbreviations
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,

    // Week references
    Weekdays,
    Weekend,

    // Month boundaries
    Som, // Start of month
    Eom, // End of month

    // Year boundaries
    Soy, // Start of year
    Eoy, // End of year

    // Quarter references
    Q1,
    Q2,
    Q3,
    Q4,

    // Special holidays (basic set)
    NewYear,
    Valentine,
    Easter,
    Independence,
    Halloween,
    Thanksgiving,
    Christmas,

    // Relative time units (for use in expressions)
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl DateSynonym {
    /// Get all available date synonyms
    pub fn all() -> Vec<DateSynonym> {
        vec![
            DateSynonym::Now,
            DateSynonym::Today,
            DateSynonym::Yesterday,
            DateSynonym::Tomorrow,
            DateSynonym::Monday,
            DateSynonym::Tuesday,
            DateSynonym::Wednesday,
            DateSynonym::Thursday,
            DateSynonym::Friday,
            DateSynonym::Saturday,
            DateSynonym::Sunday,
            DateSynonym::Mon,
            DateSynonym::Tue,
            DateSynonym::Wed,
            DateSynonym::Thu,
            DateSynonym::Fri,
            DateSynonym::Sat,
            DateSynonym::Sun,
            DateSynonym::Weekdays,
            DateSynonym::Weekend,
            DateSynonym::Som,
            DateSynonym::Eom,
            DateSynonym::Soy,
            DateSynonym::Eoy,
            DateSynonym::Q1,
            DateSynonym::Q2,
            DateSynonym::Q3,
            DateSynonym::Q4,
            DateSynonym::NewYear,
            DateSynonym::Valentine,
            DateSynonym::Easter,
            DateSynonym::Independence,
            DateSynonym::Halloween,
            DateSynonym::Thanksgiving,
            DateSynonym::Christmas,
            DateSynonym::Second,
            DateSynonym::Minute,
            DateSynonym::Hour,
            DateSynonym::Day,
            DateSynonym::Week,
            DateSynonym::Month,
            DateSynonym::Quarter,
            DateSynonym::Year,
        ]
    }

    /// Get synonyms that represent absolute dates
    pub fn absolute_synonyms() -> Vec<DateSynonym> {
        vec![
            DateSynonym::Now,
            DateSynonym::Today,
            DateSynonym::Yesterday,
            DateSynonym::Tomorrow,
            DateSynonym::Monday,
            DateSynonym::Tuesday,
            DateSynonym::Wednesday,
            DateSynonym::Thursday,
            DateSynonym::Friday,
            DateSynonym::Saturday,
            DateSynonym::Sunday,
            DateSynonym::Som,
            DateSynonym::Eom,
            DateSynonym::Soy,
            DateSynonym::Eoy,
            DateSynonym::Q1,
            DateSynonym::Q2,
            DateSynonym::Q3,
            DateSynonym::Q4,
        ]
    }

    /// Get synonyms that represent time units (for relative calculations)
    pub fn time_unit_synonyms() -> Vec<DateSynonym> {
        vec![
            DateSynonym::Second,
            DateSynonym::Minute,
            DateSynonym::Hour,
            DateSynonym::Day,
            DateSynonym::Week,
            DateSynonym::Month,
            DateSynonym::Quarter,
            DateSynonym::Year,
        ]
    }

    /// Get holiday synonyms
    pub fn holiday_synonyms() -> Vec<DateSynonym> {
        vec![
            DateSynonym::NewYear,
            DateSynonym::Valentine,
            DateSynonym::Easter,
            DateSynonym::Independence,
            DateSynonym::Halloween,
            DateSynonym::Thanksgiving,
            DateSynonym::Christmas,
        ]
    }

    // Parsing is provided via the std::str::FromStr trait implementation below.

    // Use Display implementation for canonical string representation.

    /// Check if this synonym represents a weekday
    pub fn is_weekday(&self) -> bool {
        matches!(
            self,
            DateSynonym::Monday
                | DateSynonym::Tuesday
                | DateSynonym::Wednesday
                | DateSynonym::Thursday
                | DateSynonym::Friday
                | DateSynonym::Saturday
                | DateSynonym::Sunday
                | DateSynonym::Mon
                | DateSynonym::Tue
                | DateSynonym::Wed
                | DateSynonym::Thu
                | DateSynonym::Fri
                | DateSynonym::Sat
                | DateSynonym::Sun
        )
    }

    /// Check if this synonym represents a time unit
    pub fn is_time_unit(&self) -> bool {
        matches!(
            self,
            DateSynonym::Second
                | DateSynonym::Minute
                | DateSynonym::Hour
                | DateSynonym::Day
                | DateSynonym::Week
                | DateSynonym::Month
                | DateSynonym::Quarter
                | DateSynonym::Year
        )
    }

    /// Check if this synonym represents a holiday
    pub fn is_holiday(&self) -> bool {
        matches!(
            self,
            DateSynonym::NewYear
                | DateSynonym::Valentine
                | DateSynonym::Easter
                | DateSynonym::Independence
                | DateSynonym::Halloween
                | DateSynonym::Thanksgiving
                | DateSynonym::Christmas
        )
    }
}

impl fmt::Display for DateSynonym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DateSynonym::Now => "now",
            DateSynonym::Today => "today",
            DateSynonym::Yesterday => "yesterday",
            DateSynonym::Tomorrow => "tomorrow",
            DateSynonym::Monday => "monday",
            DateSynonym::Tuesday => "tuesday",
            DateSynonym::Wednesday => "wednesday",
            DateSynonym::Thursday => "thursday",
            DateSynonym::Friday => "friday",
            DateSynonym::Saturday => "saturday",
            DateSynonym::Sunday => "sunday",
            DateSynonym::Mon => "mon",
            DateSynonym::Tue => "tue",
            DateSynonym::Wed => "wed",
            DateSynonym::Thu => "thu",
            DateSynonym::Fri => "fri",
            DateSynonym::Sat => "sat",
            DateSynonym::Sun => "sun",
            DateSynonym::Weekdays => "weekdays",
            DateSynonym::Weekend => "weekend",
            DateSynonym::Som => "som",
            DateSynonym::Eom => "eom",
            DateSynonym::Soy => "soy",
            DateSynonym::Eoy => "eoy",
            DateSynonym::Q1 => "q1",
            DateSynonym::Q2 => "q2",
            DateSynonym::Q3 => "q3",
            DateSynonym::Q4 => "q4",
            DateSynonym::NewYear => "newyear",
            DateSynonym::Valentine => "valentine",
            DateSynonym::Easter => "easter",
            DateSynonym::Independence => "independence",
            DateSynonym::Halloween => "halloween",
            DateSynonym::Thanksgiving => "thanksgiving",
            DateSynonym::Christmas => "christmas",
            DateSynonym::Second => "second",
            DateSynonym::Minute => "minute",
            DateSynonym::Hour => "hour",
            DateSynonym::Day => "day",
            DateSynonym::Week => "week",
            DateSynonym::Month => "month",
            DateSynonym::Quarter => "quarter",
            DateSynonym::Year => "year",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for DateSynonym {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            "now" => Ok(DateSynonym::Now),
            "today" => Ok(DateSynonym::Today),
            "yesterday" => Ok(DateSynonym::Yesterday),
            "tomorrow" => Ok(DateSynonym::Tomorrow),
            "monday" => Ok(DateSynonym::Monday),
            "tuesday" => Ok(DateSynonym::Tuesday),
            "wednesday" => Ok(DateSynonym::Wednesday),
            "thursday" => Ok(DateSynonym::Thursday),
            "friday" => Ok(DateSynonym::Friday),
            "saturday" => Ok(DateSynonym::Saturday),
            "sunday" => Ok(DateSynonym::Sunday),
            "mon" => Ok(DateSynonym::Mon),
            "tue" => Ok(DateSynonym::Tue),
            "wed" => Ok(DateSynonym::Wed),
            "thu" => Ok(DateSynonym::Thu),
            "fri" => Ok(DateSynonym::Fri),
            "sat" => Ok(DateSynonym::Sat),
            "sun" => Ok(DateSynonym::Sun),
            "weekdays" => Ok(DateSynonym::Weekdays),
            "weekend" => Ok(DateSynonym::Weekend),
            "som" => Ok(DateSynonym::Som),
            "eom" => Ok(DateSynonym::Eom),
            "soy" => Ok(DateSynonym::Soy),
            "eoy" => Ok(DateSynonym::Eoy),
            "q1" => Ok(DateSynonym::Q1),
            "q2" => Ok(DateSynonym::Q2),
            "q3" => Ok(DateSynonym::Q3),
            "q4" => Ok(DateSynonym::Q4),
            "newyear" | "new_year" => Ok(DateSynonym::NewYear),
            "valentine" | "valentines" => Ok(DateSynonym::Valentine),
            "easter" => Ok(DateSynonym::Easter),
            "independence" | "july4" | "july_4" => Ok(DateSynonym::Independence),
            "halloween" => Ok(DateSynonym::Halloween),
            "thanksgiving" => Ok(DateSynonym::Thanksgiving),
            "christmas" | "xmas" => Ok(DateSynonym::Christmas),
            "second" | "seconds" | "sec" => Ok(DateSynonym::Second),
            "minute" | "minutes" | "min" => Ok(DateSynonym::Minute),
            "hour" | "hours" | "hr" => Ok(DateSynonym::Hour),
            "day" | "days" | "d" => Ok(DateSynonym::Day),
            "week" | "weeks" | "w" => Ok(DateSynonym::Week),
            "month" | "months" | "m" => Ok(DateSynonym::Month),
            "quarter" | "quarters" | "q" => Ok(DateSynonym::Quarter),
            "year" | "years" | "y" => Ok(DateSynonym::Year),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_from_str() {
        assert_eq!(DateSynonym::from_str("today"), Ok(DateSynonym::Today));
        assert_eq!(DateSynonym::from_str("MONDAY"), Ok(DateSynonym::Monday));
        assert_eq!(DateSynonym::from_str("eom"), Ok(DateSynonym::Eom));
        assert!(DateSynonym::from_str("invalid").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{today}", today = DateSynonym::Today), "today");
        assert_eq!(format!("{mon}", mon = DateSynonym::Monday), "monday");
        assert_eq!(format!("{eom}", eom = DateSynonym::Eom), "eom");
    }

    #[test]
    fn test_is_weekday() {
        assert!(DateSynonym::Monday.is_weekday());
        assert!(DateSynonym::Fri.is_weekday());
        assert!(!DateSynonym::Today.is_weekday());
    }

    #[test]
    fn test_is_time_unit() {
        assert!(DateSynonym::Day.is_time_unit());
        assert!(DateSynonym::Month.is_time_unit());
        assert!(!DateSynonym::Monday.is_time_unit());
    }

    #[test]
    fn test_all_synonyms() {
        let all = DateSynonym::all();
        assert!(!all.is_empty());
        assert!(all.contains(&DateSynonym::Today));
        assert!(all.contains(&DateSynonym::Eom));
    }
}
