use std::env;

use ::serenity::all::GuildId;
use poise::serenity_prelude as serenity;

struct Data(); // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(
    slash_command,
    name_localized("ko", "정보"),
    description_localized("ko", "현재 OBS 상태를 볼 수 있습니다")
)]
async fn info(
    ctx: Context<'_>,
    #[description = "server location"]
    #[description_localized("ko", "서버의 위치입니다")]
    where_: Option<String>,
) -> Result<(), Error> {
    let lit_singil = match ctx.locale() {
        Some("ko") => env::var("SERVER_SINGIL_KO")?,
        _ => env::var("SERVER_SINGIL_EN")?,
    };
    let lit_cwmc = match ctx.locale() {
        Some("ko") => env::var("SERVER_CWMC_KO")?,
        _ => env::var("SERVER_CWMC_EN")?,
    };
    let u = where_.unwrap_or_else(|| lit_singil.clone());
    let loc = if u == lit_singil {
        "SERVER_SINGIL_URL"
    } else if u == lit_cwmc {
        "SERVER_CWMC_URL"
    } else {
        return Err("Invalid server location".into());
    };
    let location = format!("http://{}:{}", env::var(loc)?, env::var("SERVER_PORT")?);
    match reqwest::blocking::get(location)?.error_for_status() {
        Ok(response) => {
            let response_text = response.text()?.replace("\\n", "\n");
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
            commands: vec![info()],
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
