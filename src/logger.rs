use tracing_subscriber;

pub fn init() {
	// TODO: log to file
	tracing_subscriber::fmt::init();
}
