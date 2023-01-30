pub struct Config {
    pub cmd_cooldown_ms: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cmd_cooldown_ms: 20,
        }
    }
}
