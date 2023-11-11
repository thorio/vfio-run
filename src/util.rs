use std::collections::HashMap;

#[derive(Default)]
pub struct ArgWriter {
	args: Vec<String>,
}

impl ArgWriter {
	pub fn push(&mut self, arg: impl Into<String>) -> &'_ mut Self {
		self.args.push(arg.into());

		self
	}

	pub fn push_many<T: Into<String>>(&mut self, args: Vec<T>) -> &'_ mut Self {
		for arg in args.into_iter() {
			self.args.push(arg.into());
		}

		self
	}

	pub fn get_args(self) -> Vec<String> {
		self.args
	}
}

#[derive(Default)]
pub struct EnvWriter {
	env: HashMap<String, String>,
}

impl EnvWriter {
	pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) -> &'_ mut Self {
		let key = key.into();
		if self.env.contains_key(&key) {
			// TODO error handling; use entry API?
			panic!("env conflict");
		}

		self.env.insert(key, value.into());
		self
	}

	pub fn get_envs(self) -> HashMap<String, String> {
		self.env
	}
}
