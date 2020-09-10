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

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::io::{self, Cursor};

use blockdata::transaction::Transaction;
use consensus::{encode, Encodable, Decodable};
use util::psbt::map::Map;
use util::psbt::raw;
use util::psbt;
use util::psbt::Error;
use util::bip32::{Fingerprint, DerivationPath, ExtendedPubKey};

/// Type: Unsigned Transaction PSBT_GLOBAL_UNSIGNED_TX = 0x00
const PSBT_GLOBAL_UNSIGNED_TX: u8 = 0x00;
/// Type: Extended Public Key PSBT_GLOBAL_XPUB = 0x01
const PSBT_GLOBAL_XPUB: u8 = 0x01;
/// Type: Version Number PSBT_GLOBAL_VERSION = 0xFB
const PSBT_GLOBAL_VERSION: u8 = 0xFB;
/// Type: Proprietary Use Type PSBT_GLOBAL_PROPRIETARY = 0xFC
const PSBT_GLOBAL_PROPRIETARY: u8 = 0xFC;

/// A key-value map for global data.
#[derive(Clone, Debug, PartialEq)]
pub struct Global {
    /// The unsigned transaction, scriptSigs and witnesses for each input must be
    /// empty.
    pub unsigned_tx: Transaction,
    /// A global map frpm extended public keys to the used key fingerprint and
    /// derivation path as defined by BIP 32
    pub xpub: BTreeMap<ExtendedPubKey, (Fingerprint, DerivationPath)>,
    /// The version number of this PSBT. If ommitted, the version number is 0.
    pub version: u32,
    /// Global proprietary key-value pairs.
    pub proprietary: BTreeMap<raw::Key, Vec<u8>>,
    /// Unknown global key-value pairs.
    pub unknown: BTreeMap<raw::Key, Vec<u8>>,
}

impl Global {
    /// Create a Global from an unsigned transaction, error if not unsigned
    pub fn from_unsigned_tx(tx: Transaction) -> Result<Self, psbt::Error> {
        for txin in &tx.input {
            if !txin.script_sig.is_empty() {
                return Err(Error::UnsignedTxHasScriptSigs);
            }

            if !txin.witness.is_empty() {
                return Err(Error::UnsignedTxHasScriptWitnesses);
            }
        }

        Ok(Global {
            unsigned_tx: tx,
            xpub: Default::default(),
            version: 0,
            proprietary: Default::default(),
            unknown: Default::default(),
        })
    }
}

impl Map for Global {
    fn insert_pair(&mut self, pair: raw::Pair) -> Result<(), encode::Error> {
        let raw::Pair {
            key: raw_key,
            value: raw_value,
        } = pair;

        let mut version = None;
        match raw_key.type_value {
            PSBT_GLOBAL_UNSIGNED_TX => return Err(Error::DuplicateKey(raw_key).into()),
            PSBT_GLOBAL_XPUB => {
                impl_psbt_insert_pair! {
                    self.xpub <= <raw_key: ExtendedPubKey>|<raw_value: (Fingerprint, DerivationPath)>
                }
            }
            PSBT_GLOBAL_VERSION => {
                impl_psbt_insert_pair! {
                    version <= <raw_key: _>|<raw_value: u32>
                }
            },
            PSBT_GLOBAL_PROPRIETARY => match self.proprietary.entry(raw_key) {
                Entry::Vacant(empty_key) => {empty_key.insert(raw_value);},
                Entry::Occupied(k) => return Err(Error::DuplicateKey(k.key().clone()).into()),
            }
            _ => match self.unknown.entry(raw_key) {
                Entry::Vacant(empty_key) => {empty_key.insert(raw_value);},
                Entry::Occupied(k) => return Err(Error::DuplicateKey(k.key().clone()).into()),
            }
        }
        if let Some(ver) = version {
            self.version = ver;
        }

        Ok(())
    }

