use crate::actor::Role;
use crate::backstage::transport::Location;

pub struct KeyBlob{
    type_name: Vec<u8>,
    data: Vec<u8>
}

impl KeyBlob{
    pub fn new<T: Role + ?Sized>(key: T::Key) -> Self{
        todo!();
    }

    pub fn into_key<T: Role + ?Sized>(self) -> T::Key{
        todo!();
    }
}

pub struct ActorBlob{
    data: Vec<u8>
}

impl ActorBlob{
    pub fn new<T: Role + ?Sized>(actor: &T::Actor) -> Self{
        todo!();
    }

    pub fn into_actor<T: Role + ?Sized>(self) -> T::Actor{
        todo!();
    }
}


pub struct LocationBlob{
    data: Vec<u8>
}

impl LocationBlob{
    pub fn new<L: Location>(actor: L) -> Self{
        todo!();
    }

    pub fn into_location<L: Location>(self) -> L{
        todo!();
    }
}