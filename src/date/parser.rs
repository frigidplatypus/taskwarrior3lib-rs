//! Date parser implementation
//!
//! This module provides comprehensive date parsing functionality including
//! ISO-8601 formats, named synonyms, and relative date calculations.

use chrono::{DateTime, Utc, TimeZone, Datelike, Weekday, NaiveDate};
use chrono_tz::Tz;
use crate::error::DateError;
use crate::date::DateParsing;

/// Main date parser implementation
#[derive(Debug, Clone)]
pub struct DateParser {
    /// Timezone for parsing (defaults to UTC)
    timezone: Tz,
    /// Custom date format patterns
    custom_formats: Vec<String>,
}

impl Default for DateParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DateParser {
    /// Create a new date parser with default settings
    pub fn new() -> Self {
        Self {
            timezone: Tz::UTC,
            custom_formats: vec![
                "%Y-%m-%d".to_string(),           // ISO date
                "%Y-%m-%dT%H:%M:%S".to_string(),  // ISO datetime
                "%Y-%m-%dT%H:%M:%SZ".to_string(), // ISO datetime with Z
                "%m/%d/%Y".to_string(),           // US format
                "%d/%m/%Y".to_string(),           // European format
                "%Y/%m/%d".to_string(),           // Alternative ISO
            ],
        }
    }
    
    /// Create a parser with specific timezone
    pub fn with_timezone(timezone: Tz) -> Self {
        Self {
            timezone,
            custom_formats: Self::new().custom_formats,
        }
    }
    
    /// Add a custom date format
    pub fn add_format(&mut self, format: String) {
        self.custom_formats.push(format);
    }
}

impl DateParsing for DateParser {
    fn parse_date(&self, input: &str) -> Result<DateTime<Utc>, DateError> {
        let input = input.trim();
        
        // Try parsing as synonym first
        if let Ok(date) = self.parse_synonym(input) {
            return Ok(date);
        }
        
        // Try each custom format
        for format in &self.custom_formats {
            if let Ok(date) = self.parse_with_format(input, format) {
                return Ok(date);
            }
        }
        
        // Try parsing as relative date
        if input.contains("+") || input.contains("-") {
            return self.calculate_relative_date(Utc::now(), input);
        }
        
        Err(DateError::InvalidFormat {
            input: input.to_string(),
        })
    }
    
