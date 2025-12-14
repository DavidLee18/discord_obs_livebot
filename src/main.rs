use std::env;

use ::serenity::all::GuildId;
use poise::serenity_prelude as serenity;

struct Data(); // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

async fn get_url(locale: Option<&str>, value: Option<String>) -> Result<String, Error> {
    let lit_singil = match locale {
        Some("ko") => env::var("SERVER_SINGIL_KO")?,
        _ => env::var("SERVER_SINGIL_EN")?,
    };
    let lit_cwmc = match locale {
        Some("ko") => env::var("SERVER_CWMC_KO")?,
        _ => env::var("SERVER_CWMC_EN")?,
    };
    match value {
        None => Ok(String::from("SERVER_SINGIL_URL")),
        Some(v) if v == lit_singil => Ok(String::from("SERVER_SINGIL_URL")),
        Some(v_) if v_ == lit_cwmc => Ok(String::from("SERVER_CWMC_URL")),
        _ => Err("Invalid server location".into()),
    }
}

#[poise::command(
    slash_command,
    name_localized("ko", "정보"),
    description_localized("ko", "현재 OBS 상태를 볼 수 있습니다")
)]
async fn info(
    ctx: Context<'_>,
    #[name_localized("ko", "위치")]
    #[description = "server location"]
    #[description_localized("ko", "서버의 위치입니다")]
    where_: Option<String>,
) -> Result<(), Error> {
    let location = format!(
        "http://{}:{}",
        env::var(get_url(ctx.locale(), where_).await?)?,
        env::var("SERVER_PORT")?
    );
    match reqwest::get(location).await?.error_for_status() {
        Ok(response) => {
            let response_text = response.text().await?.replace("\\n", "\n");
            ctx.say(response_text).await?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[poise::command(
    slash_command,
    name_localized("ko", "중단"),
    description_localized("ko", "OBS에서 방송 송출을 중단합니다")
)]
async fn stop(
    ctx: Context<'_>,
    #[name_localized("ko", "위치")]
    #[description = "server location"]
    #[description_localized("ko", "서버의 위치입니다")]
    where_: Option<String>,
) -> Result<(), Error> {
    let location = format!(
        "http://{}:{}",
        env::var(get_url(ctx.locale(), where_).await?)?,
        env::var("SERVER_PORT")?
    );
    match reqwest::Client::new()
        .delete(location)
        .send()
        .await?
        .error_for_status()
    {
        Ok(response) => {
            let response_text = response.text().await?.replace("\\n", "\n");
            ctx.say(response_text).await?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[poise::command(
    slash_command,
    name_localized("ko", "전환"),
    description_localized("ko", "OBS에서 화면을 전환합니다")
)]
async fn switch(
    ctx: Context<'_>,
    #[name_localized("ko", "위치")]
    #[description = "server location"]
    #[description_localized("ko", "서버의 위치입니다")]
    where_: Option<String>,
    #[name_localized("ko", "화면")]
    #[description = "scene to change into"]
    #[description_localized("ko", "전환할 화면의 이름입니다")]
    scene: String,
) -> Result<(), Error> {
    let location = format!(
        "http://{}:{}",
        env::var(get_url(ctx.locale(), where_).await?)?,
        env::var("SERVER_PORT")?
    );
    match reqwest::Client::new()
        .post(location)
        .body(serde_json::to_string(
            &serde_json::json!({ "scene": scene }),
        )?)
        .send()
        .await?
        .error_for_status()
    {
        Ok(response) => {
            let response_text = response.text().await?.replace("\\n", "\n");
            ctx.say(response_text).await?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = std::env::var("DISCORD_BOT_TOKEN").expect("missing DISCORD_BOT_TOKEN");
    let guild_id = std::env::var("DISCORD_CHANNEL_ID").expect("missing DISCORD_CHANNEL_ID");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![info(), stop(), switch()],
            ..Default::default()
        })
        .setup(|ctx, _, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GuildId::new(guild_id.parse()?),
                )
                .await?;
                Ok(Data())
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;
    if let Err(e) = client.start().await {
        eprintln!("Error starting a bot: {:?}", e);
    }
    Ok(())
}
