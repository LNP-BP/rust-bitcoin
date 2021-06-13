// Rust Bitcoin Library
// Written in 2014 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! Addresses
//!
//! Support for ordinary base58 Bitcoin addresses and private keys
//!
//! # Example: creating a new address from a randomly-generated key pair
//!
//! ```rust
//!
//! use bitcoin::network::constants::Network;
//! use bitcoin::util::address::Address;
//! use bitcoin::util::ecdsa;
//! use bitcoin::secp256k1::Secp256k1;
//! use bitcoin::secp256k1::rand::thread_rng;
//!
//! // Generate random key pair
//! let s = Secp256k1::new();
//! let public_key = ecdsa::PublicKey::new(s.generate_keypair(&mut thread_rng()).1);
//!
//! // Generate pay-to-pubkey-hash address
//! let address = Address::p2pkh(&public_key, Network::Bitcoin);
//! ```

use std::fmt;
use std::str::FromStr;
use std::error;
use std::num::ParseIntError;

use bech32;
use hashes::Hash;
use hash_types::{PubkeyHash, WPubkeyHash, ScriptHash, WScriptHash};
use blockdata::{script, opcodes};
use network::constants::Network;
use util::base58;
use util::ecdsa;
use blockdata::script::Instruction;

/// Address error.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    /// Base58 encoding error
    Base58(base58::Error),
    /// Bech32 encoding error
    Bech32(bech32::Error),
    /// The bech32 payload was empty
    EmptyBech32Payload,
    /// Script version must be 0 to 16 inclusive
    InvalidWitnessVersion(u8),
    /// Unable to parse witness version from string or opcode value
    UnparsableWitnessVersion(ParseIntError),
    /// Bitcoin script opcode does not match any known witness version, the script if malformed
    MalformedWitnessVersion,
    /// The witness program must be between 2 and 40 bytes in length.
    InvalidWitnessProgramLength(usize),
    /// A v0 witness program must be either of length 20 or 32.
    InvalidSegwitV0ProgramLength(usize),
    /// An uncompressed pubkey was used where it is not allowed.
    UncompressedPubkey,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Base58(ref e) => write!(f, "base58: {}", e),
            Error::Bech32(ref e) => write!(f, "bech32: {}", e),
            Error::EmptyBech32Payload => write!(f, "the bech32 payload was empty"),
            Error::InvalidWitnessVersion(v) => write!(f, "invalid witness script version: {}", v),
            Error::UnparsableWitnessVersion(ref e) => write!(f, "Incorrect format of a witness version byte: {}", e),
            Error::MalformedWitnessVersion => f.write_str("bitcoin script opcode does not match any known witness version, the script if malformed"),
            Error::InvalidWitnessProgramLength(l) => write!(f,
                "the witness program must be between 2 and 40 bytes in length: length={}", l,
            ),
            Error::InvalidSegwitV0ProgramLength(l) => write!(f,
                "a v0 witness program must be either of length 20 or 32 bytes: length={}", l,
            ),
            Error::UncompressedPubkey => write!(f,
                "an uncompressed pubkey was used where it is not allowed",
            ),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::Base58(ref e) => Some(e),
            Error::Bech32(ref e) => Some(e),
            Error::UnparsableWitnessVersion(ref e) => Some(e),
            _ => None,
        }
    }
}

#[doc(hidden)]
impl From<base58::Error> for Error {
    fn from(e: base58::Error) -> Error {
        Error::Base58(e)
    }
}

#[doc(hidden)]
impl From<bech32::Error> for Error {
    fn from(e: bech32::Error) -> Error {
        Error::Bech32(e)
    }
}

/// The different types of addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AddressType {
    /// pay-to-pubkey-hash
    P2pkh,
    /// pay-to-script-hash
    P2sh,
    /// pay-to-witness-pubkey-hash
    P2wpkh,
    /// pay-to-witness-script-hash
    P2wsh,
}

impl fmt::Display for AddressType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            AddressType::P2pkh => "p2pkh",
            AddressType::P2sh => "p2sh",
            AddressType::P2wpkh => "p2wpkh",
            AddressType::P2wsh => "p2wsh",
        })
    }
}

impl FromStr for AddressType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "p2pkh" => Ok(AddressType::P2pkh),
            "p2sh" => Ok(AddressType::P2sh),
            "p2wpkh" => Ok(AddressType::P2wpkh),
            "p2wsh" => Ok(AddressType::P2wsh),
            _ => Err(()),
        }
    }
}

