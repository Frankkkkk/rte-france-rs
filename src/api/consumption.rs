use std::fmt;

use crate::ApiClient;
use anyhow::Ok;
use polars::prelude::*;
use serde::Deserialize;
use serde_json;

use chrono::{DateTime, NaiveDateTime, Utc};

use super::DateRange;

pub struct ConsumptionForecast<'a> {
    client: &'a dyn ApiClient,
}

/// The type of forecast to retrieve
pub enum ShortTermForecastType {
    /// Realised consumption, not a forecast then
    Realised,

    /// Intra-day forecast
    Intraday,

    /// Next day forecast
    Tomorrow,

    /// Day after tomorrow forecast
    DayAfterTomorrow,
}

impl fmt::Display for ShortTermForecastType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ft = match self {
            ShortTermForecastType::Realised => "REALISED",
            ShortTermForecastType::Intraday => "ID",
            ShortTermForecastType::Tomorrow => "D-1",
            ShortTermForecastType::DayAfterTomorrow => "D-2",
        };
        write!(f, "{}", ft)
    }
}

#[derive(Deserialize, Debug)]
pub struct ShortTermResponse {
    pub short_term: Vec<ShortTerm>,
}

#[derive(Deserialize, Debug)]
pub struct ShortTerm {
    #[serde(rename = "type")]
    pub ty: String,

    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub values: Vec<ShortTermValue>,
}

#[derive(Deserialize, Debug)]
pub struct ShortTermValue {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub value: f64,
}

#[derive(Deserialize, Debug)]
pub struct WeeklyForecastResponse {
    pub weekly_forecasts: Vec<WeeklyForecast>,
}

#[derive(Deserialize, Debug)]
pub struct WeeklyForecast {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub peak: PeakForecast,
    pub values: Vec<WeeklyForecastValue>,
}

#[derive(Deserialize, Debug)]
pub struct PeakForecast {
    pub peak_hour: DateTime<Utc>,
    pub value: f64,
    pub temperature: f64,
    pub temperature_deviation: f64,
}

// XXX maybe we could share it with ShortTermValue (except for update)
#[derive(Deserialize, Debug)]
pub struct WeeklyForecastValue {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub value: f64,
}

impl<'a> ConsumptionForecast<'a> {
    const SHORT_TERM_URL: &'static str = "/open_api/consumption/v1/short_term";
    const WEEKLY_URL: &'static str = "/open_api/consumption/v1/weekly_forecasts";

    pub fn new(client: &'a dyn ApiClient) -> Self {
        Self { client }
    }

    /// Returns a short term forecast given the forecast type
    pub fn short_term(
        &self,
        forecast_type: ShortTermForecastType,
        date_range: Option<DateRange>,
    ) -> Result<ShortTermResponse, anyhow::Error> {
        let mut qs: Vec<(String, String)> = vec![];

        qs.push(("type".to_string(), forecast_type.to_string()));

        if let Some(date_range) = date_range {
            qs.append(&mut date_range.to_query_string());
        }

        let response = self
            .client
            .http_get(ConsumptionForecast::SHORT_TERM_URL, &qs);

        let reply = response.unwrap();

        let res = serde_json::from_str(&reply);
        if let Err(e) = res {
            eprintln!(
                "Error: parsing reply of {}?{:?} => '{:?}': {:?}",
                ConsumptionForecast::SHORT_TERM_URL,
                qs,
                reply,
                e
            );
            return Err(anyhow::Error::msg("Failed to parse response"));
        }
        Ok(res.unwrap())
    }

    pub fn weekly_forecast(
        &self,
        date_range: Option<DateRange>,
    ) -> Result<WeeklyForecastResponse, anyhow::Error> {
        let mut qs: Vec<(String, String)> = vec![];
        if let Some(date_range) = date_range {
            qs.append(&mut date_range.to_query_string());
        }

        let response = self.client.http_get(ConsumptionForecast::WEEKLY_URL, &qs);

        let reply = response.unwrap();

        let res = serde_json::from_str(&reply);
        if let Err(e) = res {
            eprintln!(
                "Error: parsing reply of {}?{:?} => '{:?}': {:?}",
                ConsumptionForecast::WEEKLY_URL,
                qs,
                reply,
                e
            );
            return Err(anyhow::Error::msg("Failed to parse response"));
        }
        Ok(res.unwrap())
    }
}

// XXX trait
impl ShortTermResponse {
    pub fn as_polars_df(&self) -> Result<polars::prelude::DataFrame, anyhow::Error> {
        let mut start_dates: Vec<NaiveDateTime> = vec![];
        let mut end_dates: Vec<NaiveDateTime> = vec![];
        let mut updated_dates: Vec<NaiveDateTime> = vec![];
        let mut values: Vec<f64> = vec![];

        let short_term_response = &self.short_term[0];

        for st in short_term_response.values.iter() {
            start_dates.push(st.start_date.naive_utc());
            end_dates.push(st.end_date.naive_utc());
            //if let Some(ud) = &st.updated_date {
            updated_dates.push(st.updated_date.naive_utc());
            // }
            values.push(st.value);
        }

        let start_dates_series = Series::new("start_date".into(), start_dates);
        let end_dates_series = Series::new("end_date".into(), end_dates);
        let updated_dates_series = Series::new("updated_date".into(), updated_dates);
        let values_series = Series::new("value".into(), values);

        let df = DataFrame::new(vec![
            start_dates_series,
            end_dates_series,
            updated_dates_series,
            values_series,
        ])?;

        Ok(df)
    }
}

impl WeeklyForecastResponse {
    pub fn as_polars_df(&self) -> Result<polars::prelude::DataFrame, anyhow::Error> {
        let mut start_dates: Vec<NaiveDateTime> = vec![];
        let mut end_dates: Vec<NaiveDateTime> = vec![];
        let mut updated_dates: Vec<NaiveDateTime> = vec![];
        let mut peak_temperatures: Vec<f64> = vec![];
        let mut peak_temperature_deviations: Vec<f64> = vec![];
        let mut values: Vec<f64> = vec![];

        for day_forecast in &self.weekly_forecasts {
            let temperature = day_forecast.peak.temperature;
            let temperature_deviation = day_forecast.peak.temperature_deviation;

            for wf in day_forecast.values.iter() {
                start_dates.push(wf.start_date.naive_utc());
                end_dates.push(wf.end_date.naive_utc());
                updated_dates.push(day_forecast.updated_date.naive_utc());
                peak_temperatures.push(temperature);
                peak_temperature_deviations.push(temperature_deviation);
                values.push(wf.value);
            }
        }

        let start_dates_series = Series::new("start_date".into(), start_dates);
        let end_dates_series = Series::new("end_date".into(), end_dates);
        let updated_dates_series = Series::new("updated_date".into(), updated_dates);
        let peak_temperatures_series = Series::new("peak_temperature".into(), peak_temperatures);
        let peak_temperature_deviations_series = Series::new(
            "peak_temperature_deviation".into(),
            peak_temperature_deviations,
        );

        let df = DataFrame::new(vec![
            start_dates_series,
            end_dates_series,
            updated_dates_series,
            peak_temperatures_series,
            peak_temperature_deviations_series,
            Series::new("value".into(), values),
        ])?;

        Ok(df)
    }
}
