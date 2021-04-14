pub mod remote;

use serde::{Serialize, de::DeserializeOwned};
use async_std::channel::Sender;
use async_std::sync::Arc;
use async_std::sync::RwLock;
use async_std::task;
use async_trait::async_trait;
use core::hash::Hash;
use futures::future;
use std::ops::{Deref, DerefMut};
use switch_channel::async_channel::async_std::{diunbounded, DiSwitchReceiver, DiSwitchSender};
use switch_channel::err::recv::RecvError;
use crate::BroadwayContext;

pub trait Role {
    type Actor: Actor + Serialize + DeserializeOwned;
    type Key: Hash + Eq + Serialize + DeserializeOwned + Copy;

    type Calls: Handler<Self::Actor>;
    type MutCalls: MutHandler<Self::Actor>;
}

#[async_trait]
pub trait Actor: Send + Sync {
    fn new() -> Self;

    async fn start(&mut self, ctx: Arc<BroadwayContext>) {}

    async fn stop(&mut self) {}
}

pub struct ActorInstance<T: Role + ?Sized> {
    pub(self) handling_loop: Option<Box<dyn std::future::Future<Output = T::Actor>>>,
    pub(self) actor: Option<T::Actor>,
    pub(self) channel: Option<(DiSwitchReceiver<T::Calls>, DiSwitchReceiver<T::MutCalls>)>,
}

pub struct ActorChannel<T: Role + ?Sized> {
    pub calls_sender: DiSwitchSender<T::Calls>,
    pub mut_calls_sender: DiSwitchSender<T::MutCalls>,
}

impl<T: Role + ?Sized> Clone for ActorChannel<T> {
    fn clone(&self) -> Self {
        Self {
            calls_sender: self.calls_sender.clone(),
            mut_calls_sender: self.mut_calls_sender.clone(),
        }
    }
}

impl<T: 'static + Role + ?Sized> ActorInstance<T> {
    pub(crate) fn build_actor() -> (ActorChannel<T>, ActorInstance<T>) {
        let (mut_calls_sender, mut_calls) = diunbounded();
        let (calls_sender, calls) = diunbounded();

        let actor = T::Actor::new();

        (
            ActorChannel {
                calls_sender,
                mut_calls_sender,
            },
            ActorInstance {
                handling_loop: None,
                actor: Some(actor),
                channel: Some((calls, mut_calls)),
            },
        )
    }

    /// This function is kind of weird, because I only ever want an actor that has been instantiated
    /// to start ONCE across an entire cluster, knowing this may be impossible, I atleast want to ensure
    /// it only ever happens ONCE per node.
    pub(crate) fn start_actor(&mut self, ctx: Arc<BroadwayContext>){
        if let Some(mut actor) = self.actor.take(){
            let (calls, mut_calls) = self.channel.take().unwrap();
            self.handling_loop = Some(Box::new(task::spawn(Self::run_actor(ctx, actor, calls, mut_calls))))
        }
    }

    async fn run_actor(
        ctx: Arc<BroadwayContext>,
        mut actor: T::Actor,
        calls: DiSwitchReceiver<T::Calls>,
        mut_calls: DiSwitchReceiver<T::MutCalls>,
    ) -> T::Actor {
        actor.start(ctx).await;
        let actor = RwLock::new(actor);
        let mut call_fut = None;
        let mut mut_call_fut = None;
        let mut priority = false;
        let actor = Arc::new(actor);
        loop {
            if call_fut.is_none() && (!calls.is_empty() || !calls.is_closed()) {
                call_fut = Some(Box::pin(calls.recv()));
            }

            if mut_call_fut.is_none() && (!mut_calls.is_empty() || !mut_calls.is_closed()) {
                mut_call_fut = Some(Box::pin(mut_calls.recv()));
            }

            match (call_fut.take(), mut_call_fut.take()) {
                (Some(call), Some(mut_call)) => {
                    // Not the biggest fan of this setup, but select
                    // favors the left side so we need to swap to prevent
                    // starvation
                    if priority {
                        match future::select(call, mut_call).await {
                            future::Either::Left((call, mut_call)) => {
                                // Call Logic
                                Self::call_loop(actor.clone(), call, &calls).await;
                                mut_call_fut = Some(mut_call);
                            }
                            future::Either::Right((mut_call, call)) => {
                                // Immut call logic
                                Self::mut_call_loop(actor.clone(), mut_call, &mut_calls).await;
                                call_fut = Some(call);
                            }
                        }
                    } else {
                        match future::select(mut_call, call).await {
                            future::Either::Right((call, mut_call)) => {
                                // Call Logic
                                Self::call_loop(actor.clone(), call, &calls).await;
                                mut_call_fut = Some(mut_call);
                            }
                            future::Either::Left((mut_call, call)) => {
                                // Immut call logic
                                Self::mut_call_loop(actor.clone(), mut_call, &mut_calls).await;
                                call_fut = Some(call);
                            }
                        }
                    }
                    priority = !priority;
                }
                (Some(call_fut), None) => {
                    Self::call_loop(actor.clone(), call_fut.await, &calls).await
                }
                (None, Some(mut_call_fut)) => {
                    Self::mut_call_loop(actor.clone(), mut_call_fut.await, &mut_calls).await
                }
                (None, None) => break,
            };
        }
        Arc::try_unwrap(actor).ok().unwrap().into_inner()
    }

    async fn call_loop(
        actor: Arc<RwLock<T::Actor>>,
        first: Result<T::Calls, RecvError>,
        calls: &DiSwitchReceiver<T::Calls>,
    ) {
        future::join_all(
            first
                .ok()
                .into_iter()
                .chain(calls.switch().into_iter())
                .map(|call| {
                    let actor = actor.clone();
                    task::spawn(async move { call.handle(&actor.read().await.deref()).await })
                }),
        )
        .await;
    }

    async fn mut_call_loop(
        actor: Arc<RwLock<T::Actor>>,
        first: Result<T::MutCalls, RecvError>,
        calls: &DiSwitchReceiver<T::MutCalls>,
    ) {
        for call in first.ok().into_iter().chain(calls.switch().into_iter()) {
            call.handle_mut(&mut actor.write().await.deref_mut()).await
        }
    }
}

unsafe impl<T: Role> Send for ActorInstance<T> {}
unsafe impl<T: Role> Sync for ActorInstance<T> {}

#[async_trait]
pub trait Handler<T>: Send {
    async fn handle(self, actor: &T);
}

#[async_trait]
pub trait MutHandler<T>: Send {
    async fn handle_mut(self, actor: &mut T);
}

pub struct Call<C, R> {
    pub return_channel: Sender<R>,
    pub call: C,
}

unsafe impl<T: Send, R: Send> Send for Call<T, R> {}
