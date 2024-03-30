use crate::context::TmpFile;
use nix::sys::stat::Mode;
use nix::unistd::{Gid, Uid};
use std::{collections::HashMap, path::PathBuf};

#[derive(Default)]
pub struct ArgWriter {
	args: Vec<String>,
}

impl ArgWriter {
	pub fn add(&mut self, arg: impl Into<String>) -> &'_ mut Self {
		self.args.push(arg.into());

		self
	}

	pub fn add_many<T: Into<String>>(&mut self, args: Vec<T>) -> &'_ mut Self {
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
			panic!("env conflict: {}", key);
		}

		self.env.insert(key, value.into());
		self
	}

	pub fn get_envs(self) -> HashMap<String, String> {
		self.env
	}
}

#[derive(Default)]
pub struct TmpFileWriter {
	files: Vec<TmpFile>,
}

impl TmpFileWriter {
	pub fn add(&mut self, path: impl Into<PathBuf>, uid: Uid, gid: Gid, mode: Mode) -> &'_ mut Self {
		self.files.push(TmpFile {
			path: path.into(),
			uid,
			gid,
			mode,
		});
		self
	}

	pub fn get_tmp_files(self) -> Vec<TmpFile> {
		self.files
	}
}