/// Version of the WitnessProgram: first byte of `scriptPubkey` in
/// transaction output for transactions starting with opcodes ranging from 0
/// to 16 (inclusive).
///
/// Structure helps to limit possible version of the witness according to the
/// specification; if a plain `u8` type will be used instead it will mean that
/// version > 16, which is incorrect.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum WitnessVersion {
    /// Initial version of Witness Program. Used for P2WPKH and P2WPK outputs
    V0 = 0,
    /// Version of Witness Program used for Taproot P2TR outputs
    V1 = 1,
    /// Future (unsupported) version of Witness Program
    V2 = 2,
    /// Future (unsupported) version of Witness Program
    V3 = 3,
    /// Future (unsupported) version of Witness Program
    V4 = 4,
    /// Future (unsupported) version of Witness Program
    V5 = 5,
    /// Future (unsupported) version of Witness Program
    V6 = 6,
    /// Future (unsupported) version of Witness Program
    V7 = 7,
    /// Future (unsupported) version of Witness Program
    V8 = 8,
    /// Future (unsupported) version of Witness Program
    V9 = 9,
    /// Future (unsupported) version of Witness Program
    V10 = 10,
    /// Future (unsupported) version of Witness Program
    V11 = 11,
    /// Future (unsupported) version of Witness Program
    V12 = 12,
    /// Future (unsupported) version of Witness Program
    V13 = 13,
    /// Future (unsupported) version of Witness Program
    V14 = 14,
    /// Future (unsupported) version of Witness Program
    V15 = 15,
    /// Future (unsupported) version of Witness Program
    V16 = 16,
}

impl fmt::Display for WitnessVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}


impl FromStr for WitnessVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let version = u8::from_str(s).map_err(|err| Error::UnparsableWitnessVersion(err))?;
        WitnessVersion::from_no(version)
    }
}

impl WitnessVersion {
    /// Converts 5-bit unsigned integer value matching single symbol from Bech32(m) address encoding
    /// ([`bech32::u5`]) into [`WitnessVersion`] variant.
    ///
    /// # Returns
    /// Version of the Witness program
    ///
    /// # Errors
    /// If the integer does not corresponds to any witness version, errors with
    /// [`Error::InvalidWitnessVersion`]
    pub fn from_u5(value: ::bech32::u5) -> Result<Self, Error> {
        WitnessVersion::from_no(value.to_u8())
    }

    /// Converts 8-bit unsigned integer value into [`WitnessVersion`] variant.
    ///
    /// # Returns
    /// Version of the Witness program
    ///
    /// # Errors
    /// If the integer does not corresponds to any witness version, errors with
    /// [`Error::InvalidWitnessVersion`]
    pub fn from_no(no: u8) -> Result<Self, Error> {
        Ok(match no {
            0 => WitnessVersion::V0,
            1 => WitnessVersion::V1,
            2 => WitnessVersion::V2,
            3 => WitnessVersion::V3,
            4 => WitnessVersion::V4,
            5 => WitnessVersion::V5,
            6 => WitnessVersion::V6,
            7 => WitnessVersion::V7,
            8 => WitnessVersion::V8,
            9 => WitnessVersion::V9,
            10 => WitnessVersion::V10,
            11 => WitnessVersion::V11,
            12 => WitnessVersion::V12,
            13 => WitnessVersion::V13,
            14 => WitnessVersion::V14,
            15 => WitnessVersion::V15,
            16 => WitnessVersion::V16,
            wrong => Err(Error::InvalidWitnessVersion(wrong))?,
        })
    }

    /// Converts bitcoin script opcode into [`WitnessVersion`] variant.
    ///
    /// # Returns
    /// Version of the Witness program (for opcodes in range of `OP_0`..`OP_16`)
    ///
    /// # Errors
    /// If the opcode does not corresponds to any witness version, errors with
    /// [`Error::UnparsableWitnessVersion`] for the rest of opcodes
    pub fn from_opcode(opcode: opcodes::All) -> Result<Self, Error> {
        match opcode.into_u8() {
            0 => Ok(WitnessVersion::V0),
            version if version >= opcodes::all::OP_PUSHNUM_1.into_u8() && version <= opcodes::all::OP_PUSHNUM_16.into_u8() =>
                WitnessVersion::from_no(version - opcodes::all::OP_PUSHNUM_1.into_u8() + 1),
            _ => Err(Error::MalformedWitnessVersion)
        }
    }

