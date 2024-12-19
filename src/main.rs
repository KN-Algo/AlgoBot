use serenity::{all::GatewayIntents, Client};

fn read_token() -> Result<String, std::io::Error> {
    std::fs::read_to_string("token")
}

#[tokio::main]
async fn main() -> serenity::Result<()> {
    let token = match read_token() {
        Ok(t) => t,
        Err(e) => panic!("Couldn't read token! {e}"),
    };

    match serenity::utils::validate_token(&token) {
        Ok(_) => (),
        Err(e) => panic!("Invalid token!: {e}"),
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents).await?;

    client.start().await?;
    Ok(())
}
