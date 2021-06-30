// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the snarkOS library.

// The snarkOS library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkOS library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkOS library. If not, see <https://www.gnu.org/licenses/>.

use crate::*;
use snarkvm_algorithms::traits::LoadableMerkleParameters;
use snarkvm_dpc::{
    Block,
    BlockError,
    BlockHeaderHash,
    DatabaseTransaction,
    Op,
    Storage,
    StorageError,
    TransactionScheme,
    Transactions as DPCTransactions,
};
use snarkvm_utilities::{to_bytes, FromBytes, ToBytes};

use std::sync::atomic::Ordering;

impl<T: TransactionScheme, P: LoadableMerkleParameters, S: Storage> Ledger<T, P, S> {

    /// De-commit the latest block and return its header hash.
    pub fn decommit_latest_block(&self) -> Result<BlockHeaderHash, StorageError> {
        let current_block_height = self.get_current_block_height();

        tracing::debug!("Decommitting block at height {}", current_block_height);

        if current_block_height == 0 {
            return Err(StorageError::InvalidBlockDecommit);
        }

        let new_best_block_number = current_block_height - 1;
        let block_hash: BlockHeaderHash = self.get_block_hash(current_block_height)?;

        let mut database_transaction = DatabaseTransaction::new();

        database_transaction.push(Op::Insert {
            col: COL_META,
            key: KEY_BEST_BLOCK_NUMBER.as_bytes().to_vec(),
            value: new_best_block_number.to_le_bytes().to_vec(),
        });

        database_transaction.push(Op::Delete {
            col: COL_DIGEST,
            key: self.current_digest()?,
        });

        let mut sn_index = self.current_sn_index()?;
        let mut cm_index = self.current_cm_index()?;
        let mut memo_index = self.current_memo_index()?;

        for transaction in self.get_block_transactions(&block_hash)?.0 {
            for sn in transaction.old_serial_numbers() {
                database_transaction.push(Op::Delete {
                    col: COL_SERIAL_NUMBER,
                    key: to_bytes![sn]?.to_vec(),
                });
                sn_index -= 1;
            }

            for cm in transaction.new_commitments() {
                database_transaction.push(Op::Delete {
                    col: COL_COMMITMENT,
                    key: to_bytes![cm]?.to_vec(),
                });
                cm_index -= 1;
            }

            database_transaction.push(Op::Delete {
                col: COL_MEMO,
                key: to_bytes![transaction.memorandum()]?.to_vec(),
            });
            memo_index -= 1;
        }

        // Update the database state for current indexes

        database_transaction.push(Op::Insert {
            col: COL_META,
            key: KEY_CURR_SN_INDEX.as_bytes().to_vec(),
            value: (sn_index as u32).to_le_bytes().to_vec(),
        });
        database_transaction.push(Op::Insert {
            col: COL_META,
            key: KEY_CURR_CM_INDEX.as_bytes().to_vec(),
            value: (cm_index as u32).to_le_bytes().to_vec(),
        });
        database_transaction.push(Op::Insert {
            col: COL_META,
            key: KEY_CURR_MEMO_INDEX.as_bytes().to_vec(),
            value: (memo_index as u32).to_le_bytes().to_vec(),
        });

        database_transaction.push(Op::Delete {
            col: COL_BLOCK_LOCATOR,
            key: current_block_height.to_le_bytes().to_vec(),
        });

        database_transaction.push(Op::Delete {
            col: COL_BLOCK_LOCATOR,
            key: block_hash.0.to_vec(),
        });

        self.storage.batch(database_transaction)?;

        self.current_block_height.fetch_sub(1, Ordering::SeqCst);

        self.update_merkle_tree(new_best_block_number)?;

        Ok(block_hash)
    }

}
