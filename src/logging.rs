use simplelog::*;

pub fn init() {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
    )
    .unwrap()])
    .unwrap();
}
