// Rust Bitcoin Library
// Written in 2019 by
//     The rust-bitcoin developers.
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! Taproot
//!

use hashes::{sha256, sha256t};

/// The SHA-256 midstate value for the TapLeaf hash.
const MIDSTATE_TAPLEAF: [u8; 32] = [
	156, 224, 228, 230, 124, 17, 108, 57, 56, 179, 202, 242, 195, 15, 80, 137, 211, 243, 147, 108,
	71, 99, 110, 96, 125, 179, 62, 234, 221, 198, 240, 201,
];
// 9ce0e4e67c116c3938b3caf2c30f5089d3f3936c47636e607db33eeaddc6f0c9

/// The SHA-256 midstate value for the TapBranch hash.
const MIDSTATE_TAPBRANCH: [u8; 32] = [
	35, 168, 101, 169, 184, 164, 13, 167, 151, 124, 30, 4, 196, 158, 36, 111, 181, 190, 19, 118,
	157, 36, 201, 183, 181, 131, 181, 212, 168, 210, 38, 210,
];
// 23a865a9b8a40da7977c1e04c49e246fb5be13769d24c9b7b583b5d4a8d226d2

/// The SHA-256 midstate value for the TapTweak hash.
const MIDSTATE_TAPTWEAK: [u8; 32] = [
	209, 41, 162, 243, 112, 28, 101, 93, 101, 131, 182, 195, 185, 65, 151, 39, 149, 244, 226, 50,
	148, 253, 84, 244, 162, 174, 141, 133, 71, 202, 89, 11,
];
// d129a2f3701c655d6583b6c3b941972795f4e23294fd54f4a2ae8d8547ca590b

/// The SHA-256 midstate value for the TapSigHash hash.
const MIDSTATE_TAPSIGHASH: [u8; 32] = [
	245, 4, 164, 37, 215, 248, 120, 59, 19, 99, 134, 138, 227, 229, 86, 88, 110, 238, 148, 93, 188,
	120, 136, 221, 2, 166, 226, 195, 24, 115, 254, 159,
];
// f504a425d7f8783b1363868ae3e556586eee945dbc7888dd02a6e2c31873fe9f

/// Macro used to define a tagged hash as defined in the taproot BIP.
/// It creates two public types:
/// - a sha246t::Tag struct,
/// - a sha256t::Hash type alias.
#[macro_export]
macro_rules! tagged_hash {
	($name:ident, $tag:ident, $hash:ident, $midstate:ident) => {
		/// The `$name` hash tag.
		#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
		pub struct $tag;

		impl sha256t::Tag for $tag {
			fn engine() -> $crate::hashes::sha256::HashEngine {
				//TODO(stevenroose) optimize this when following two PRs are merged:
				// https://github.com/rust-bitcoin/bitcoin_hashes/pull/61
				// https://github.com/rust-bitcoin/bitcoin_hashes/pull/62
				let midstate = sha256::Midstate::from_inner($midstate.clone());
				sha256::HashEngine::from_midstate(midstate, 64)
			}
		}

		/// A hash tagged with `$name`.
		pub type $hash = sha256t::Hash<$tag>;
	};
}

tagged_hash!(TapLeaf, TapLeafTag, TapLeafHash, MIDSTATE_TAPLEAF);
tagged_hash!(TapBranch, TapBranchTag, TapBranchHash, MIDSTATE_TAPBRANCH);
tagged_hash!(TapTweak, TapTweakTag, TapTweakHash, MIDSTATE_TAPTWEAK);
tagged_hash!(TapSighash, TapSighashTag, TapSighashHash, MIDSTATE_TAPSIGHASH);

#[cfg(test)]
mod test {
	use super::*;
	use hashes::hex::ToHex;
	use hashes::sha256t::Tag;
	use hashes::{sha256, Hash, HashEngine};

	fn tag_engine(tag_name: &str) -> sha256::HashEngine {
		let mut engine = sha256::Hash::engine();
		let tag_hash = sha256::Hash::hash(tag_name.as_bytes());
		engine.input(&tag_hash[..]);
		engine.input(&tag_hash[..]);
		engine
	}

