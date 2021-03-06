// Rust Bitcoin Library
// Written by
//   The Rust Bitcoin developers
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

use std::error;
use std::fmt;

use blockdata::transaction::Transaction;
use consensus::encode;
use util::psbt::raw;

use hashes::{self, sha256, hash160, sha256d, ripemd160};
use hash_types::Txid;

/// Support hash-preimages in psbt
#[derive(Debug)]
pub enum PsbtHash{
    Ripemd160(ripemd160::Hash),
    Sha256(sha256::Hash),
    Hash256(sha256d::Hash),
    Hash160(hash160::Hash),
}
/// Ways that a Partially Signed Transaction might fail.
#[derive(Debug)]
pub enum Error {
    /// Magic bytes for a PSBT must be the ASCII for "psbt" serialized in most
    /// significant byte order.
    InvalidMagic,
    /// The separator for a PSBT must be `0xff`.
    InvalidSeparator,
    /// Known keys must be according to spec.
    InvalidKey(raw::Key),
    /// Non-proprietary key type found when proprietary key was expected
    InvalidProprietaryKey,
    /// Keys within key-value map should never be duplicated.
    DuplicateKey(raw::Key),
    /// The scriptSigs for the unsigned transaction must be empty.
    UnsignedTxHasScriptSigs,
    /// The scriptWitnesses for the unsigned transaction must be empty.
    UnsignedTxHasScriptWitnesses,
    /// A PSBT must have an unsigned transaction.
    MustHaveUnsignedTx,
    /// Signals that there are no more key-value pairs in a key-value map.
    NoMorePairs,
    /// Attempting to merge with a PSBT describing a different unsigned
    /// transaction.
    UnexpectedUnsignedTx {
        /// Expected
        expected: Transaction,
        /// Actual
        actual: Transaction,
    },
    /// Unable to parse as a standard SigHash type.
    NonStandardSigHashType(u32),
    /// Parsing errors from bitcoin_hashes
    HashParseError(hashes::Error),
    /// The pre-image must hash to the correponding psbt hash
    InvalidPreimageHashPair{
        /// Pre-image
        preimage: Vec<u8>,
        /// Hash value
        hash: PsbtHash,
    },
    /// If NonWitnessUtxo is used, the nonWitnessUtxo txid must
    /// be the same of prevout txid
    InvalidNonWitnessUtxo{
        /// Pre-image
        prevout_txid: Txid,
        /// Hash value
        non_witness_utxo_txid: Txid,
    },
    /// Incorrect P2sh/p2wsh script hash for the witness/redeem
    /// script
    InvalidWitnessScript{
        /// Expected Witness/Redeem Script Hash
        // returns a vec to unify the p2wsh(sha2) and p2sh(hash160)
        expected: Vec<u8>,
        /// Actual Witness script Hash
        actual: Vec<u8>,
    },
    /// Currently only p2wpkh and p2wsh scripts are possible in segwit
    UnrecognizedWitnessProgram,
    /// The psbt input must either have an associated nonWitnessUtxo or
    /// a WitnessUtxo
    MustHaveSpendingUtxo,
    /// Serialization error in bitcoin consensus-encoded structures
    ConsensusEncoding,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidKey(ref rkey) => write!(f, "invalid key: {}", rkey),
            Error::InvalidProprietaryKey => write!(f, "non-proprietary key type found when proprietary key was expected"),
            Error::DuplicateKey(ref rkey) => write!(f, "duplicate key: {}", rkey),
            Error::UnexpectedUnsignedTx { expected: ref e, actual: ref a } => write!(f, "different unsigned transaction: expected {}, actual {}", e.txid(), a.txid()),
            Error::NonStandardSigHashType(ref sht) => write!(f, "non-standard sighash type: {}", sht),
            Error::InvalidMagic => f.write_str("invalid magic"),
            Error::InvalidSeparator => f.write_str("invalid separator"),
            Error::UnsignedTxHasScriptSigs => f.write_str("the unsigned transaction has script sigs"),
            Error::UnsignedTxHasScriptWitnesses => f.write_str("the unsigned transaction has script witnesses"),
            Error::MustHaveUnsignedTx => {
                f.write_str("partially signed transactions must have an unsigned transaction")
            }
            Error::NoMorePairs => f.write_str("no more key-value pairs for this psbt map"),
            Error::HashParseError(e) => write!(f, "Hash Parse Error: {}", e),
            Error::InvalidPreimageHashPair{ref preimage, ref hash} => {
                // directly using debug forms of psbthash enums
                write!(f, "Preimage {:?} does not match hash {:?}", preimage, hash )
            },
            Error::InvalidNonWitnessUtxo{ref prevout_txid, ref non_witness_utxo_txid} => {
                write!(f, "NonWitnessUtxo txid {} must be the same as prevout txid {}", non_witness_utxo_txid, prevout_txid)
            },
            Error::InvalidWitnessScript{ref expected, ref actual} => {
                write!(f, "Invalid Witness/Redeem script: Expected {:?}, got {:?}", expected, actual)
            }
            Error::UnrecognizedWitnessProgram => {
                f.write_str("Witness program must be p2wpkh/p2wsh")
            }
            Error::MustHaveSpendingUtxo => {
                f.write_str("Input must either WitnessUtxo/ NonWitnessUtxo")
            }
            Error::ConsensusEncoding => f.write_str("bitcoin consensus encoding error"),
        }
    }
}

#[allow(deprecated)]
impl error::Error for Error {
    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }
}

#[doc(hidden)]
impl From<hashes::Error> for Error {
    fn from(e: hashes::Error) -> Error {
        Error::HashParseError(e)
    }
}

impl From<encode::Error> for Error {
    fn from(err: encode::Error) -> Self {
        match err {
            encode::Error::Psbt(err) => err,
            _ => Error::ConsensusEncoding,
        }
    }
}
