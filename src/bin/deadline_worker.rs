use legal_agent_failure_tracker::{
    infrai_client::InfraiClient,
    matter_loop::{track_failure, AgentFailure},
};

#[tokio::main]
async fn main() {
    let matter_id = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "MAT-1042".to_owned());
    let client = match InfraiClient::from_env() {
        Ok(client) => client,
        Err(error) => {
            eprintln!("configuration: {error}");
            std::process::exit(2);
        }
    };

    match track_failure(&client, &matter_id, AgentFailure::DeliveryUnconfirmed).await {
        Ok(disposition) => println!("matter={matter_id} disposition={disposition:?}"),
        Err(error) => {
            eprintln!("worker: {error:?}");
            std::process::exit(1);
        }
    }
}