    /// Converts bitcoin script [`Instruction`] (parsed opcode) into [`WitnessVersion`] variant.
    ///
    /// # Returns
    /// Version of the Witness program for [`Instruction::Op`] and [`Instruction::PushBytes`] with
    /// byte value within `1..=16` range.
    ///
    /// # Errors
    /// If the opcode does not corresponds to any witness version, errors with
    /// [`Error::UnparsableWitnessVersion`] for the rest of opcodes
    pub fn from_instruction(instruction: Instruction) -> Result<Self, Error> {
        match instruction {
            Instruction::Op(op) => WitnessVersion::from_opcode(op),
            Instruction::PushBytes(bytes) if bytes.len() == 0 => Ok(WitnessVersion::V0),
            Instruction::PushBytes(_) => Err(Error::MalformedWitnessVersion),
        }
    }

    /// Returns integer version number representation for a given [`WitnessVersion`] value.
    /// NB: this is not the same as an integer representation of the opcode signifying witness
    /// version in bitcoin script. Thus, there is no function to directly convert witness version
    /// into a byte since the conversion requires context (bitcoin script or just a version number)
    pub fn into_no(self) -> u8 {
        self as u8
    }
}

impl From<WitnessVersion> for ::bech32::u5 {
    /// Converts [`WitnessVersion`] instance into corresponding Bech32(m) u5-value ([`bech32::u5`])
    fn from(version: WitnessVersion) -> Self {
        ::bech32::u5::try_from_u8(version.into_no()).expect("WitnessVersion must be 0..=16")
    }
}

impl From<WitnessVersion> for opcodes::All {
    /// Converts [`WitnessVersion`] instance into corresponding Bitcoin scriptopcode (`OP_0`..`OP_16`)
    fn from(version: WitnessVersion) -> opcodes::All {
        match version {
            WitnessVersion::V0 => opcodes::all::OP_PUSHBYTES_0,
            no => opcodes::All::from(opcodes::all::OP_PUSHNUM_1.into_u8() + no.into_no() - 1)
        }
    }
}

/// The method used to produce an address
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Payload {
    /// P2PKH address
    PubkeyHash(PubkeyHash),
    /// P2SH address
    ScriptHash(ScriptHash),
    /// Segwit addresses
    WitnessProgram {
        /// The witness program version
        version: WitnessVersion,
        /// The witness program
        program: Vec<u8>,
    },
}

impl Payload {
    /// Get a [Payload] from an output script (scriptPubkey).
    pub fn from_script(script: &script::Script) -> Option<Payload> {
        Some(if script.is_p2pkh() {
            let mut hash_inner = [0u8; 20];
            hash_inner.copy_from_slice(&script.as_bytes()[3..23]);
            Payload::PubkeyHash(PubkeyHash::from_inner(hash_inner))
        } else if script.is_p2sh() {
            let mut hash_inner = [0u8; 20];
            hash_inner.copy_from_slice(&script.as_bytes()[2..22]);
            Payload::ScriptHash(ScriptHash::from_inner(hash_inner))
        } else if script.is_witness_program() {
            Payload::WitnessProgram {
                version: WitnessVersion::from_opcode(opcodes::All::from(script[0])).ok()?,
                program: script[2..].to_vec(),
            }
        } else {
            return None;
        })
    }

