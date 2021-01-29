// Copyright 2021 Parity Technologies (UK) Ltd.
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

use std::convert::From;
use std::pin::Pin;
use std::result::Result;

use futures::channel::mpsc;
use futures::stream::Stream;
use futures::task::{Context, Poll};
use strum::IntoEnumIterator;

use parity_scale_codec::{Decode, Error as DecodingError};

use sc_network::config as network;
use sc_network::PeerId;

use polkadot_node_network_protocol::request_response::{
	request::IncomingRequest, v1, Protocol, RequestResponseConfig,
};
use polkadot_subsystem::messages::AllMessages;

/// Multiplex incoming network requests.
///
/// This multiplexer consumes all request streams and makes them a `Stream` of a single message
/// type, useful for the network bridge to send them via the `Overseer` to other subsystems.
pub struct RequestMultiplexer {
	receivers: Vec<(Protocol, mpsc::Receiver<network::IncomingRequest>)>,
	next_poll: usize,
}

/// Multiplexing can fail in case of invalid messages.
pub struct RequestMultiplexError {
	/// The peer that sent the invalid message.
	pub peer: PeerId,
	/// The error that occurred.
	pub error: DecodingError,
}

impl RequestMultiplexer {
	/// Create a new `RequestMultiplexer`.
	///
	/// This function uses `Protocol::get_config` for each available protocol and creates a
	/// `RequestMultiplexer` from it. The returned `RequestResponseConfig`s must be passed to the
	/// network implementation.
	pub fn new() -> (Self, Vec<RequestResponseConfig>) {
		let (receivers, cfgs): (Vec<_>, Vec<_>) = Protocol::iter()
			.map(|p| {
				let (rx, cfg) = p.get_config();
				((p, rx), cfg)
			})
			.unzip();

		(
			Self {
				receivers,
				next_poll: 0,
			},
			cfgs,
		)
	}
}

impl Stream for RequestMultiplexer {
	type Item = Result<AllMessages, RequestMultiplexError>;

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let len = self.receivers.len();
		let mut count = len;
		let mut i = self.next_poll;
		let mut result = Poll::Ready(None);
		// Poll streams in round robin fashion:
		while count > 0 {
			let (p, rx): &mut (_, _) = &mut self.receivers[i % len];
			i += 1;
			match Pin::new(rx).poll_next(cx) {
				// If at least one stream is pending, we are pending as well:
				Poll::Pending => result = Poll::Pending,
				Poll::Ready(None) => {}
				Poll::Ready(Some(v)) => {
					result = Poll::Ready(Some(multiplex_single(*p, v)));
					break;
				}
			}
			count -= 1;
		}
		self.next_poll = i % len;
		result
	}
}

/// Convert a single raw incoming request into a `MultiplexMessage`.
fn multiplex_single(
	p: Protocol,
	network::IncomingRequest {
		payload,
		peer,
		pending_response,
	}: network::IncomingRequest,
) -> Result<AllMessages, RequestMultiplexError> {
	let r = match p {
		Protocol::AvailabilityFetching => From::from(IncomingRequest::new(
			peer,
			decode_with_peer::<v1::AvailabilityFetchingRequest>(peer, payload)?,
			pending_response,
		)),
	};
	Ok(r)
}

fn decode_with_peer<Req: Decode>(
	peer: PeerId,
	payload: Vec<u8>,
) -> Result<Req, RequestMultiplexError> {
	match Req::decode(&mut payload.as_ref()) {
		Err(error) => Err(RequestMultiplexError { peer, error }),
		Ok(req) => Ok(req),
	}
}
