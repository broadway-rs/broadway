use super::Role;

pub struct RemoteActor<T: Role + ?Sized>{
    data: std::marker::PhantomData<T>,
}