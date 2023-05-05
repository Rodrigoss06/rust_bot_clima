use dotenv::dotenv;
use reqwest;
use serenity::builder::CreateEmbed;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::env;

use serde::Deserialize;
use serenity::async_trait;
use serenity::client::{Client, Context};

#[derive(Deserialize)]
struct OpenWeatherResponse {
    weather: Vec<Weather>,
    main: Main,
}

#[derive(Deserialize)]
struct Weather {
    description: String,
}

#[derive(Deserialize)]
struct Main {
    temp: f32,
    feels_like: f32,
}
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("!clima") {
            let city = msg.content.trim_start_matches("!clima ");
            let api_key: String =
                env::var("OPENWEATHER_API_KEY").expect("expected OPENWEATHER_API_KEY");

            let url = format!(
                "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
                city, api_key
            );

            let res = reqwest::get(url).await;
            let body = match res {
                Ok(res) => match res.json::<OpenWeatherResponse>().await {
                    Ok(body) => body,
                    Err(e) => {
                        println!("Error al obtener el JSON: {}", e);
                        return;
                    }
                },
                Err(e) => {
                    println!("Error al realizar la petición HTTP: {}", e);
                    return;
                }
            };
            let weather_description = body.weather[0].description.clone();
            let temperature = body.main.temp;
            let feels_like = body.main.feels_like;

            let mut embed = CreateEmbed::default();
            embed.title(format!("Weather in {}", city));
            embed.color(0x00FF00);
            embed.description(format!(
                "Current weather: {}\nTemperature: {:.1}°C\nFeels like: {:.1}°C",
                weather_description, temperature, feels_like
            ));
            embed.thumbnail("https://www.example.com/weather.png");

            if let Err(why) = msg
                .channel_id
                .send_message(&ctx.http, |m| {
                    m.content("test").embed(|e| {
                        e.title(format!("El clima en {}", city))
                        .description(format!(
                            "Current weather: {}\nTemperature: {:.1}°C\nFeels like: {:.1}°C",
                            weather_description, temperature, feels_like
                        ))
                        .color(0x00FF00)
                    })
                })
                .await
            {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Cargar las variables de entorno del archivo .env
    dotenv().ok();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("Discord_token").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
