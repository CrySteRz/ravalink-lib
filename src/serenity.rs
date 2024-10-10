use futures::executor;
use std::sync::Arc;

use crate::{init_ravalink, Ravalink, RavalinkConfig};
use serenity::prelude::TypeMapKey;
pub use serenity::client::ClientBuilder;
use serenity::*;
use tokio::sync::Mutex;

pub struct RavalinkKey;

impl TypeMapKey for RavalinkKey {
    type Value = Arc<Mutex<Ravalink>>;
}

pub trait SerenityInit {
    #[must_use]
    fn register_ravalink(self, broker: String, config: RavalinkConfig) -> Self;
}

impl SerenityInit for ClientBuilder {
    fn register_ravalink(self, broker: String, config: RavalinkConfig) -> Self {
        let c = init_ravalink(broker, config);
        self.type_map_insert::<RavalinkKey>(executor::block_on(c))
    }
}


#[macro_export]
macro_rules! get_handler_from_interaction_mutable {
    ($ctx: expr, $interaction: expr, $reference: ident) => {
        let r = $ctx.data.read().await;
        let guild_id = match $interaction.guild_id {
            Some(gid) => gid,
            None => {
                eprintln!("No guild ID found in interaction");
                return Ok(());
            }
        };
        let manager = r.get::<RavalinkKey>();
        let mut mx = manager.unwrap().lock().await;
        let mut players = mx.players.write().await;
        $reference = players.get_mut(&guild_id.to_string());
    };
}
#[macro_export]
macro_rules! get_handler_from_interaction {
    ($ctx: expr, $interaction: expr, $reference: ident) => {
        let r = $ctx.data.read().await;
        let guild_id = match $interaction.guild_id {
            Some(gid) => gid,
            None => {
                eprintln!("No guild ID found in interaction");
                return Ok(());
            }
        };
        let manager = r.get::<RavalinkKey>();
        let mut mx = manager.unwrap().lock().await;
        let mut players = mx.players.write().await;
        $reference = players.get_mut(&guild_id.to_string());
    };
}