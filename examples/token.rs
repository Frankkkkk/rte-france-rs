use rte_france::RteApi;

fn main() {
    let client_id = std::env::var("CLIENT_ID").expect("CLIENT_ID must be set");
    let client_secret = std::env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set");
    let mut rte_api = RteApi::new(client_id, client_secret);
    println!("rte_api: {:?}", rte_api);

    rte_api.authenticate().expect("Failed to authenticate");

    let token = rte_api.get_token();
    println!("token: {:?}", token);
}
