use crate::actor::Role;

pub struct KeyBlob{
    data: Vec<u8>
}

impl KeyBlob{
    pub fn new<T: Role + ?Sized>(key: T::Key) -> Self{
        todo!();
    }
}

pub struct ActorBlob{
    data: Vec<u8>
}

pub struct LocationBlob{
    data: Vec<u8>
}