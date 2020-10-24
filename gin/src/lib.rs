use std::marker::PhantomData;
use juniper::Context;

#[derive(Debug, Clone)]
pub struct HasOne<T, S, M> {
    sql_type: PhantomData<T>,
    schema: PhantomData<S>,
    model: PhantomData<M>,
}

#[derive(Debug, Clone)]
pub struct HasMany<S, F, M> {
    schema: PhantomData<S>,
    forign_key: PhantomData<F>,
    model: PhantomData<M>,
}

pub trait Preloadable<C: Context, T> {
    fn preload_children(self_models: &Vec<T>, context: &C);
}