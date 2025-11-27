use crate::rust::actions::{Action, ActionError};
use crate::rust::entities::{ColumnSort, MPSCCommand, QSong, SongField};
use crate::rust::qmpd_connector::qobject::QMPDConnector;
use base64::prelude::*;
use bincode::config;
use bincode::serde::encode_to_vec;
use bytes::Bytes;
use core::pin::Pin;
use cxx_qt::{CxxQtThread, CxxQtType};
use cxx_qt_lib::{QByteArray, QString};
use futures::future::BoxFuture;
use mpd_client::{ClientController, commands, filter::Filter, responses::PlayState, responses::Song, tag::Tag};
use quick_cache::unsync::Cache;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use tower::{BoxError, Service};
use tracing;

#[derive(Clone)]
pub(crate) struct ActionService {
    pub mpd_client: ClientController,
    pub qt_thread: CxxQtThread<QMPDConnector>,
}

impl<C: Action + Clone> Service<C> for ActionService {
    type Response = ();
    type Error = ActionError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: C) -> Self::Future {
        req.queue(self.mpd_client.clone(), self.qt_thread.clone())
    }
}

impl ActionService {
    pub fn new(mpd_client: ClientController, qt_thread: CxxQtThread<QMPDConnector>) -> ActionService {
        ActionService { mpd_client, qt_thread }
    }
}
