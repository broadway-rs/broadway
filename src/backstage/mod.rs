pub mod transport;

use core::ops::Deref;
use std::any::{Any};
use crate::actor::{Role, ActorChannel, Actor};
use crate::data::{ActorBlob, LocationBlob, KeyBlob};
use crate::BroadwayContext;
use async_trait::async_trait;

pub enum Lease{
    Empty(EmptyLease),
    Created(CreatedLease),
    Stored(StoredLease),
}

pub enum CreatedLease{
    Local(LocalLease),
    Remote(RemoteLease)
}

pub struct LocalLease{
    key: KeyBlob
}

pub struct RemoteLease{
    key: KeyBlob,
    location: LocationBlob,
}

pub struct EmptyLease{
    key: KeyBlob,
}

pub struct StoredLease{
    key: KeyBlob,
    actor_data: ActorBlob,
}

#[async_trait]
pub trait Backstage: Send + Sync{
    async fn init_actor(&mut self, ctx: BroadwayContext);

    /// Try to get the actor
    async fn get_actor(&self, key: KeyBlob) -> Lease;

    /// Try to set the actor as being at a given location (should always be local)
    /// this should really always turn a created lease.
    async fn set_actor(&self, empty: EmptyLease, location: LocationBlob) -> CreatedLease;

    /// Update the actor, only the controlling node can do this
    async fn update_actor(&self, actor: ActorBlob, lease: LocalLease) -> LocalLease;

    /// Store the local actor, this consumes the given actor
    async fn store_local_actor(&self, actor: ActorBlob, lease: LocalLease) -> StoredLease;

    /// Store an actor on another node, necessary for scale down and fail over
    async fn store_remote_actor(&self, lease: RemoteLease) -> StoredLease;

    /// Delete the local actor
    async fn delete_local_actor(&self, actor: ActorBlob, lease: LocalLease) -> EmptyLease;

    /// Delete an actor on another node, this consumes the given actor
    async fn delete_remote_actor(&self, lease: RemoteLease) -> EmptyLease;
}