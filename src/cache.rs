#[cfg(test)]
mod test {
    use lta::prelude::*;
    use lta::r#async::bus::get_bus_services;
    use lta::r#async::bus::get_bus_stops;
    use lta::r#async::lta_client::LTAClient;
    use std::env::var;

    #[actix_rt::test]
    async fn get_no_bus_stops() {
        let api_key = var("API_KEY").expect("API_KEY NOT FOUND!");
        let client = LTAClient::with_api_key(api_key);
        let mut bus_stops = vec![];

        for skip in (0..=5500).step_by(500) {
            let skip = if skip == 0 { None } else { Some(skip) };

            let mut buses = get_bus_stops(&client, skip).await.unwrap();
            bus_stops.append(&mut buses);
        }

        let bus_stop_code = bus_stops
            .into_iter()
            .map(|v| v.bus_stop_code)
            .collect::<Vec<_>>();

        dbg!(bus_stop_code);
    }
}
