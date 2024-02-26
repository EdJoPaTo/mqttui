use chrono::NaiveDateTime;

#[derive(Debug, Clone, Copy)]
pub enum Time {
    Retained,
    Local(NaiveDateTime),
}

impl Time {
    pub fn new_now(retain: bool) -> Self {
        if retain {
            Self::Retained
        } else {
            Self::Local(chrono::Local::now().naive_local())
        }
    }

    pub const fn as_optional(&self) -> Option<&NaiveDateTime> {
        if let Self::Local(time) = self {
            Some(time)
        } else {
            None
        }
    }

    #[cfg(test)]
    pub fn datetime_example() -> NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(1996, 12, 19)
            .unwrap()
            .and_hms_opt(16, 39, 57)
            .unwrap()
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Retained => fmt.pad("RETAINED"),
            Self::Local(time) => fmt.write_fmt(format_args!("{}", time.format("%_H:%M:%S.%3f"))),
        }
    }
}

#[test]
fn new_now_retained() {
    let result = Time::new_now(true);
    dbg!(result);
    assert!(matches!(result, Time::Retained));
}

#[test]
fn optional_retained() {
    let time = Time::Retained;
    assert_eq!(time.as_optional(), None);
}

#[test]
fn optional_time() {
    let date = Time::datetime_example();
    let time = Time::Local(date);
    assert_eq!(time.as_optional(), Some(&date));
}

#[test]
fn local_to_string() {
    let date = Time::datetime_example();
    let time = Time::Local(date);
    assert_eq!(time.to_string(), "16:39:57.000");
}

#[test]
fn retained_to_string() {
    let time = Time::Retained;
    assert_eq!(time.to_string(), "RETAINED");
}

#[test]
fn retained_fmt_width() {
    let time = Time::Retained;
    let time = format!("{time:12}");
    assert_eq!(time, "RETAINED    ");
}
