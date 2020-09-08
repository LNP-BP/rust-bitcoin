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

use blockdata::script::Script;
use blockdata::transaction::{SigHashType, Transaction, TxOut};
use consensus::encode;
use util::bip32::{DerivationPath, Fingerprint};
use util::key::PublicKey;
use util::psbt;
use util::psbt::map::Map;
use util::psbt::raw;
use util::psbt::Error;

/// Type: Non-Witness UTXO PSBT_IN_NON_WITNESS_UTXO = 0x00
const PSBT_IN_NON_WITNESS_UTXO: u8 = 0x00;
/// Type: Witness UTXO PSBT_IN_WITNESS_UTXO = 0x01
const PSBT_IN_WITNESS_UTXO: u8 = 0x01;
/// Type: Partial Signature PSBT_IN_PARTIAL_SIG = 0x02
const PSBT_IN_PARTIAL_SIG: u8 = 0x02;
/// Type: Sighash Type PSBT_IN_SIGHASH_TYPE = 0x03
const PSBT_IN_SIGHASH_TYPE: u8 = 0x03;
/// Type: Redeem Script PSBT_IN_REDEEM_SCRIPT = 0x04
const PSBT_IN_REDEEM_SCRIPT: u8 = 0x04;
/// Type: Witness Script PSBT_IN_WITNESS_SCRIPT = 0x05
const PSBT_IN_WITNESS_SCRIPT: u8 = 0x05;
/// Type: BIP 32 Derivation Path PSBT_IN_BIP32_DERIVATION = 0x06
const PSBT_IN_BIP32_DERIVATION: u8 = 0x06;
/// Type: Finalized scriptSig PSBT_IN_FINAL_SCRIPTSIG = 0x07
const PSBT_IN_FINAL_SCRIPTSIG: u8 = 0x07;
/// Type: Finalized scriptWitness PSBT_IN_FINAL_SCRIPTWITNESS = 0x08
const PSBT_IN_FINAL_SCRIPTWITNESS: u8 = 0x08;
/// Type: Proprietary Use Type PSBT_IN_PROPRIETARY = 0xFC
const PSBT_IN_PROPRIETARY: u8 = 0xFC;

/// A key-value map for an input of the corresponding index in the unsigned
/// transaction.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Input {
    /// The non-witness transaction this input spends from. Should only be
    /// [std::option::Option::Some] for inputs which spend non-segwit outputs or
    /// if it is unknown whether an input spends a segwit output.
    pub non_witness_utxo: Option<Transaction>,
    /// The transaction output this input spends from. Should only be
    /// [std::option::Option::Some] for inputs which spend segwit outputs,
    /// including P2SH embedded ones.
    pub witness_utxo: Option<TxOut>,
    /// A map from public keys to their corresponding signature as would be
    /// pushed to the stack from a scriptSig or witness.
    pub partial_sigs: BTreeMap<PublicKey, Vec<u8>>,
    /// The sighash type to be used for this input. Signatures for this input
    /// must use the sighash type.
    pub sighash_type: Option<SigHashType>,
    /// The redeem script for this input.
    pub redeem_script: Option<Script>,
    /// The witness script for this input.
    pub witness_script: Option<Script>,
    /// A map from public keys needed to sign this input to their corresponding
    /// master key fingerprints and derivation paths.
    pub bip32_derivation: BTreeMap<PublicKey, (Fingerprint, DerivationPath)>,
    /// The finalized, fully-constructed scriptSig with signatures and any other
    /// scripts necessary for this input to pass validation.
    pub final_script_sig: Option<Script>,
    /// The finalized, fully-constructed scriptWitness with signatures and any
    /// other scripts necessary for this input to pass validation.
    pub final_script_witness: Option<Vec<Vec<u8>>>,
    /// Proprietary key-value pairs for this input.
    pub proprietary: BTreeMap<raw::Key, Vec<u8>>,
    /// Unknown key-value pairs for this input.
    pub unknown: BTreeMap<raw::Key, Vec<u8>>,
}

