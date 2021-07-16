pub mod transport;

use async_std::sync::Arc;
use transport::Location;
use std::sync::Weak;
use core::any::TypeId;
use dashmap::DashMap;
use core::ops::Deref;
use std::any::{Any};
use crate::actor::{Role, ActorChannel, Actor};
use crate::BroadwayContext;
use async_trait::async_trait;

pub enum Lease<'a>{
    Empty(EmptyLease),
    Created(CreatedLease),
    Stored(StoredLease<'a>),
}

pub enum CreatedLease{
    Local(LocalLease),
    Remote(RemoteLease)
}

pub struct LocalLease{
    pub key: KeyBlob,
}

pub struct RemoteLease{
    pub key: KeyBlob,
    pub location: Location,
}

pub struct EmptyLease{
    pub key: KeyBlob,
}

pub struct StoredLease<'a>{
    pub key: KeyBlob,
    pub actor_data: ActorBlob,
}

#[async_trait]
pub trait Backstage: Send + Sync + Sized{
    /// Try to get the actor
    async fn get_actor(&self, key: KeyBlob) -> Lease;

    /// Try to set the actor as being at a given location (should always be local)
    /// this should really always turn a created lease.
    async fn set_actor(&self, empty: EmptyLease, location: Location) -> CreatedLease;

    /// Update the actor, only the controlling node can do this
    async fn update_actor(&self, actor: &ActorBlob, lease: LocalLease) -> LocalLease;

    /// Store the local actor, this consumes the given actor
    async fn store_local_actor(&self, actor: &ActorBlob, lease: LocalLease) -> StoredLease;

    /// Store an actor on another node, necessary for scale down and fail over
    async fn store_remote_actor(&self, lease: RemoteLease) -> StoredLease<'a>;

    /// Delete the local actor
    async fn delete_local_actor(&self, actor: &ActorBlob, lease: LocalLease) -> EmptyLease;

    /// Delete an actor on another node, this consumes the given actor
    async fn delete_remote_actor(&self, lease: RemoteLease) -> EmptyLease;
}

#[async_trait]
pub trait BackstageBuilder{

}