use chrono::{DateTime, Utc};

pub mod consumption;
pub mod generation;

pub trait FormatToApiFmt {
    fn to_api_format(&self) -> String;
}

impl FormatToApiFmt for DateTime<Utc> {
    fn to_api_format(&self) -> String {
        // Define the desired format for your API
        // You can adjust this format string to match the API's expected format
        self.format("%Y-%m-%dT%H:%M:%S+00:00").to_string()
    }
}

#[derive(Debug)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl DateRange {
    fn to_query_string(&self) -> Vec<(String, String)> {
        vec![
            ("start_date".to_string(), self.start.to_api_format()),
            ("end_date".to_string(), self.end.to_api_format()),
        ]
    }
}
