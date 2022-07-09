use std::fs::File;

use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

pub fn init() -> anyhow::Result<()> {
	CombinedLogger::init(vec![
		TermLogger::new(
			LevelFilter::Debug,
			Config::default(),
			TerminalMode::Mixed,
			ColorChoice::Auto,
		),
		WriteLogger::new(
			LevelFilter::Info,
			Config::default(),
			File::create("logs.log")?
		),
	])?;

	Ok(())
}