	#[test]
	#[ignore]
	fn test_midstates() {
		// check midstate against hard-coded values
		assert_eq!(MIDSTATE_TAPLEAF, tag_engine("TapLeaf").midstate().into_inner());
		assert_eq!(MIDSTATE_TAPBRANCH, tag_engine("TapBranch").midstate().into_inner());
		assert_eq!(MIDSTATE_TAPTWEAK, tag_engine("TapTweak").midstate().into_inner());
		assert_eq!(MIDSTATE_TAPSIGHASH, tag_engine("TapSighash").midstate().into_inner());

		// test that engine creation roundtrips
		assert_eq!(tag_engine("TapLeaf").midstate(), TapLeafTag::engine().midstate());
		assert_eq!(tag_engine("TapBranch").midstate(), TapBranchTag::engine().midstate());
		assert_eq!(tag_engine("TapTweak").midstate(), TapTweakTag::engine().midstate());
		assert_eq!(tag_engine("TapSighash").midstate(), TapSighashTag::engine().midstate());

		// check that hash creation is the same as building into the same engine
		fn empty_hash(tag_name: &str) -> [u8; 32] {
			let mut e = tag_engine(tag_name);
			e.input(&[]);
			sha256::Hash::from_engine(e).into_inner()
		}
		assert_eq!(empty_hash("TapLeaf"), TapLeafHash::hash(&[]).into_inner());
		assert_eq!(empty_hash("TapBranch"), TapBranchHash::hash(&[]).into_inner());
		assert_eq!(empty_hash("TapTweak"), TapTweakHash::hash(&[]).into_inner());
		assert_eq!(empty_hash("TapSighash"), TapSighashHash::hash(&[]).into_inner());
	}

	#[test]
	fn test_vectors_core() {
		//! Test vectors taken from Core

		// uninitialized writers
		//   CHashWriter writer = HasherTapLeaf;
		//   writer.GetSHA256().GetHex()
		assert_eq!(
			TapLeafHash::from_engine(TapLeafTag::engine()).to_hex(),
			"cbfa0621df37662ca57697e5847b6abaf92934a1a5624916f8d177a388c21252"
		);
		assert_eq!(
			TapBranchHash::from_engine(TapBranchTag::engine()).to_hex(),
			"dffd9fbe4c21c893fa934f8774eda0e1efdc06f52ffbf5c1533c6f4dec73c353"
		);
		assert_eq!(
			TapTweakHash::from_engine(TapTweakTag::engine()).to_hex(),
			"e4156b45ff9b277dd92a042af9eed8c91f1d037f68f0d6b20001ab749422a48a"
		);
		assert_eq!(
			TapSighashHash::from_engine(TapSighashTag::engine()).to_hex(),
			"03c8b9d47cdb5f7bf924e282ce99ba8d2fe581262a04002907d8bc4a9111bcda"
		);

		// 0-byte
		//   CHashWriter writer = HasherTapLeaf;
		//   writer << std::vector<unsigned char>{};
		//   writer.GetSHA256().GetHex()
		// Note that Core writes the 0 length prefix when an empty vector is written.
		assert_eq!(
			TapLeafHash::hash(&[0]).to_hex(),
			"29589d5122ec666ab5b4695070b6debc63881a4f85d88d93ddc90078038213ed"
		);
		assert_eq!(
			TapBranchHash::hash(&[0]).to_hex(),
			"1deb45569eb6b2da88b5c2ab46d6a64ab08d58a2fdd5f75a24e6c760194b5392"
		);
		assert_eq!(
			TapTweakHash::hash(&[0]).to_hex(),
			"1eea90d42a359c89bbf702ddf6bde140349e95b9e8036ff1c37f04e6b53787cd"
		);
		assert_eq!(
			TapSighashHash::hash(&[0]).to_hex(),
			"cd10c023c300fb9a507dff136370fba1d8a0566667cfafc4099a8803e00dfdc2"
		);
	}
}
