use rte_france::api::generation::ForecastType;
use rte_france::api::generation::GenerationForecast;
use rte_france::api::generation::ProductionType;
use rte_france::api::DateRange;
use rte_france::RteApi;

fn main() {
    let mut rte_api = RteApi::from_env_values();
    rte_api.authenticate().expect("Failed to authenticate");

    let gf = GenerationForecast::new(&rte_api);

    let in_1h = chrono::Utc::now() + chrono::Duration::hours(1);

    let range = DateRange {
        start: in_1h,
        end: in_1h + chrono::Duration::hours(23),
    };

    let forecast = gf.short_term(
        Some(ProductionType::Solar),
        None,        //Some(ForecastType::AfterAfterTomorrow),
        Some(range), //None,
    );
    for forecast in forecast.unwrap().forecasts {
        println!(
            "forecast: {:?} / {:?} / {:?}",
            forecast.ty, forecast.sub_type, forecast.production_type
        );
        println!("{}", forecast.as_polars_df().unwrap());
    }
}