    /// Generates a script pubkey spending to this [Payload].
    pub fn script_pubkey(&self) -> script::Script {
        match *self {
            Payload::PubkeyHash(ref hash) =>
                script::Script::new_p2pkh(hash),
            Payload::ScriptHash(ref hash) =>
                script::Script::new_p2sh(hash),
            Payload::WitnessProgram {
                version,
                program: ref prog,
            } => script::Script::new_witness_program(version, prog)
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A Bitcoin address
pub struct Address {
    /// The type of the address
    pub payload: Payload,
    /// The network on which this address is usable
    pub network: Network,
}
serde_string_impl!(Address, "a Bitcoin address");

impl Address {
    /// Creates a pay to (compressed) public key hash address from a public key
    /// This is the preferred non-witness type address
    #[inline]
    pub fn p2pkh(pk: &ecdsa::PublicKey, network: Network) -> Address {
        let mut hash_engine = PubkeyHash::engine();
        pk.write_into(&mut hash_engine).expect("engines don't error");

        Address {
            network: network,
            payload: Payload::PubkeyHash(PubkeyHash::from_engine(hash_engine)),
        }
    }

    /// Creates a pay to script hash P2SH address from a script
    /// This address type was introduced with BIP16 and is the popular type to implement multi-sig these days.
    #[inline]
    pub fn p2sh(script: &script::Script, network: Network) -> Address {
        Address {
            network: network,
            payload: Payload::ScriptHash(ScriptHash::hash(&script[..])),
        }
    }

    /// Create a witness pay to public key address from a public key
    /// This is the native segwit address type for an output redeemable with a single signature
    ///
    /// Will only return an Error when an uncompressed public key is provided.
    pub fn p2wpkh(pk: &ecdsa::PublicKey, network: Network) -> Result<Address, Error> {
        if !pk.compressed {
            return Err(Error::UncompressedPubkey);
        }

        let mut hash_engine = WPubkeyHash::engine();
        pk.write_into(&mut hash_engine).expect("engines don't error");

        Ok(Address {
            network: network,
            payload: Payload::WitnessProgram {
                version: WitnessVersion::V0,
                program: WPubkeyHash::from_engine(hash_engine)[..].to_vec(),
            },
        })
    }

    /// Create a pay to script address that embeds a witness pay to public key
    /// This is a segwit address type that looks familiar (as p2sh) to legacy clients
    ///
    /// Will only return an Error when an uncompressed public key is provided.
    pub fn p2shwpkh(pk: &ecdsa::PublicKey, network: Network) -> Result<Address, Error> {
        if !pk.compressed {
            return Err(Error::UncompressedPubkey);
        }

        let mut hash_engine = WPubkeyHash::engine();
        pk.write_into(&mut hash_engine).expect("engines don't error");

        let builder = script::Builder::new()
            .push_int(0)
            .push_slice(&WPubkeyHash::from_engine(hash_engine)[..]);

        Ok(Address {
            network: network,
            payload: Payload::ScriptHash(ScriptHash::hash(builder.into_script().as_bytes())),
        })
    }

    /// Create a witness pay to script hash address
    pub fn p2wsh(script: &script::Script, network: Network) -> Address {
        Address {
            network: network,
            payload: Payload::WitnessProgram {
                version: WitnessVersion::V0,
                program: WScriptHash::hash(&script[..])[..].to_vec(),
            },
        }
    }

    /// Create a pay to script address that embeds a witness pay to script hash address
    /// This is a segwit address type that looks familiar (as p2sh) to legacy clients
    pub fn p2shwsh(script: &script::Script, network: Network) -> Address {
        let ws = script::Builder::new()
            .push_int(0)
            .push_slice(&WScriptHash::hash(&script[..])[..])
            .into_script();

        Address {
            network: network,
            payload: Payload::ScriptHash(ScriptHash::hash(&ws[..])),
        }
    }

    /// Get the address type of the address.
    /// None if unknown, non-standard or related to the future witness version.
    pub fn address_type(&self) -> Option<AddressType> {
        match self.payload {
            Payload::PubkeyHash(_) => Some(AddressType::P2pkh),
            Payload::ScriptHash(_) => Some(AddressType::P2sh),
            Payload::WitnessProgram {
                version,
                program: ref prog,
            } => {
                // BIP-141 p2wpkh or p2wsh addresses.
                match version {
                    WitnessVersion::V0 => match prog.len() {
                        20 => Some(AddressType::P2wpkh),
                        32 => Some(AddressType::P2wsh),
                        _ => None,
                    },
                    _ => None,
                }
            }
        }
    }

    /// Check whether or not the address is following Bitcoin
    /// standard rules.
    ///
    /// Segwit addresses with unassigned witness versions or non-standard
    /// program sizes are considered non-standard.
    pub fn is_standard(&self) -> bool {
        self.address_type().is_some()
    }

    /// Get an [Address] from an output script (scriptPubkey).
    pub fn from_script(script: &script::Script, network: Network) -> Option<Address> {
        Some(Address {
            payload: Payload::from_script(script)?,
            network: network,
        })
    }

    /// Generates a script pubkey spending to this address
    pub fn script_pubkey(&self) -> script::Script {
        self.payload.script_pubkey()
    }

    /// Creates a URI string *bitcoin:address* optimized to be encoded in QR codes.
    ///
    /// If the address is bech32, both the schema and the address become uppercase.
    /// If the address is base58, the schema is lowercase and the address is left mixed case.
    ///
    /// Quoting BIP 173 "inside QR codes uppercase SHOULD be used, as those permit the use of
    /// alphanumeric mode, which is 45% more compact than the normal byte mode."
    pub fn to_qr_uri(&self) -> String {
        let schema = match self.payload {
            Payload::WitnessProgram { .. } => "BITCOIN",
            _ => "bitcoin",
        };
        format!("{}:{:#}", schema, self)
    }
}

// Alternate formatting `{:#}` is used to return uppercase version of bech32 addresses which should
// be used in QR codes, see [Address::to_qr_uri]
impl fmt::Display for Address {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.payload {
            Payload::PubkeyHash(ref hash) => {
                let mut prefixed = [0; 21];
                prefixed[0] = match self.network {
                    Network::Bitcoin => 0,
                    Network::Testnet | Network::Signet | Network::Regtest => 111,
                };
                prefixed[1..].copy_from_slice(&hash[..]);
                base58::check_encode_slice_to_fmt(fmt, &prefixed[..])
            }
            Payload::ScriptHash(ref hash) => {
                let mut prefixed = [0; 21];
                prefixed[0] = match self.network {
                    Network::Bitcoin => 5,
                    Network::Testnet | Network::Signet | Network::Regtest => 196,
                };
                prefixed[1..].copy_from_slice(&hash[..]);
                base58::check_encode_slice_to_fmt(fmt, &prefixed[..])
            }
            Payload::WitnessProgram {
                version,
                program: ref prog,
            } => {
                let hrp = match self.network {
                    Network::Bitcoin => "bc",
                    Network::Testnet | Network::Signet => "tb",
                    Network::Regtest => "bcrt",
                };
                let mut upper_writer;
                let writer = if fmt.alternate() {
                    upper_writer = UpperWriter(fmt);
                    &mut upper_writer as &mut dyn fmt::Write
                } else {
                    fmt as &mut dyn fmt::Write
                };
                let mut bech32_writer = bech32::Bech32Writer::new(hrp, writer)?;
                bech32::WriteBase32::write_u5(&mut bech32_writer, version.into())?;
                bech32::ToBase32::write_base32(&prog, &mut bech32_writer)
            }
        }
    }
}

struct UpperWriter<W: fmt::Write>(W);

impl<W: fmt::Write> fmt::Write for UpperWriter<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.0.write_char(c.to_ascii_uppercase())?;
        }
        Ok(())
    }
}

