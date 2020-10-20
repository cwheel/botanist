use std::marker::PhantomData;

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

pub trait Preloadable {
    fn preload_children(self_models: Vec<Box<Self>>);
}