impl Map for Input {
    fn insert_pair(&mut self, pair: raw::Pair) -> Result<(), encode::Error> {
        let raw::Pair {
            key: raw_key,
            value: raw_value,
        } = pair;

        match raw_key.type_value {
            PSBT_IN_NON_WITNESS_UTXO => {
                impl_psbt_insert_pair! {
                    self.non_witness_utxo <= <raw_key: _>|<raw_value: Transaction>
                }
            }
            PSBT_IN_WITNESS_UTXO => {
                impl_psbt_insert_pair! {
                    self.witness_utxo <= <raw_key: _>|<raw_value: TxOut>
                }
            }
            PSBT_IN_PARTIAL_SIG => {
                impl_psbt_insert_pair! {
                    self.partial_sigs <= <raw_key: PublicKey>|<raw_value: Vec<u8>>
                }
            }
            PSBT_IN_SIGHASH_TYPE => {
                impl_psbt_insert_pair! {
                    self.sighash_type <= <raw_key: _>|<raw_value: SigHashType>
                }
            }
            PSBT_IN_REDEEM_SCRIPT => {
                impl_psbt_insert_pair! {
                    self.redeem_script <= <raw_key: _>|<raw_value: Script>
                }
            }
            PSBT_IN_WITNESS_SCRIPT => {
                impl_psbt_insert_pair! {
                    self.witness_script <= <raw_key: _>|<raw_value: Script>
                }
            }
            PSBT_IN_BIP32_DERIVATION => {
                impl_psbt_insert_pair! {
                    self.bip32_derivation <= <raw_key: PublicKey>|<raw_value: (Fingerprint, DerivationPath)>
                }
            }
            PSBT_IN_FINAL_SCRIPTSIG => {
                impl_psbt_insert_pair! {
                    self.final_script_sig <= <raw_key: _>|<raw_value: Script>
                }
            }
            PSBT_IN_FINAL_SCRIPTWITNESS => {
                impl_psbt_insert_pair! {
                    self.final_script_witness <= <raw_key: _>|<raw_value: Vec<Vec<u8>>>
                }
            }
            PSBT_IN_PROPRIETARY => match self.proprietary.entry(raw_key) {
                ::std::collections::btree_map::Entry::Vacant(empty_key) => {empty_key.insert(raw_value);},
                ::std::collections::btree_map::Entry::Occupied(k) => return Err(Error::DuplicateKey(k.key().clone()).into()),
            }
            _ => match self.unknown.entry(raw_key) {
                ::std::collections::btree_map::Entry::Vacant(empty_key) => {empty_key.insert(raw_value);},
                ::std::collections::btree_map::Entry::Occupied(k) => return Err(Error::DuplicateKey(k.key().clone()).into()),
            }
        }

        Ok(())
    }

    fn get_pairs(&self) -> Result<Vec<raw::Pair>, encode::Error> {
        let mut rv: Vec<raw::Pair> = Default::default();

        impl_psbt_get_pair! {
            rv.push(self.non_witness_utxo as <PSBT_IN_NON_WITNESS_UTXO, _>|<Transaction>)
        }

        impl_psbt_get_pair! {
            rv.push(self.witness_utxo as <PSBT_IN_WITNESS_UTXO, _>|<TxOut>)
        }

        impl_psbt_get_pair! {
            rv.push(self.partial_sigs as <PSBT_IN_PARTIAL_SIG, PublicKey>|<Vec<u8>>)
        }

        impl_psbt_get_pair! {
            rv.push(self.sighash_type as <PSBT_IN_SIGHASH_TYPE, _>|<SigHashType>)
        }

        impl_psbt_get_pair! {
            rv.push(self.redeem_script as <PSBT_IN_REDEEM_SCRIPT, _>|<Script>)
        }

        impl_psbt_get_pair! {
            rv.push(self.witness_script as <PSBT_IN_WITNESS_SCRIPT, _>|<Script>)
        }

        impl_psbt_get_pair! {
            rv.push(self.bip32_derivation as <PSBT_IN_BIP32_DERIVATION, PublicKey>|<(Fingerprint, DerivationPath)>)
        }

        impl_psbt_get_pair! {
            rv.push(self.final_script_sig as <PSBT_IN_FINAL_SCRIPTSIG, _>|<Script>)
        }

        impl_psbt_get_pair! {
            rv.push(self.final_script_witness as <PSBT_IN_FINAL_SCRIPTWITNESS, _>|<Script>)
        }

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
        merge!(non_witness_utxo, self, other);

        if let (&None, Some(witness_utxo)) = (&self.witness_utxo, other.witness_utxo) {
            self.witness_utxo = Some(witness_utxo);
            self.non_witness_utxo = None; // Clear out any non-witness UTXO when we set a witness one
        }

        self.partial_sigs.extend(other.partial_sigs);
        self.bip32_derivation.extend(other.bip32_derivation);
        self.proprietary.extend(other.proprietary);
        self.unknown.extend(other.unknown);

        merge!(redeem_script, self, other);
        merge!(witness_script, self, other);
        merge!(final_script_sig, self, other);
        merge!(final_script_witness, self, other);

        Ok(())
    }
}

impl_psbtmap_consensus_enc_dec_oding!(Input);
