use std::{
	path::Path,
	str::FromStr,
	sync::{Arc, RwLock},
};

use bm_version::VersionKey;
use ironworks::{Ironworks, excel::Excel, sqpack::SqPack};
use tokio::sync::watch;

use crate::error::{Error, Result};
use crate::install::Install;
use figment::value::magic::RelativePathBuf;

pub struct Data {
	channel: watch::Sender<Vec<VersionKey>>,
	version: Arc<RwLock<Option<Arc<Version>>>>,
	version_key: VersionKey,
	game_dir: RelativePathBuf,
}

impl Data {
	pub fn new(game_dir: RelativePathBuf) -> Self {
		let (sender, _receiver) = watch::channel(vec![]);

		Data {
			channel: sender,
			version: Arc::new(RwLock::new(None)),
			// Use a constant version key
			version_key: {
				// Convert RelativePathBuf to Path and append ffxivgame.ver
				let ver_path = game_dir.relative().join("ffxivgame.ver");
				let ver_str = std::fs::read_to_string(&ver_path)
					.expect(&format!("Failed to read {:?}", ver_path));
				let key_str: String = ver_str.chars().filter(|c| c.is_ascii_digit()).collect();
				VersionKey::from_str(&key_str).expect("Invalid version key in ffxivgame.ver")
			},
			game_dir,
		}
	}

	pub fn ready(&self) -> bool {
		self.version.read().expect("poisoned").is_some()
	}

	pub fn subscribe(&self) -> watch::Receiver<Vec<VersionKey>> {
		self.channel.subscribe()
	}

	/// Initialize with a single version using filesystem data
	pub fn initialize(&self) -> Result<()> {
		let game_dir = self.game_dir.relative();
		let version = Version::new(&game_dir)?;

		*self.version.write().expect("poisoned") = Some(Arc::new(version));

		// Broadcast the version list
		self.broadcast_version_list();

		Ok(())
	}

	pub fn version_key(&self) -> VersionKey {
		self.version_key
	}

	pub fn version(&self, version: VersionKey) -> Result<Arc<Version>> {
		self.version
			.read()
			.expect("poisoned")
			.clone()
			.ok_or_else(|| Error::UnknownVersion(version))
	}

	fn broadcast_version_list(&self) {
		let keys = vec![self.version_key];

		self.channel.send_if_modified(|value| {
			if &keys != value {
				*value = keys;
				return true;
			}
			false
		});
	}
}

pub struct Version {
	ironworks: Arc<Ironworks>,
	excel: Arc<Excel>,
}

impl Version {
	fn new(game_dir: &Path) -> Result<Self> {
		let install = Install::at(game_dir);
		let ironworks = Arc::new(Ironworks::new().with_resource(SqPack::new(install)));
		let excel = Arc::new(Excel::new(ironworks.clone()));
		Ok(Self { ironworks, excel })
	}

	pub fn ironworks(&self) -> Arc<Ironworks> {
		self.ironworks.clone()
	}

	pub fn excel(&self) -> Arc<Excel> {
		self.excel.clone()
	}
}