    fn parse_synonym(&self, synonym: &str) -> Result<DateTime<Utc>, DateError> {
        let synonym_lower = synonym.to_lowercase();
        let now = Utc::now();
        
        let date = match synonym_lower.as_str() {
            "now" => now,
            "today" => {
                let date = now.date_naive();
                self.timezone.from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap()).single()
                    .ok_or_else(|| DateError::Timezone { 
                        message: "Ambiguous local date".to_string() 
                    })?
                    .with_timezone(&Utc)
            },
            "yesterday" => {
                let date = (now - chrono::Duration::days(1)).date_naive();
                self.timezone.from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap()).single()
                    .ok_or_else(|| DateError::Timezone { 
                        message: "Ambiguous local date".to_string() 
                    })?
                    .with_timezone(&Utc)
            },
            "tomorrow" => {
                let date = (now + chrono::Duration::days(1)).date_naive();
                self.timezone.from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap()).single()
                    .ok_or_else(|| DateError::Timezone { 
                        message: "Ambiguous local date".to_string() 
                    })?
                    .with_timezone(&Utc)
            },
            // Weekdays
            "monday" | "tue" | "tuesday" | "wed" | "wednesday" | 
            "thu" | "thursday" | "fri" | "friday" | "sat" | "saturday" | "sun" | "sunday" => {
                self.next_weekday(&synonym_lower)?
            },
            // Month boundaries
            "som" => self.start_of_month(now)?,
            "eom" => self.end_of_month(now)?,
            // Year boundaries
            "soy" => self.start_of_year(now)?,
            "eoy" => self.end_of_year(now)?,
            // Quarters
            "q1" => self.start_of_quarter(now, 1)?,
            "q2" => self.start_of_quarter(now, 2)?,
            "q3" => self.start_of_quarter(now, 3)?,
            "q4" => self.start_of_quarter(now, 4)?,
            _ => return Err(DateError::UnknownSynonym {
                synonym: synonym.to_string(),
            }),
        };
        
        Ok(date)
    }
    
    fn parse_date_with_format(&self, input: &str, format: &str) -> Result<DateTime<Utc>, DateError> {
        self.parse_with_format(input, format)
    }
    
    fn calculate_relative_date(&self, base: DateTime<Utc>, expression: &str) -> Result<DateTime<Utc>, DateError> {
        let expression = expression.trim();
        
        // Parse expressions like "+1week", "-3days", "now+2months"
        let (base_date, offset_str) = if let Some(stripped) = expression.strip_prefix("now") {
            (Utc::now(), stripped)
        } else {
            (base, expression)
        };
        
        if offset_str.is_empty() {
            return Ok(base_date);
        }
        
        let (sign, rest) = if let Some(stripped) = offset_str.strip_prefix('+') {
            (1, stripped)
        } else if let Some(stripped) = offset_str.strip_prefix('-') {
            (-1, stripped)
        } else {
            (1, offset_str)
        };
        
        // Extract number and unit
        let (num_str, unit) = self.split_number_unit(rest)?;
        let number: i64 = num_str.parse().map_err(|_| DateError::InvalidRelative {
            expression: expression.to_string(),
        })?;
        
        let signed_number = sign * number;
        
        let result = match unit {
            "d" | "day" | "days" => base_date + chrono::Duration::days(signed_number),
            "w" | "week" | "weeks" => base_date + chrono::Duration::weeks(signed_number),
            "m" | "month" | "months" => {
                // Add months using date arithmetic
                let mut date = base_date.date_naive();
                if signed_number > 0 {
                    for _ in 0..signed_number {
                        date = self.add_month(date);
                    }
                } else {
                    for _ in 0..(-signed_number) {
                        date = self.subtract_month(date);
                    }
                }
                self.timezone.from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap()).single()
                    .ok_or_else(|| DateError::Timezone {
                        message: "Invalid date after month calculation".to_string()
                    })?
                    .with_timezone(&Utc)
            },
            "y" | "year" | "years" => {
                let mut date = base_date.date_naive();
                let new_year = date.year() + signed_number as i32;
                date = date.with_year(new_year).ok_or_else(|| DateError::InvalidRelative {
                    expression: expression.to_string(),
                })?;
                self.timezone.from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap()).single()
                    .ok_or_else(|| DateError::Timezone {
                        message: "Invalid date after year calculation".to_string()
                    })?
                    .with_timezone(&Utc)
            },
            _ => return Err(DateError::InvalidRelative {
                expression: expression.to_string(),
            }),
        };
        
        Ok(result)
    }
    
    fn format_date(&self, date: DateTime<Utc>) -> String {
        date.format("%Y-%m-%d").to_string()
    }
    
    fn get_supported_synonyms(&self) -> Vec<String> {
        vec![
            "now".to_string(), "today".to_string(), "yesterday".to_string(), "tomorrow".to_string(),
            "monday".to_string(), "tuesday".to_string(), "wednesday".to_string(), "thursday".to_string(),
            "friday".to_string(), "saturday".to_string(), "sunday".to_string(),
            "som".to_string(), "eom".to_string(), "soy".to_string(), "eoy".to_string(),
            "q1".to_string(), "q2".to_string(), "q3".to_string(), "q4".to_string(),
        ]
    }
}

// Private helper methods
impl DateParser {
    fn parse_with_format(&self, input: &str, format: &str) -> Result<DateTime<Utc>, DateError> {
        // Try parsing with timezone awareness
        if let Ok(datetime) = DateTime::parse_from_str(input, format) {
            return Ok(datetime.with_timezone(&Utc));
        }
        
        // Try parsing as naive datetime and assume UTC
        if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(input, format) {
            return Ok(Utc.from_utc_datetime(&naive_dt));
        }
        
        // Try parsing as date only
        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(input, format) {
            let naive_dt = naive_date.and_hms_opt(0, 0, 0)
                .ok_or_else(|| DateError::InvalidFormat {
                    input: input.to_string(),
                })?;
            return Ok(Utc.from_utc_datetime(&naive_dt));
        }
        
        Err(DateError::InvalidFormat {
            input: input.to_string(),
        })
    }
    
