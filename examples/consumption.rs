use rte_france::api::consumption::ConsumptionForecast;
use rte_france::api::DateRange;
use rte_france::RteApi;

fn main() {
    let mut rte_api = RteApi::from_env_values();
    rte_api.authenticate().expect("Failed to authenticate");

    let consumption_forecast = ConsumptionForecast::new(&rte_api);

    let in_1h = chrono::Utc::now() + chrono::Duration::hours(1);

    let range = DateRange {
        start: in_1h,
        end: in_1h + chrono::Duration::hours(35),
    };
    println!("range: {:?}", range);
    //    let data =
    //        consumption_forecast.short_term(ShortTermForecastType::DayAfterTomorrow, Some(range));
    //    println!("data: {:?}", data);
    //    println!("{}", data.unwrap().as_polars_df().unwrap());

    let weekly = consumption_forecast.weekly_forecast(None);
    println!("data: {:?}", weekly);
    println!("{}", weekly.unwrap().as_polars_df().unwrap());
}
