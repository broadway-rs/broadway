pub mod transport;

use core::ops::Deref;
use std::any::{Any};
use crate::actor::{Role, ActorChannel, Actor};
use crate::data::{ActorBlob, LocationBlob, KeyBlob};

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

pub trait Backstage: Send + Sync{
    /// Try to get the actor
    fn get_actor(&self, key: KeyBlob) -> Lease;

    /// Update the actor, only the controlling node can do this
    fn update_actor(&self, actor: ActorBlob, lease: LocalLease) -> LocalLease;

    /// Store the local actor, this consumes the given actor
    fn store_local_actor(&self, actor: ActorBlob, lease: LocalLease) -> StoredLease;

    /// Store an actor on another node, necessary for scale down and fail over
    fn store_remote_actor(&self, lease: RemoteLease) -> StoredLease;

    /// Delete the local actor
    fn delete_local_actor(&self, actor: ActorBlob, lease: LocalLease) -> EmptyLease;

    /// Delete an actor on another node, this consumes the given actor
    fn delete_remote_actor(&self, lease: RemoteLease) -> EmptyLease;
}