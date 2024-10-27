use core::fmt;

use chrono::{DateTime, NaiveDateTime, Utc};
use polars::prelude::*;
use polars::{frame::DataFrame, series::Series};
use serde::Deserialize;

use crate::ApiClient;

use super::DateRange;

pub struct GenerationForecast<'a> {
    client: &'a dyn ApiClient,
}

#[derive(Debug)]
pub enum ProductionType {
    /// Agrégée France
    AggregatedFrance,
    /// Eolien terrestre
    WindOnshore,
    /// Eolien en mer
    WindOffshore,
    /// Solaire
    Solar,
    /// Agrégée OA
    AggregatedCpc,
    /// Production potentielle des cogénérations MDSE (Mise à disposition du système électrique)
    Mdse,
}

impl fmt::Display for ProductionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pt = match self {
            ProductionType::AggregatedFrance => "AGGREGATED_FRANCE",
            ProductionType::WindOnshore => "WIND_ONSHORE",
            ProductionType::WindOffshore => "WIND_OFFSHORE",
            ProductionType::Solar => "SOLAR",
            ProductionType::AggregatedCpc => "AGGREGATED_CPC",
            ProductionType::Mdse => "MDSE",
        };
        write!(f, "{}", pt)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProductionTypeResponse {
    /// production des moyens programmables agrégée sur la France
    AggregatedProgrammableFrance,
    /// production des moyens dits "fatals"  agrégée sur la France
    AggregatedNonProgrammableFrance,

    WindOnshore,
    WindOffshore,
    Solar,
    AggregatedCpc,
    /// Installations bénéficiant d'un contrat d'achat indexé aux prix de marché Trading Region France
    #[serde(rename = "MDSETRF")]
    MdseTrf,
    /// Installations bénéficiant d'un contrat d'achat indexé sur le tarif réglementé de fourniture de gaz STS
    #[serde(rename = "MDSESTS")]
    MdseSts,
}
impl fmt::Display for ProductionTypeResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pt = match self {
            ProductionTypeResponse::AggregatedProgrammableFrance => {
                "AGGREGATED_PROGRAMMABLE_FRANCE"
            }
            ProductionTypeResponse::AggregatedNonProgrammableFrance => {
                "AGGREGATED_NON_PROGRAMMABLE_FRANCE"
            }
            ProductionTypeResponse::WindOnshore => "WIND_ONSHORE",
            ProductionTypeResponse::WindOffshore => "WIND_OFFSHORE",
            ProductionTypeResponse::Solar => "SOLAR",
            ProductionTypeResponse::AggregatedCpc => "AGGREGATED_CPC",
            ProductionTypeResponse::MdseTrf => "MDSE_TRF",
            ProductionTypeResponse::MdseSts => "MDSE_STS",
        };
        write!(f, "{}", pt)
    }
}

#[derive(Debug, Deserialize)]
pub enum ForecastType {
    Current,
    Intraday,
    Tomorrow,
    AfterTomorrow,
    AfterAfterTomorrow, // Lol, this is a dumb name
}

impl fmt::Display for ForecastType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ft = match self {
            ForecastType::Current => "CURRENT",
            ForecastType::Intraday => "ID",
            ForecastType::Tomorrow => "D-1",
            ForecastType::AfterTomorrow => "D-2",
            ForecastType::AfterAfterTomorrow => "D-3",
        };
        write!(f, "{}", ft)
    }
}

#[derive(Deserialize, Debug)]
pub struct ForecastResponse {
    pub forecasts: Vec<Forecast>,
}

#[derive(Deserialize, Debug)]
pub struct Forecast {
    #[serde(rename = "type")]
    pub ty: String, // XXX enum

    pub sub_type: Option<String>,                // XXX enum
    pub production_type: ProductionTypeResponse, // XXX enum

    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub values: Vec<ForecastValue>,
}

#[derive(Deserialize, Debug)]
pub struct ForecastValue {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub value: f64,
    pub load_factor: Option<f64>, // only for production_type: Wind*
}

impl<'a> GenerationForecast<'a> {
    const URL: &'static str = "/open_api/generation_forecast/v2/forecasts";

    pub fn new(client: &'a dyn ApiClient) -> Self {
        Self { client }
    }

    /// Returns a short term forecast given the forecast type
    pub fn short_term(
        &self,
        production_type: Option<ProductionType>,
        forecast_type: Option<ForecastType>,
        date_range: Option<DateRange>,
    ) -> Result<ForecastResponse, anyhow::Error> {
        let mut qs: Vec<(String, String)> = vec![];

        //qs.push(("type".to_string(), forecast_type.to_string()));

        if let Some(production_type) = production_type {
            qs.push(("production_type".to_string(), production_type.to_string()));
        }
        if let Some(forecast_type) = forecast_type {
            qs.push(("type".to_string(), forecast_type.to_string()));
        }

        if let Some(date_range) = date_range {
            qs.append(&mut date_range.to_query_string());
        }

        let response = self.client.http_get(GenerationForecast::URL, &qs);

        let reply = response.unwrap();

        let res = serde_json::from_str(&reply);
        if let Err(e) = res {
            eprintln!(
                "Error: parsing reply of {}?{:?} => '{:?}': {:?}",
                GenerationForecast::URL,
                qs,
                reply,
                e
            );
            return Err(anyhow::Error::msg("Failed to parse response"));
        }
        Ok(res.unwrap())
    }
}

impl Forecast {
    pub fn as_polars_df(&self) -> Result<polars::prelude::DataFrame, anyhow::Error> {
        let mut start_dates: Vec<NaiveDateTime> = vec![];
        let mut end_dates: Vec<NaiveDateTime> = vec![];
        let mut updated_dates: Vec<NaiveDateTime> = vec![];
        let mut values: Vec<f64> = vec![];
        let mut load_factors: Vec<Option<f64>> = vec![];

        for fv in &self.values {
            start_dates.push(fv.start_date.naive_utc());
            end_dates.push(fv.end_date.naive_utc());
            updated_dates.push(fv.updated_date.naive_utc());
            values.push(fv.value);
            load_factors.push(fv.load_factor);
        }

        let start_dates_series = Series::new("start_date".into(), start_dates);
        let end_dates_series = Series::new("end_date".into(), end_dates);
        let updated_dates_series = Series::new("updated_date".into(), updated_dates);
        let value_series = Series::new("value".into(), values);
        let lf_series = Series::new("load_factor".into(), load_factors);

        let df = DataFrame::new(vec![
            start_dates_series,
            end_dates_series,
            updated_dates_series,
            value_series,
            lf_series,
        ])?;

        Ok(df)
    }
}
