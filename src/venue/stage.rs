use dashmap::{DashMap, mapref::entry::Entry};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::future::Future;
use std::pin::Pin;
use crate::actor::{Role, ActorChannel, ActorInstance, remote::RemoteActor};
use crate::BroadwayContext;
use crate::data::*;
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

    fn start_actor(&mut self, ctx: Arc<BroadwayContext>){
        if let Actor::Local(ref mut local) = self.instance{
            local.start_actor(ctx);
        }
    }
}

enum Actor<T: Role + ?Sized>{
    Local(ActorInstance<T>),
    Remote(RemoteActor<T>)
}

pub enum ActorStatus{
    Starting,
    Running,
    ShuttingDown,
    Off
}

pub struct Stage<T: Role + ?Sized + 'static>{
    actors: DashMap<T::Key, ActorManager<T>>,
    ctx: Arc<BroadwayContext>,
}

impl<T: Role + ?Sized + 'static> Stage<T>{
    pub fn new(ctx: Arc<BroadwayContext>) -> Self{
        Self{
            actors: DashMap::new(),
            ctx,
        }
    }

    pub async fn get_actor(&self, key: T::Key) -> ActorChannel<T>{
        // Check if we already have it
        if let Entry::Occupied(mut o) = self.actors
            .entry(key.clone()){
                if o.get().needs_start(){
                    o.get_mut().start_actor(self.ctx.clone());
                }
                return o.get().comms.clone()
            };

        let key_blob = KeyBlob::new::<T>(key.clone());
        match self.ctx.backstage.get_actor(key_blob){
            Lease::Empty(empty) => todo!(),
            Lease::Created(created) => self.created_lease_handler(created, key).await,
            Lease::Stored(stored) => todo!(),
        }
    }

    async fn created_lease_handler(&self, created: CreatedLease, key: T::Key) -> ActorChannel<T>{
        match created{
            CreatedLease::Local(local) => self.local_lease_handler(local, key).await,
            CreatedLease::Remote(remote) => todo!(),
        }
    }

    fn local_lease_handler<'a>(&'a self, local: LocalLease, key: T::Key) -> Pin<Box<dyn Future<Output = ActorChannel<T>> + 'a>>{
        // If we got a local lease, that means that another thread is making the actor
        // so we can get the entry for where the actor will go to see if it's been made yet
        // if it hasn't, we can try to make the actor and insert it ourselves.
        match self.actors.entry(key.clone()){
            Entry::Occupied(mut o) => {
                if o.get().needs_start(){
                    o.get_mut().start_actor(self.ctx.clone());
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
}