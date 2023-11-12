use cli::{Command, Configuration};
use context::Context;
use log::*;
use nix::unistd::Uid;

mod cli;
mod config;
mod context;
mod modprobe;
mod runner;
mod util;
mod virsh;

fn main() {
	let cli = cli::parse();
	init_logger(cli.debug);
	debug!("{:?}", cli);

	if !Uid::effective().is_root() {
		warn!("running as non-root, here be dragons");
	}

	match cli.command {
		Command::Run {
			configuration,
			window,
			skip_attach,
		} => run(window, configuration, skip_attach),
		Command::Detach { configuration } => detach(configuration),
		Command::Attach { configuration } => attach(configuration),
	}
}

fn detach(configuration: Configuration) {
	let context = get_context(false, &configuration);

	runner::detach_devices(&context).ok();
}

fn attach(configuration: Configuration) {
	let context = get_context(false, &configuration);

	runner::reattach_devices(&context).ok();
}

fn run(window: bool, configuration: Configuration, skip_attach: bool) {
	let context = get_context(window, &configuration);

	if runner::run(context, skip_attach).is_ok() {
		info!("exit successful")
	}
}

fn get_context(window: bool, configuration: &Configuration) -> Context {
	let builder = config::get_builder(window, configuration);
	debug!("{:?}", builder);

	let context = builder.build();
	debug!("{:?}", context);

	context
}

fn init_logger(debug: bool) {
	stderrlog::new()
		.timestamp(stderrlog::Timestamp::Off)
		.verbosity(if debug { 3 } else { 2 })
		.init()
		.expect("logger already initialized");
}