    fn get_pairs(&self) -> Result<Vec<raw::Pair>, encode::Error> {
        let mut rv: Vec<raw::Pair> = Default::default();

        rv.push(raw::Pair {
            key: raw::Key {
                type_value: PSBT_GLOBAL_UNSIGNED_TX,
                key: vec![],
            },
            value: {
                // Manually serialized to ensure 0-input txs are serialized
                // without witnesses.
                let mut ret = Vec::new();
                self.unsigned_tx.version.consensus_encode(&mut ret)?;
                self.unsigned_tx.input.consensus_encode(&mut ret)?;
                self.unsigned_tx.output.consensus_encode(&mut ret)?;
                self.unsigned_tx.lock_time.consensus_encode(&mut ret)?;
                ret
            },
        });

        for (key, value) in self.proprietary.iter() {
            rv.push(raw::Pair {
                key: key.clone(),
                value: value.clone(),
            });
        }

        for (key, value) in self.unknown.iter() {
            rv.push(raw::Pair {
                key: key.clone(),
                value: value.clone(),
            });
        }

        Ok(rv)
    }

    fn merge(&mut self, other: Self) -> Result<(), psbt::Error> {
        if self.unsigned_tx != other.unsigned_tx {
            return Err(psbt::Error::UnexpectedUnsignedTx {
                expected: self.unsigned_tx.clone(),
                actual: other.unsigned_tx,
            });
        }

        self.proprietary.extend(other.proprietary);
        self.unknown.extend(other.unknown);
        Ok(())
    }
}

impl_psbtmap_consensus_encoding!(Global);

impl Decodable for Global {
    fn consensus_decode<D: io::Read>(mut d: D) -> Result<Self, encode::Error> {

        let mut tx: Option<Transaction> = None;
        let mut unknowns: BTreeMap<raw::Key, Vec<u8>> = Default::default();
        let mut proprietary: BTreeMap<raw::Key, Vec<u8>> = Default::default();

        loop {
            match raw::Pair::consensus_decode(&mut d) {
                Ok(pair) => {
                    match pair.key.type_value {
                        PSBT_GLOBAL_UNSIGNED_TX => {
                            // key has to be empty
                            if pair.key.key.is_empty() {
                                // there can only be one unsigned transaction
                                if tx.is_none() {
                                    let vlen: usize = pair.value.len();
                                    let mut decoder = Cursor::new(pair.value);

                                    // Manually deserialized to ensure 0-input
                                    // txs without witnesses are deserialized
                                    // properly.
                                    tx = Some(Transaction {
                                        version: Decodable::consensus_decode(&mut decoder)?,
                                        input: Decodable::consensus_decode(&mut decoder)?,
                                        output: Decodable::consensus_decode(&mut decoder)?,
                                        lock_time: Decodable::consensus_decode(&mut decoder)?,
                                    });

                                    if decoder.position() != vlen as u64 {
                                        return Err(encode::Error::ParseFailed("data not consumed entirely when explicitly deserializing"))
                                    }
                                } else {
                                    return Err(Error::DuplicateKey(pair.key).into())
                                }
                            } else {
                                return Err(Error::InvalidKey(pair.key).into())
                            }
                        }
                        PSBT_GLOBAL_PROPRIETARY => match proprietary.entry(pair.key) {
                            Entry::Vacant(empty_key) => {empty_key.insert(pair.value);},
                            Entry::Occupied(k) => return Err(Error::DuplicateKey(k.key().clone()).into()),
                        }
                        _ => match unknowns.entry(pair.key) {
                            Entry::Vacant(empty_key) => {empty_key.insert(pair.value);},
                            Entry::Occupied(k) => return Err(Error::DuplicateKey(k.key().clone()).into()),
                        }
                    }
                }
                Err(::consensus::encode::Error::Psbt(::util::psbt::Error::NoMorePairs)) => break,
                Err(e) => return Err(e),
            }
        }

        if let Some(tx) = tx {
            let mut rv: Global = Global::from_unsigned_tx(tx)?;
            rv.unknown = unknowns;
            Ok(rv)
        } else {
            Err(Error::MustHaveUnsignedTx.into())
        }
    }
}
