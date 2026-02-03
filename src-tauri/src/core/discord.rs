use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        OnceLock,
    },
    thread,
    time::{Duration, Instant},
};

static LOG_SENDER: OnceLock<Sender<String>> = OnceLock::new();

pub fn init() {
    let (tx, rx) = channel();
    let _ = LOG_SENDER.set(tx);
    thread::spawn(move || rpc_loop(rx));
}

pub fn push_log(line: String) {
    if let Some(tx) = LOG_SENDER.get() {
        let _ = tx.send(line);
    }
}

fn discord_client_id() -> Option<String> {
    std::env::var("DISCORD_CLIENT_ID")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        // keep your fallback if you want it:
        .or_else(|| Some("1462544460675285014".to_string()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum McState {
    #[default]
    Launcher,
    Menus,
    Playing,
}

#[derive(Clone, Debug, Default)]
struct McContext {
    state: McState,
    version: Option<String>,
}

struct PresenceClient {
    client_id: String,
    client: Option<DiscordIpcClient>,

    last_state: String,
    last_details: String,
    last_large: String,
    last_small: String,

    last_connect_attempt: Instant,
    start_time: i64,
}

impl PresenceClient {
    fn new() -> Option<Self> {
        let client_id = discord_client_id()?;

        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        Some(Self {
            client_id,
            client: None,
            last_state: String::new(),
            last_details: String::new(),
            last_large: String::new(),
            last_small: String::new(),
            last_connect_attempt: Instant::now() - Duration::from_secs(10),
            start_time,
        })
    }

    fn ensure_connected(&mut self) -> bool {
        if self.client.is_some() {
            return true;
        }
        if self.last_connect_attempt.elapsed() < Duration::from_secs(3) {
            return false;
        }
        self.last_connect_attempt = Instant::now();

        let mut c = match DiscordIpcClient::new(self.client_id.as_str()) {
            Ok(c) => c,
            Err(_) => return false,
        };

        if c.connect().is_err() {
            return false;
        }

        self.client = Some(c);

        // force resend after reconnect
        self.last_state.clear();
        self.last_details.clear();
        self.last_large.clear();
        self.last_small.clear();

        true
    }

    fn disconnect(&mut self) {
        if let Some(mut c) = self.client.take() {
            let _ = c.close();
        }
    }

    fn set(&mut self, state: &str, details: &str, large: &str, small: &str) {
        if self.client.is_some()
            && self.last_state == state
            && self.last_details == details
            && self.last_large == large
            && self.last_small == small
        {
            return;
        }

        self.last_state.clear();
        self.last_state.push_str(state);

        self.last_details.clear();
        self.last_details.push_str(details);

        self.last_large.clear();
        self.last_large.push_str(large);

        self.last_small.clear();
        self.last_small.push_str(small);

        if !self.ensure_connected() {
            return;
        }

        let assets = activity::Assets::new()
            .large_image(large)
            .small_image(small);

        let timestamps = activity::Timestamps::new().start(self.start_time);

        let act = activity::Activity::new()
            .state(state)
            .details(details)
            .assets(assets)
            .timestamps(timestamps);

        if let Err(_) = self.client.as_mut().unwrap().set_activity(act) {
            self.disconnect();
        }
    }
}

fn parse_version(line: &str) -> Option<String> {
    // super common patterns across clients/modloaders
    for pat in ["Loading Minecraft", "Minecraft version:", "Running Minecraft"] {
        if let Some(rest) = line.split(pat).nth(1) {
            let token = rest.trim().split_whitespace().next()?;
            if token.chars().next()?.is_ascii_digit() {
                return Some(token.to_string());
            }
        }
    }
    None
}

fn detect_state_from_line(line: &str, ctx: &mut McContext) {
    if ctx.version.is_none() {
        if let Some(v) = parse_version(line) {
            ctx.version = Some(v);
        }
    }

    // “Playing” signals (singleplayer + multiplayer without server/IP)
    if line.contains("Starting integrated minecraft server")
        || line.contains("Preparing spawn area")
        || line.contains("Logged in with entity id")
        || line.contains("Connecting to")
        || line.contains("Joining world")
    {
        ctx.state = McState::Playing;
        return;
    }

    // back to menus / closed world
    if line.contains("Disconnecting from server")
        || line.contains("Stopping integrated server")
        || line.contains("Returning to title screen")
    {
        ctx.state = McState::Menus;
        return;
    }

    // game exit -> menus is still fine; launcher will keep launcher state until logs arrive
    if line.contains("Stopping!") {
        ctx.state = McState::Menus;
        return;
    }
}

fn apply_presence(p: &mut PresenceClient, ctx: &McContext) {
    let version = ctx.version.as_deref().unwrap_or("");

    match ctx.state {
        McState::Launcher => {
            // launcher view: large launcher, small minecraft
            p.set("In Launcher", "", "launcher", "minecraft");
        }
        McState::Menus => {
            // in game but not playing yet: large minecraft, small launcher
            p.set("In Menus", version, "minecraft", "launcher");
        }
        McState::Playing => {
            // clean lunar/feather vibe
            p.set("Playing Minecraft", version, "minecraft", "launcher");
        }
    }
}

fn rpc_loop(rx: Receiver<String>) {
    let mut presence = match PresenceClient::new() {
        Some(p) => p,
        None => return,
    };

    let mut ctx = McContext {
        state: McState::Launcher,
        version: None,
    };

    // initial presence
    apply_presence(&mut presence, &ctx);

    let mut last_heartbeat = Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_secs(2)) {
            Ok(line) => {
                // Any log line means MC is running / ran, so leave Launcher
                if ctx.state == McState::Launcher {
                    ctx.state = McState::Menus;
                }

                detect_state_from_line(&line, &mut ctx);
                apply_presence(&mut presence, &ctx);
                last_heartbeat = Instant::now();
            }
            Err(_) => {
                // heartbeat keeps it alive + re-push after discord restart
                if last_heartbeat.elapsed() >= Duration::from_secs(5) {
                    apply_presence(&mut presence, &ctx);
                    last_heartbeat = Instant::now();
                }
            }
        }
    }
}