/// Extract the bech32 prefix.
/// Returns the same slice when no prefix is found.
fn find_bech32_prefix(bech32: &str) -> &str {
    // Split at the last occurrence of the separator character '1'.
    match bech32.rfind('1') {
        None => bech32,
        Some(sep) => bech32.split_at(sep).0,
    }
}

impl FromStr for Address {
    type Err = Error;

    fn from_str(s: &str) -> Result<Address, Error> {
        // try bech32
        let bech32_network = match find_bech32_prefix(s) {
            // note that upper or lowercase is allowed but NOT mixed case
            "bc" | "BC" => Some(Network::Bitcoin),
            "tb" | "TB" => Some(Network::Testnet), // this may also be signet
            "bcrt" | "BCRT" => Some(Network::Regtest),
            _ => None,
        };
        if let Some(network) = bech32_network {
            // decode as bech32
            let (_, payload) = bech32::decode(s)?;
            if payload.is_empty() {
                return Err(Error::EmptyBech32Payload);
            }

            // Get the script version and program (converted from 5-bit to 8-bit)
            let (version, program): (WitnessVersion, Vec<u8>) = {
                let (v, p5) = payload.split_at(1);
                (WitnessVersion::from_u5(v[0])?, bech32::FromBase32::from_base32(p5)?)
            };

            if program.len() < 2 || program.len() > 40 {
                return Err(Error::InvalidWitnessProgramLength(program.len()));
            }

            // Specific segwit v0 check.
            if version == WitnessVersion::V0 && (program.len() != 20 && program.len() != 32) {
                return Err(Error::InvalidSegwitV0ProgramLength(program.len()));
            }

            return Ok(Address {
                payload: Payload::WitnessProgram {
                    version: version,
                    program: program,
                },
                network: network,
            });
        }

        // Base58
        if s.len() > 50 {
            return Err(Error::Base58(base58::Error::InvalidLength(s.len() * 11 / 15)));
        }
        let data = base58::from_check(s)?;
        if data.len() != 21 {
            return Err(Error::Base58(base58::Error::InvalidLength(data.len())));
        }

        let (network, payload) = match data[0] {
            0 => (
                Network::Bitcoin,
                Payload::PubkeyHash(PubkeyHash::from_slice(&data[1..]).unwrap()),
            ),
            5 => (
                Network::Bitcoin,
                Payload::ScriptHash(ScriptHash::from_slice(&data[1..]).unwrap()),
            ),
            111 => (
                Network::Testnet,
                Payload::PubkeyHash(PubkeyHash::from_slice(&data[1..]).unwrap()),
            ),
            196 => (
                Network::Testnet,
                Payload::ScriptHash(ScriptHash::from_slice(&data[1..]).unwrap()),
            ),
            x => return Err(Error::Base58(base58::Error::InvalidVersion(vec![x]))),
        };

        Ok(Address {
            network: network,
            payload: payload,
        })
    }
}

impl ::std::fmt::Debug for Address {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::string::ToString;

