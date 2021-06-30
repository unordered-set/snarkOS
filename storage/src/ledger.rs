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
use arc_swap::ArcSwap;
use snarkos_parameters::GenesisBlock;
use snarkvm_algorithms::{merkle_tree::MerkleTree, traits::LoadableMerkleParameters};
use snarkvm_dpc::{Block, BlockHeaderHash, DatabaseTransaction, LedgerScheme, Op, Storage, TransactionScheme, errors::StorageError};
use snarkvm_parameters::{traits::genesis::Genesis, LedgerMerkleTreeParameters, Parameter};
use snarkvm_utilities::bytes::FromBytes;

use std::{
    fs,
    marker::PhantomData,
    path::Path,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
use anyhow::*;

#[async_trait::async_trait]
pub trait Ledger {
    async fn get_current_block_height(&self) -> Result<u32>;

    // async fn insert_block(&self, block: &Block<T>) -> Result<()>;

    async fn commit_block(&self, hash: BlockHeaderHash) -> Result<()>;

    async fn decommit_block(&self, hash: BlockHeaderHash) -> Result<()>;

    async fn get_transaction(&self, hash: &[u8]) -> Result<()>;
}
