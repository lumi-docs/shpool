// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{io, path::Path};

use anyhow::Context;
use shpool_protocol::{ConnectHeader, SendInputReply, SendInputRequest};

use crate::{protocol, protocol::ClientResult};

/// Inject hex-encoded bytes into a named session's PTY master fd.
///
/// Exit codes:
/// - 0: bytes written successfully (session was ready)
/// - 1: session exists but is not ready yet (retry)
/// - 2: session not found
pub fn run<P>(session: String, hex_data: String, socket: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let data = hex::decode(&hex_data).context("decoding hex data")?;

    let mut client = match protocol::Client::new(socket) {
        Ok(ClientResult::JustClient(c)) => c,
        Ok(ClientResult::VersionMismatch { client, .. }) => client,
        Err(err) => {
            let io_err = err.downcast::<io::Error>()?;
            return Err(io_err).context("connecting to daemon");
        }
    };

    client
        .write_connect_header(ConnectHeader::SendInput(SendInputRequest { session, data }))
        .context("writing send_input request")?;

    let reply: SendInputReply = client.read_reply().context("reading reply")?;

    match reply {
        SendInputReply::Ok => Ok(()),
        SendInputReply::NotFound => std::process::exit(1),
    }
}
