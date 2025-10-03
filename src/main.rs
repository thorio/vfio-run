use cli::{Command, Options};
use context::{Context, ContextBuilder};
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
		Command::Run { config, skip_attach } => run(config, skip_attach),
		Command::Detach { config } => detach(config),
		Command::Attach { config } => attach(config),
	}
}

fn detach(config: Options) {
	let context = get_context(&config);

	runner::detach_devices(&context).ok();
}

fn attach(config: Options) {
	let context = get_context(&config);

	runner::reattach_devices(&context);
}

fn run(config: Options, skip_attach: bool) {
	let context = get_context(&config);

	if runner::run(context, skip_attach).is_ok() {
		log::info!("exit successful");
	}
}

fn get_context(config: &Options) -> Context {
	let mut builder = ContextBuilder::default();
	config::configure(&mut builder, config);
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