    use hashes::hex::{FromHex, ToHex};

    use blockdata::script::Script;
    use network::constants::Network::{Bitcoin, Testnet};
    use util::ecdsa::PublicKey;

    use super::*;

    macro_rules! hex (($hex:expr) => (Vec::from_hex($hex).unwrap()));
    macro_rules! hex_key (($hex:expr) => (PublicKey::from_slice(&hex!($hex)).unwrap()));
    macro_rules! hex_script (($hex:expr) => (Script::from(hex!($hex))));
    macro_rules! hex_pubkeyhash (($hex:expr) => (PubkeyHash::from_hex(&$hex).unwrap()));
    macro_rules! hex_scripthash (($hex:expr) => (ScriptHash::from_hex($hex).unwrap()));

    fn roundtrips(addr: &Address) {
        assert_eq!(
            Address::from_str(&addr.to_string()).unwrap(),
            *addr,
            "string round-trip failed for {}",
            addr,
        );
        assert_eq!(
            Address::from_script(&addr.script_pubkey(), addr.network).as_ref(),
            Some(addr),
            "script round-trip failed for {}",
            addr,
        );
        //TODO: add serde roundtrip after no-strason PR
    }

    #[test]
    fn test_p2pkh_address_58() {
        let addr = Address {
            network: Bitcoin,
            payload: Payload::PubkeyHash(hex_pubkeyhash!("162c5ea71c0b23f5b9022ef047c4a86470a5b070")),
        };

        assert_eq!(
            addr.script_pubkey(),
            hex_script!("76a914162c5ea71c0b23f5b9022ef047c4a86470a5b07088ac")
        );
        assert_eq!(&addr.to_string(), "132F25rTsvBdp9JzLLBHP5mvGY66i1xdiM");
        assert_eq!(addr.address_type(), Some(AddressType::P2pkh));
        roundtrips(&addr);
    }

    #[test]
    fn test_p2pkh_from_key() {
        let key = hex_key!("048d5141948c1702e8c95f438815794b87f706a8d4cd2bffad1dc1570971032c9b6042a0431ded2478b5c9cf2d81c124a5e57347a3c63ef0e7716cf54d613ba183");
        let addr = Address::p2pkh(&key, Bitcoin);
        assert_eq!(&addr.to_string(), "1QJVDzdqb1VpbDK7uDeyVXy9mR27CJiyhY");

        let key = hex_key!(&"03df154ebfcf29d29cc10d5c2565018bce2d9edbab267c31d2caf44a63056cf99f");
        let addr = Address::p2pkh(&key, Testnet);
        assert_eq!(&addr.to_string(), "mqkhEMH6NCeYjFybv7pvFC22MFeaNT9AQC");
        assert_eq!(addr.address_type(), Some(AddressType::P2pkh));
        roundtrips(&addr);
    }

    #[test]
    fn test_p2sh_address_58() {
        let addr = Address {
            network: Bitcoin,
            payload: Payload::ScriptHash(hex_scripthash!("162c5ea71c0b23f5b9022ef047c4a86470a5b070")),
        };

        assert_eq!(
            addr.script_pubkey(),
            hex_script!("a914162c5ea71c0b23f5b9022ef047c4a86470a5b07087")
        );
        assert_eq!(&addr.to_string(), "33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k");
        assert_eq!(addr.address_type(), Some(AddressType::P2sh));
        roundtrips(&addr);
    }

    #[test]
    fn test_p2sh_parse() {
        let script = hex_script!("552103a765fc35b3f210b95223846b36ef62a4e53e34e2925270c2c7906b92c9f718eb2103c327511374246759ec8d0b89fa6c6b23b33e11f92c5bc155409d86de0c79180121038cae7406af1f12f4786d820a1466eec7bc5785a1b5e4a387eca6d797753ef6db2103252bfb9dcaab0cd00353f2ac328954d791270203d66c2be8b430f115f451b8a12103e79412d42372c55dd336f2eb6eb639ef9d74a22041ba79382c74da2338fe58ad21035049459a4ebc00e876a9eef02e72a3e70202d3d1f591fc0dd542f93f642021f82102016f682920d9723c61b27f562eb530c926c00106004798b6471e8c52c60ee02057ae");
        let addr = Address::p2sh(&script, Testnet);

        assert_eq!(&addr.to_string(), "2N3zXjbwdTcPsJiy8sUK9FhWJhqQCxA8Jjr");
        assert_eq!(addr.address_type(), Some(AddressType::P2sh));
        roundtrips(&addr);
    }

