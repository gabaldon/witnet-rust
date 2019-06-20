//! Error type definition
use failure::Fail;

use witnet_net::client::tcp;
use witnet_rad::error::RadError;

use crate::storage;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "wallet not connected to a node")]
    NodeNotConnected,
    #[fail(display = "could not send request")]
    RequestFailedToSend(#[cause] actix::MailboxError),
    #[fail(display = "request failed with an error: {}", _0)]
    RequestFailed(#[cause] tcp::Error),
    #[fail(display = "could not subscribe: {}", _0)]
    SubscribeFailed(&'static str),
    #[fail(display = "could not unsubscribe: {}", _0)]
    UnsubscribeFailed(&'static str),
    #[fail(display = "could not run RAD request")]
    RadScheduleFailed(#[cause] actix::MailboxError),
    #[fail(display = "{}", _0)]
    RadFailed(#[cause] RadError),
    #[fail(display = "could not communicate with database")]
    StorageCommFailed(#[cause] actix::MailboxError),
    #[fail(display = "{}", _0)]
    StorageOpFailed(#[cause] storage::Error),
    #[fail(display = "could not communicate with cryptographic engine")]
    CryptoCommFailed(#[cause] actix::MailboxError),
}