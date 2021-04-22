pub mod actor;
pub use broadway_macro;
pub mod venue;
pub mod backstage;

use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct BroadwayContext{
    backvenue: backstage::BackVenue,
    venue: AtomicPtr<venue::Venue>,
}

impl BroadwayContext{
    pub fn new<B, T>() -> Arc<Self>{
        let ctx = Arc::new(Self{
            backvenue: backstage::BackVenue::new(),
            venue: AtomicPtr::new(std::ptr::null_mut()),
        });

        let venue = Box::into_raw(Box::new(venue::Venue::new(Arc::downgrade(&ctx))));

        ctx.venue.store(venue, Ordering::SeqCst);
        ctx
    }

    async fn get_actor<T: 'static + actor::Role + ?Sized>(&self, key: T::Key) -> actor::ActorChannel<T>{
        unsafe{self.venue.load(Ordering::SeqCst).as_ref().unwrap()}.get_actor(key).await
    }
}

impl Drop for BroadwayContext{
    fn drop(&mut self){
        unsafe{Box::from_raw(self.venue.load(Ordering::SeqCst));}
    }
}