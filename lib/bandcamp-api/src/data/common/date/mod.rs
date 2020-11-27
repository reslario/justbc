#[cfg(feature = "query")]
pub mod parse;

use std::{
    fmt,
    num::NonZeroU8
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub day: NonZeroU8,
    pub month: Month,
    pub year: u16
}

impl Date {
    pub fn fmt_short(self) -> impl fmt::Display {
        FmtDate {
            date: self,
            month_str: Month::short
        }
    }

    pub fn fmt_long(self) -> impl fmt::Display {
        FmtDate {
            date: self,
            month_str: Month::long
        }
    }
}

struct FmtDate {
    date: Date,
    month_str: fn(Month) -> &'static str
}

impl fmt::Display for FmtDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, "{} {} {}",
            self.date.day,
            (self.month_str)(self.date.month),
            self.date.year
        )
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Month {
    January = 1,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December
}

impl Month {
    const SHORT_LEN: usize = 3;

    #[cfg(feature = "query")]
    const ALL: [Month; 12] = [
        Month::January,
        Month::February,
        Month::March,
        Month::April,
        Month::May,
        Month::June,
        Month::July,
        Month::August,
        Month::September,
        Month::October,
        Month::November,
        Month::December
    ];

    #[cfg(feature = "query")]
    fn iter() -> impl Iterator<Item = Month> {
        Month::ALL.iter().cloned()
    }

    pub fn long(self) -> &'static str {
        macro_rules! long {
            ( $($month:ident),+ ) => {
                match self {
                    $(
                        $month => stringify!($month)
                    ),+
                }
            }
        }

        use Month::*;

        long!(
            January,
            February,
            March,
            April,
            May,
            June,
            July,
            August,
            September,
            October,
            November,
            December
        )
    }

    pub fn short(self) -> &'static str {
        &self.long()[..Month::SHORT_LEN]
    }

    #[cfg(feature = "query")]
    fn matches_str(self, s: &str) -> bool {
        if let Some(rest) = s.strip_prefix(self.short()) {
            rest.is_empty() || rest == &self.long()[Month::SHORT_LEN..]
        } else {
            false
        }
    }
}

impl fmt::Debug for Month {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.long(), f)
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