    #[test]
    fn test_p2wpkh() {
        // stolen from Bitcoin transaction: b3c8c2b6cfc335abbcb2c7823a8453f55d64b2b5125a9a61e8737230cdb8ce20
        let mut key = hex_key!("033bc8c83c52df5712229a2f72206d90192366c36428cb0c12b6af98324d97bfbc");
        let addr = Address::p2wpkh(&key, Bitcoin).unwrap();
        assert_eq!(&addr.to_string(), "bc1qvzvkjn4q3nszqxrv3nraga2r822xjty3ykvkuw");
        assert_eq!(addr.address_type(), Some(AddressType::P2wpkh));
        roundtrips(&addr);

        // Test uncompressed pubkey
        key.compressed = false;
        assert_eq!(Address::p2wpkh(&key, Bitcoin), Err(Error::UncompressedPubkey));
    }

    #[test]
    fn test_p2wsh() {
        // stolen from Bitcoin transaction 5df912fda4becb1c29e928bec8d64d93e9ba8efa9b5b405bd683c86fd2c65667
        let script = hex_script!("52210375e00eb72e29da82b89367947f29ef34afb75e8654f6ea368e0acdfd92976b7c2103a1b26313f430c4b15bb1fdce663207659d8cac749a0e53d70eff01874496feff2103c96d495bfdd5ba4145e3e046fee45e84a8a48ad05bd8dbb395c011a32cf9f88053ae");
        let addr = Address::p2wsh(&script, Bitcoin);
        assert_eq!(
            &addr.to_string(),
            "bc1qwqdg6squsna38e46795at95yu9atm8azzmyvckulcc7kytlcckxswvvzej"
        );
        assert_eq!(addr.address_type(), Some(AddressType::P2wsh));
        roundtrips(&addr);
    }

    #[test]
    fn test_p2shwpkh() {
        // stolen from Bitcoin transaction: ad3fd9c6b52e752ba21425435ff3dd361d6ac271531fc1d2144843a9f550ad01
        let mut key = hex_key!("026c468be64d22761c30cd2f12cbc7de255d592d7904b1bab07236897cc4c2e766");
        let addr = Address::p2shwpkh(&key, Bitcoin).unwrap();
        assert_eq!(&addr.to_string(), "3QBRmWNqqBGme9er7fMkGqtZtp4gjMFxhE");
        assert_eq!(addr.address_type(), Some(AddressType::P2sh));
        roundtrips(&addr);

        // Test uncompressed pubkey
        key.compressed = false;
        assert_eq!(Address::p2wpkh(&key, Bitcoin), Err(Error::UncompressedPubkey));
    }

    #[test]
    fn test_p2shwsh() {
        // stolen from Bitcoin transaction f9ee2be4df05041d0e0a35d7caa3157495ca4f93b233234c9967b6901dacf7a9
        let script = hex_script!("522103e5529d8eaa3d559903adb2e881eb06c86ac2574ffa503c45f4e942e2a693b33e2102e5f10fcdcdbab211e0af6a481f5532536ec61a5fdbf7183770cf8680fe729d8152ae");
        let addr = Address::p2shwsh(&script, Bitcoin);
        assert_eq!(&addr.to_string(), "36EqgNnsWW94SreZgBWc1ANC6wpFZwirHr");
        assert_eq!(addr.address_type(), Some(AddressType::P2sh));
        roundtrips(&addr);
    }

    #[test]
    fn test_non_existent_segwit_version() {
        // 40-byte program
        let program = hex!(
            "654f6ea368e0acdfd92976b7c2103a1b26313f430654f6ea368e0acdfd92976b7c2103a1b26313f4"
        );
        let addr = Address {
            payload: Payload::WitnessProgram {
                version: WitnessVersion::V13,
                program: program,
            },
            network: Network::Bitcoin,
        };
        roundtrips(&addr);
    }

