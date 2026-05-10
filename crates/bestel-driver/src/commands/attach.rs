//! `bestel-driver attach [--port N]` — discover the running Bestel
//! instance and persist the port for subsequent commands.

use anyhow::Result;
use serde_json::json;

use crate::cdp;
use crate::session::{self, Session};

pub async fn run(port: u16) -> Result<()> {
    let targets = cdp::list_targets(port).await?;
    let target = cdp::pick_main_target(&targets)?;

    session::save(&Session {
        port,
        attached_at: chrono::Utc::now().to_rfc3339(),
    })?;

    let summary = json!({
        "status": "attached",
        "port": port,
        "target_id": target.id,
        "title": target.title,
        "url": target.url,
        "all_targets": targets.iter().map(|t| json!({
            "id": t.id,
            "type": t.kind,
            "title": t.title,
            "url": t.url,
        })).collect::<Vec<_>>(),
    });
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}
