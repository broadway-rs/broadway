use dashmap::{DashMap, mapref::entry::Entry};
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use std::future::Future;
use std::pin::Pin;
use crate::actor::{Role, ActorChannel, ActorInstance, remote::RemoteActor};
use crate::BroadwayContext;
use crate::backstage::*;

/// This might get removed, but the idea with this is that
/// it's the services view of the actor, since the service
/// doesn't need to know much about the internals of an actor
/// and an actor doesn't need to know how it looks to the service
/// except for a few signals.
struct ActorManager<T: Role + ?Sized + 'static>{
    pub(self) comms: ActorChannel<T>,
    pub(self) started: AtomicBool,
    instance: Actor<T>,
}

impl<T: Role + ?Sized + 'static> ActorManager<T>{
    fn needs_start(&self) -> bool{
        match self.started.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst){
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn start_actor<B: Backstage + 'static>(&mut self, ctx: Arc<BroadwayContext<B>>){
        if let Actor::Local(ref mut local) = self.instance{
            local.start_actor(ctx);
        }
    }
}

enum Actor<T: Role + ?Sized>{
    Local(ActorInstance<T>),
    Remote(RemoteActor<T>)
}

pub struct Stage<T: Role + ?Sized + 'static, B: Backstage>{
    actors: DashMap<T::Key, ActorManager<T>>,
    ctx: Weak<BroadwayContext<B>>,
}

impl<T: Role + ?Sized + 'static, B: Backstage + 'static> Stage<T, B>{
    pub async fn new(ctx: Arc<BroadwayContext<B>>) -> Self{
        Self{
            actors: DashMap::new(),
            ctx: Arc::downgrade(&ctx),
        }
    }

    pub async fn get_actor(&self, key: T::Key) -> ActorChannel<T>{
        // Check if we already have it
        if let Entry::Occupied(mut o) = self.actors
            .entry(key.clone()){
                if o.get().needs_start(){
                    o.get_mut().start_actor(self.ctx.clone().upgrade().unwrap());
                }
                return o.get().comms.clone()
            };

        match self.ctx.clone().upgrade().unwrap().backstage.get_actor(key.clone()).await{
            Lease::Empty(empty) => self.empty_lease_handler(empty, key).await,
            Lease::Created(created) => self.created_lease_handler(created, key).await,
            Lease::Stored(stored) => todo!(),
        }
    }

    async fn created_lease_handler(&self, created: CreatedLease<T>, key: T::Key) -> ActorChannel<T>{
        match created{
            CreatedLease::Local(local) => self.local_lease_handler(local, key).await,
            CreatedLease::Remote(remote) => todo!(),
        }
    }

    fn local_lease_handler<'a>(&'a self, local: LocalLease<T>, key: T::Key) -> Pin<Box<dyn Future<Output = ActorChannel<T>> + 'a>>{
        // If we got a local lease, that means that another thread is making the actor
        // so we can get the entry for where the actor will go to see if it's been made yet
        // if it hasn't, we can try to make the actor and insert it ourselves.
        match self.actors.entry(key.clone()){
            Entry::Occupied(mut o) => {
                if o.get().needs_start(){
                    o.get_mut().start_actor(self.ctx.clone().upgrade().unwrap());
                }
                let comms = o.get().comms.clone();
                return Box::pin(async move {comms});
            },
            Entry::Vacant(v) => {
                let (comms, instance) = ActorInstance::build_actor();
                drop(v.insert(ActorManager{
                    started: AtomicBool::new(false),
                    comms,
                    instance: Actor::Local(instance),
                }));
                Box::pin(self.get_actor(key))
            }
        }
    }

    async fn empty_lease_handler(&self, empty: EmptyLease<T>, key: T::Key) -> ActorChannel<T>{
        // If we get an empty lease, we need to try to claim it (THIS BEHAVIOR SOULD BE CHANGED
        // IN THE FUTURE TO ACCOUNT FOR THE NODES CURRENT QUOTAS)
        // We either claim it, or don't, but either way we will get a created lease
        // which we can handle like any other
        todo!();
        let ctx = self.ctx.upgrade().unwrap();
        self.created_lease_handler(
            ctx.backstage.set_actor(empty, ctx.transport.get_local().clone()).await, 
            key).await
    }

    async fn stored_lease_handler(&self, stored: StoredLease<T>, key: T::Key) -> ActorChannel<T>{
        todo!();
    }
}