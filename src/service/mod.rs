pub mod mpd;
pub mod mpris;
pub mod mpris_interface;

use mpd::MPDAction;
use mpris::{MPRISAction, MPRISActionError};
use mpris_interface::Player;

use anyhow::Result;
use futures::future::BoxFuture;
use mpd_client::ClientController;
use mpris_server;
use std::pin::Pin;
use std::sync::Arc;
use tower::Service;

pub type BoxSyncFuture<'a, T> =
    Pin<Box<dyn Future<Output = T> + Send + Sync + 'a>>;

#[derive(Clone)]
pub struct MPDActionService {
    mpd_client: ClientController,
}

impl MPDActionService {
    pub fn new(mpd_client: ClientController) -> MPDActionService {
        MPDActionService { mpd_client }
    }
}

impl<C: MPDAction + Clone> Service<C> for MPDActionService {
    type Response = C::Response;
    type Error = anyhow::Error;
    type Future = BoxSyncFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: C) -> Self::Future {
        req.queue(self.mpd_client.clone())
    }
}

#[derive(Clone)]
pub struct MPRISActionService {
    mpris: Arc<mpris_server::Server<Player>>,
}

impl MPRISActionService {
    pub fn new(mpris: mpris_server::Server<Player>) -> MPRISActionService {
        MPRISActionService { mpris: Arc::new(mpris) }
    }
}

impl<C: MPRISAction + Clone> Service<C> for MPRISActionService {
    type Response = ();
    type Error = MPRISActionError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: C) -> Self::Future {
        req.queue(self.mpris.clone())
    }
}
