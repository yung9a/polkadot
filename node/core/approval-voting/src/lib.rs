// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! The Approval Voting Subsystem.
//!
//! This subsystem is responsible for determining candidates to do approval checks
//! on, performing those approval checks, and tracking the assignments and approvals
//! of others. It uses this information to determine when candidates and blocks have
//! been sufficiently approved to finalize.

use polkadot_subsystem::{Subsystem, SubsystemContext, SubsystemError, SubsystemResult, SpawnedSubsystem};
use polkadot_primitives::v1::{ValidatorIndex, Hash, SessionIndex, SessionInfo};
use sc_keystore::LocalKeystore;

use futures::prelude::*;
use futures::channel::mpsc;

use std::collections::BTreeMap;

mod aux_schema;

const APPROVAL_SESSIONS: SessionIndex = 6;

/// A base unit of time, starting from the unix epoch, split into half-second intervals.
type Tick = u64;

/// The approval voting subsystem.
pub struct ApprovalVotingSubsystem {
	// TODO [now]: keystore. chain config?
}

impl<C: SubsystemContext> Subsystem<C> for ApprovalVotingSubsystem {
	fn start(self, ctx: C) -> SpawnedSubsystem {
		let future = run(ctx)
			.map_err(|e| SubsystemError::with_origin("approval-voting", e))
			.boxed();

		SpawnedSubsystem {
			name: "approval-voting-subsystem",
			future,
		}
	}
}

struct ApprovalVoteRequest {
	validator_index: ValidatorIndex,
	block_hash: Hash,
	candidate_index: u32,
}

struct State {
	earliest_session: SessionIndex,
	session_info: Vec<SessionInfo>,
	keystore: LocalKeystore,
	// Tick -> [(Relay Block, Candidate Hash)]
	wakeups: BTreeMap<Tick, Vec<(Hash, Hash)>>,

	// These are connected to each other.
	approval_vote_tx: mpsc::Sender<ApprovalVoteRequest>,
	approval_vote_rx: mpsc::Receiver<ApprovalVoteRequest>,
}

async fn run(_: impl SubsystemContext) -> SubsystemResult<()> {
	loop { }
}
