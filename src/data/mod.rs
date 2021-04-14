use crate::actor::Role;

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

pub struct LocationBlob{
    data: Vec<u8>
}