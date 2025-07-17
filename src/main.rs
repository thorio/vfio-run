use cli::{Command, Profile};
use context::Context;
use nix::unistd::Uid;

mod cli;
mod config;
mod context;
mod runner;

fn main() {
	let cli = cli::parse();
	init_logger(cli.debug);
	log::debug!("{cli:?}");

	if !Uid::effective().is_root() {
		log::warn!("running as non-root, here be dragons");
	}

	match cli.command {
		Command::Run {
			profile,
			window,
			skip_attach,
		} => run(profile, window, skip_attach),
		Command::Detach { profile } => detach(profile),
		Command::Attach { profile } => attach(profile),
	}
}

fn detach(profile: Profile) {
	let context = get_context(&profile, false);

	runner::detach_devices(&context).ok();
}

fn attach(profile: Profile) {
	let context = get_context(&profile, false);

	runner::reattach_devices(&context).ok();
}

fn run(profile: Profile, window: bool, skip_attach: bool) {
	let context = get_context(&profile, window);

	if runner::run(context, skip_attach).is_ok() {
		log::info!("exit successful");
	}
}

fn get_context(profile: &Profile, window: bool) -> Context {
	let builder = config::get_builder(profile, window);
	log::debug!("{builder:?}");

	let context = builder.build();
	log::debug!("{context:?}");

	context
}

fn init_logger(debug: bool) {
	stderrlog::new()
		.timestamp(stderrlog::Timestamp::Off)
		.verbosity(if debug { 3 } else { 2 })
		.init()
		.expect("logger already initialized");
}
