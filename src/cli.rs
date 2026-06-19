use std::env;

pub struct Config {
    pub tick_rate_ms: u64,
    pub no_mouse: bool,
    pub initial_filter: String,
    pub no_truecolor: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_rate_ms: 500,
            no_mouse: false,
            initial_filter: String::new(),
            no_truecolor: false,
        }
    }
}

pub fn parse_args() -> Config {
    let mut config = Config::default();
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--interval" => {
                if let Some(val) = args.get(i + 1) {
                    if let Ok(ms) = val.parse::<u64>() {
                        config.tick_rate_ms = ms.max(50);
                    }
                    i += 1;
                }
            }
            "--no-mouse" => {
                config.no_mouse = true;
            }
            "--filter" => {
                if let Some(val) = args.get(i + 1) {
                    config.initial_filter = val.clone();
                    i += 1;
                }
            }
            "--no-truecolor" => {
                config.no_truecolor = true;
            }
            "--help" | "-h" => {
                println!("resource-monitor - Terminal resource monitor");
                println!();
                println!("USAGE:");
                println!("  resource-monitor [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("  --interval <ms>     Refresh interval in milliseconds (default: 500, min: 50)");
                println!("  --no-mouse          Disable mouse capture");
                println!("  --filter <query>    Pre-fill process search filter on startup");
                println!("  --no-truecolor      Use 256-color ANSI fallback instead of TrueColor");
                println!("  -h, --help          Show this help message");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }
    config
}
