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

pub struct BackVenue{
    backstage_builder: Box<dyn Fn(BroadwayContext) -> Box<dyn Any>>,
    context: Weak<BroadwayContext>,
    stages: DashMap<TypeId, Box<dyn Any>>,
}

pub enum Lease<A: Role + ?Sized>{
    Empty(EmptyLease<A>),
    Created(CreatedLease<A>),
    Stored(StoredLease<A>),
}

pub enum CreatedLease<A: Role + ?Sized>{
    Local(LocalLease<A>),
    Remote(RemoteLease<A>)
}

pub struct LocalLease<A: Role + ?Sized>{
    key: A::Key
}

pub struct RemoteLease<A: Role + ?Sized>{
    key: A::Key,
    location: Location,
}

pub struct EmptyLease<A: Role + ?Sized>{
    key: A::Key,
}

pub struct StoredLease<A: Role + ?Sized>{
    key: A::Key,
    actor_data: Box<A::Actor>,
}

#[async_trait]
pub trait Backstage<A: Role + ?Sized>: Send + Sync{
    /// Try to get the actor
    async fn get_actor(&self, key: A::Key) -> Lease<A>;

    /// Try to set the actor as being at a given location (should always be local)
    /// this should really always turn a created lease.
    async fn set_actor(&self, empty: EmptyLease<A>, location: Location) -> CreatedLease<A>;

    /// Update the actor, only the controlling node can do this
    async fn update_actor(&self, actor: &A::Actor, lease: LocalLease<A>) -> LocalLease<A>;

    /// Store the local actor, this consumes the given actor
    async fn store_local_actor(&self, actor: &A::Actor, lease: LocalLease<A>) -> StoredLease<A>;

    /// Store an actor on another node, necessary for scale down and fail over
    async fn store_remote_actor(&self, lease: RemoteLease<A>) -> StoredLease<A>;

    /// Delete the local actor
    async fn delete_local_actor(&self, actor: &A::Actor, lease: LocalLease<A>) -> EmptyLease<A>;

    /// Delete an actor on another node, this consumes the given actor
    async fn delete_remote_actor(&self, lease: RemoteLease<A>) -> EmptyLease<A>;
}

#[async_trait]
pub trait BackstageBuilder{
    async fn build_backstage<A: Role + ?Sized>(ctx: Arc<BroadwayContext>) -> Box<dyn Backstage<A>>;
}