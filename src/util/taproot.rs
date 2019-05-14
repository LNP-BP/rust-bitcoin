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
macro_rules! tagged_hash {
	($name:ident, $tag:ident, $hash:ident, $midstate:ident) => {
		/// The `$name` hash tag.
		#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
		pub struct $tag;

		impl sha256t::Tag for $tag {
			fn engine() -> sha256::HashEngine {
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
	use hashes::{sha256, Hash, HashEngine};

	#[test]
	#[ignore]
	fn test_midstates() {
		fn calculate_tag_midstate(tag_name: &str) -> [u8; 32] {
			let mut engine = sha256::Hash::engine();
			engine.input(&tag_name.as_bytes()[..]);
			engine.input(&tag_name.as_bytes()[..]);
			engine.midstate().into_inner()
		}

		assert_eq!(MIDSTATE_TAPLEAF, calculate_tag_midstate("TapLeaf"));
		assert_eq!(MIDSTATE_TAPBRANCH, calculate_tag_midstate("TapBranch"));
		assert_eq!(MIDSTATE_TAPTWEAK, calculate_tag_midstate("TapTweak"));
		assert_eq!(MIDSTATE_TAPSIGHASH, calculate_tag_midstate("TapSigHash"));
	}
}
