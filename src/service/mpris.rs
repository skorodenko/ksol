use super::mpris_interface::Player;
use futures::future::BoxFuture;
use mpris_server::{Property, Server, Signal, Time};
use std::sync::Arc;
use std::time::Duration;

pub trait MPRISAction {
    fn queue(self, mpris: Arc<Server<Player>>) -> BoxFuture<'static, Result<(), MPRISActionError>>;
}

#[derive(Debug)]
pub enum MPRISActionError {
    MPRISError(String),
}

#[derive(Clone)]
pub struct PropertyUpdate {
    props: Vec<Property>,
}

impl PropertyUpdate {
    pub fn new(props: Vec<Property>) -> PropertyUpdate {
        PropertyUpdate { props }
    }
}

impl MPRISAction for PropertyUpdate {
    fn queue(self, mpris: Arc<Server<Player>>) -> BoxFuture<'static, Result<(), MPRISActionError>> {
        let props = self.props.clone();
        Box::pin(async move {
            mpris
                .properties_changed(props)
                .await
                .map_err(|x| MPRISActionError::MPRISError(x.to_string()))
        })
    }
}

#[derive(Clone)]
pub struct Seeked {
    offset: Duration,
}

impl Seeked {
    pub fn new(offset: Duration) -> Seeked {
        Seeked { offset }
    }
}

impl MPRISAction for Seeked {
    fn queue(self, mpris: Arc<Server<Player>>) -> BoxFuture<'static, Result<(), MPRISActionError>> {
        let signal = Signal::Seeked { position: Time::from_secs(self.offset.as_secs() as i64) };
        Box::pin(async move {
            mpris.emit(signal).await.map_err(|x| MPRISActionError::MPRISError(x.to_string()))
        })
    }
}