    fn next_weekday(&self, weekday: &str) -> Result<DateTime<Utc>, DateError> {
        let target_weekday = match weekday {
            "monday" => Weekday::Mon,
            "tue" | "tuesday" => Weekday::Tue,
            "wed" | "wednesday" => Weekday::Wed,
            "thu" | "thursday" => Weekday::Thu,
            "fri" | "friday" => Weekday::Fri,
            "sat" | "saturday" => Weekday::Sat,
            "sun" | "sunday" => Weekday::Sun,
            _ => return Err(DateError::UnknownSynonym {
                synonym: weekday.to_string(),
            }),
        };
        
        let today = Utc::now().date_naive();
        let current_weekday = today.weekday();
        let target_days = target_weekday.num_days_from_monday() as i32;
        let current_days = current_weekday.num_days_from_monday() as i32;
        let days_ahead = ((target_days - current_days + 7) % 7) as u32;
        
        let target_date = if days_ahead == 0 {
            today + chrono::Duration::days(7) // Next week if today is the target day
        } else {
            today + chrono::Duration::days(days_ahead as i64)
        };
        
        Ok(self.timezone.from_local_datetime(&target_date.and_hms_opt(0, 0, 0).unwrap()).single()
            .ok_or_else(|| DateError::Timezone {
                message: "Ambiguous weekday calculation".to_string()
            })?
            .with_timezone(&Utc))
    }
    
    fn start_of_month(&self, date: DateTime<Utc>) -> Result<DateTime<Utc>, DateError> {
        let first_day = NaiveDate::from_ymd_opt(date.year(), date.month(), 1)
            .ok_or_else(|| DateError::InvalidFormat {
                input: "start of month".to_string(),
            })?;
        
        Ok(self.timezone.from_local_datetime(&first_day.and_hms_opt(0, 0, 0).unwrap()).single()
            .ok_or_else(|| DateError::Timezone {
                message: "Ambiguous start of month".to_string()
            })?
            .with_timezone(&Utc))
    }
    
    fn end_of_month(&self, date: DateTime<Utc>) -> Result<DateTime<Utc>, DateError> {
        let next_month = if date.month() == 12 {
            NaiveDate::from_ymd_opt(date.year() + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)
        };
        
        let next_month = next_month.ok_or_else(|| DateError::InvalidFormat {
            input: "end of month".to_string(),
        })?;
        
        let last_day = next_month - chrono::Duration::days(1);
        
        Ok(self.timezone.from_local_datetime(&last_day.and_hms_opt(23, 59, 59).unwrap()).single()
            .ok_or_else(|| DateError::Timezone {
                message: "Ambiguous end of month".to_string()
            })?
            .with_timezone(&Utc))
    }
    
    fn start_of_year(&self, date: DateTime<Utc>) -> Result<DateTime<Utc>, DateError> {
        let first_day = NaiveDate::from_ymd_opt(date.year(), 1, 1)
            .ok_or_else(|| DateError::InvalidFormat {
                input: "start of year".to_string(),
            })?;
        
        Ok(self.timezone.from_local_datetime(&first_day.and_hms_opt(0, 0, 0).unwrap()).single()
            .ok_or_else(|| DateError::Timezone {
                message: "Ambiguous start of year".to_string()
            })?
            .with_timezone(&Utc))
    }
    
    fn end_of_year(&self, date: DateTime<Utc>) -> Result<DateTime<Utc>, DateError> {
        let last_day = NaiveDate::from_ymd_opt(date.year(), 12, 31)
            .ok_or_else(|| DateError::InvalidFormat {
                input: "end of year".to_string(),
            })?;
        
        Ok(self.timezone.from_local_datetime(&last_day.and_hms_opt(23, 59, 59).unwrap()).single()
            .ok_or_else(|| DateError::Timezone {
                message: "Ambiguous end of year".to_string()
            })?
            .with_timezone(&Utc))
    }
    