    #[test]
    fn test_bip173_vectors() {
        let valid_vectors = [
            ("BC1QW508D6QEJXTDG4Y5R3ZARVARY0C5XW7KV8F3T4", "0014751e76e8199196d454941c45d1b3a323f1433bd6"),
            ("tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3q0sl5k7", "00201863143c14c5166804bd19203356da136c985678cd4d27a1b8c6329604903262"),
            ("bc1pw508d6qejxtdg4y5r3zarvary0c5xw7kw508d6qejxtdg4y5r3zarvary0c5xw7k7grplx", "5128751e76e8199196d454941c45d1b3a323f1433bd6751e76e8199196d454941c45d1b3a323f1433bd6"),
            ("BC1SW50QA3JX3S", "6002751e"),
            ("bc1zw508d6qejxtdg4y5r3zarvaryvg6kdaj", "5210751e76e8199196d454941c45d1b3a323"),
            ("tb1qqqqqp399et2xygdj5xreqhjjvcmzhxw4aywxecjdzew6hylgvsesrxh6hy", "0020000000c4a5cad46221b2a187905e5266362b99d5e91c6ce24d165dab93e86433"),
        ];
        for vector in &valid_vectors {
            let addr: Address = vector.0.parse().unwrap();
            assert_eq!(&addr.script_pubkey().as_bytes().to_hex(), vector.1);
            roundtrips(&addr);
        }

        let invalid_vectors = [
            "tc1qw508d6qejxtdg4y5r3zarvary0c5xw7kg3g4ty",
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t5",
            "BC13W508D6QEJXTDG4Y5R3ZARVARY0C5XW7KN40WF2",
            "bc1rw5uspcuh",
            "bc10w508d6qejxtdg4y5r3zarvary0c5xw7kw508d6qejxtdg4y5r3zarvary0c5xw7kw5rljs90",
            "BC1QR508D6QEJXTDG4Y5R3ZARVARYV98GJ9P",
            "tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3q0sL5k7",
            "bc1zw508d6qejxtdg4y5r3zarvaryvqyzf3du",
            "tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3pjxtptv",
            "bc1gmk9yu",
        ];
        for vector in &invalid_vectors {
            assert!(vector.parse::<Address>().is_err());
        }
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_json_serialize() {
        use serde_json;

        let addr = Address::from_str("132F25rTsvBdp9JzLLBHP5mvGY66i1xdiM").unwrap();
        let json = serde_json::to_value(&addr).unwrap();
        assert_eq!(
            json,
            serde_json::Value::String("132F25rTsvBdp9JzLLBHP5mvGY66i1xdiM".to_owned())
        );
        let into: Address = serde_json::from_value(json).unwrap();
        assert_eq!(addr.to_string(), into.to_string());
        assert_eq!(
            into.script_pubkey(),
            hex_script!("76a914162c5ea71c0b23f5b9022ef047c4a86470a5b07088ac")
        );

        let addr = Address::from_str("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k").unwrap();
        let json = serde_json::to_value(&addr).unwrap();
        assert_eq!(
            json,
            serde_json::Value::String("33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k".to_owned())
        );
        let into: Address = serde_json::from_value(json).unwrap();
        assert_eq!(addr.to_string(), into.to_string());
        assert_eq!(
            into.script_pubkey(),
            hex_script!("a914162c5ea71c0b23f5b9022ef047c4a86470a5b07087")
        );

        let addr =
            Address::from_str("tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3q0sl5k7")
                .unwrap();
        let json = serde_json::to_value(&addr).unwrap();
        assert_eq!(
            json,
            serde_json::Value::String(
                "tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3q0sl5k7".to_owned()
            )
        );
        let into: Address = serde_json::from_value(json).unwrap();
        assert_eq!(addr.to_string(), into.to_string());
        assert_eq!(
            into.script_pubkey(),
            hex_script!("00201863143c14c5166804bd19203356da136c985678cd4d27a1b8c6329604903262")
        );

        let addr = Address::from_str("bcrt1q2nfxmhd4n3c8834pj72xagvyr9gl57n5r94fsl").unwrap();
        let json = serde_json::to_value(&addr).unwrap();
        assert_eq!(
            json,
            serde_json::Value::String("bcrt1q2nfxmhd4n3c8834pj72xagvyr9gl57n5r94fsl".to_owned())
        );
        let into: Address = serde_json::from_value(json).unwrap();
        assert_eq!(addr.to_string(), into.to_string());
        assert_eq!(
            into.script_pubkey(),
            hex_script!("001454d26dddb59c7073c6a197946ea1841951fa7a74")
        );
    }

    #[test]
    fn test_qr_string() {
        for el in  ["132F25rTsvBdp9JzLLBHP5mvGY66i1xdiM", "33iFwdLuRpW1uK1RTRqsoi8rR4NpDzk66k"].iter() {
            let addr = Address::from_str(el).unwrap();
            assert_eq!(addr.to_qr_uri(), format!("bitcoin:{}", el));
        }

        for el in ["bcrt1q2nfxmhd4n3c8834pj72xagvyr9gl57n5r94fsl", "bc1qwqdg6squsna38e46795at95yu9atm8azzmyvckulcc7kytlcckxswvvzej"].iter() {
            let addr = Address::from_str(el).unwrap();
            assert_eq!(addr.to_qr_uri(), format!("BITCOIN:{}", el.to_ascii_uppercase()) );
        }
    }

}