    fn start_of_quarter(&self, date: DateTime<Utc>, quarter: u32) -> Result<DateTime<Utc>, DateError> {
        let month = match quarter {
            1 => 1,
            2 => 4,
            3 => 7,
            4 => 10,
            _ => return Err(DateError::InvalidFormat {
                input: format!("quarter {quarter}"),
            }),
        };
        
        let first_day = NaiveDate::from_ymd_opt(date.year(), month, 1)
            .ok_or_else(|| DateError::InvalidFormat {
                input: format!("start of quarter {quarter}"),
            })?;
        
        Ok(self.timezone.from_local_datetime(&first_day.and_hms_opt(0, 0, 0).unwrap()).single()
            .ok_or_else(|| DateError::Timezone {
                message: "Ambiguous quarter start".to_string()
            })?
            .with_timezone(&Utc))
    }
    
    fn split_number_unit<'a>(&self, input: &'a str) -> Result<(&'a str, &'a str), DateError> {
        let mut split_pos = 0;
        
        for (i, c) in input.char_indices() {
            if !c.is_ascii_digit() {
                split_pos = i;
                break;
            }
        }
        
        if split_pos == 0 {
            return Err(DateError::InvalidRelative {
                expression: input.to_string(),
            });
        }
        
        Ok((&input[..split_pos], &input[split_pos..]))
    }
    
    fn add_month(&self, date: NaiveDate) -> NaiveDate {
        if date.month() == 12 {
            NaiveDate::from_ymd_opt(date.year() + 1, 1, date.day())
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(date.year() + 1, 1, 28).unwrap())
        } else {
            NaiveDate::from_ymd_opt(date.year(), date.month() + 1, date.day())
                .unwrap_or_else(|| {
                    // Handle month-end edge cases
                    let mut day = date.day();
                    while day > 28 {
                        if let Some(new_date) = NaiveDate::from_ymd_opt(date.year(), date.month() + 1, day) {
                            return new_date;
                        }
                        day -= 1;
                    }
                    NaiveDate::from_ymd_opt(date.year(), date.month() + 1, day).unwrap()
                })
        }
    }
    
    fn subtract_month(&self, date: NaiveDate) -> NaiveDate {
        if date.month() == 1 {
            NaiveDate::from_ymd_opt(date.year() - 1, 12, date.day())
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(date.year() - 1, 12, 31).unwrap())
        } else {
            NaiveDate::from_ymd_opt(date.year(), date.month() - 1, date.day())
                .unwrap_or_else(|| {
                    // Handle month-end edge cases
                    let mut day = date.day();
                    while day > 28 {
                        if let Some(new_date) = NaiveDate::from_ymd_opt(date.year(), date.month() - 1, day) {
                            return new_date;
                        }
                        day -= 1;
                    }
                    NaiveDate::from_ymd_opt(date.year(), date.month() - 1, day).unwrap()
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_iso_date() {
        let parser = DateParser::new();
        let date = parser.parse_date("2025-09-18").unwrap();
        assert_eq!(date.year(), 2025);
        assert_eq!(date.month(), 9);
        assert_eq!(date.day(), 18);
    }

    #[test]
    fn test_parse_synonyms() {
        let parser = DateParser::new();
        
        // These should not panic
        let _today = parser.parse_synonym("today").unwrap();
        let _now = parser.parse_synonym("now").unwrap();
        let _monday = parser.parse_synonym("monday").unwrap();
    }

    #[test]
    fn test_relative_dates() {
        let parser = DateParser::new();
        let base = Utc::now();
        
        let future = parser.calculate_relative_date(base, "+1week").unwrap();
        assert!(future > base);
        
        let past = parser.calculate_relative_date(base, "-3days").unwrap();
        assert!(past < base);
    }

    #[test]
    fn test_supported_synonyms() {
        let parser = DateParser::new();
        let synonyms = parser.get_supported_synonyms();
        
        assert!(!synonyms.is_empty());
        assert!(synonyms.contains(&"today".to_string()));
        assert!(synonyms.contains(&"monday".to_string()));
    }
}
