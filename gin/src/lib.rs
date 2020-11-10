use juniper::{Context, DefaultScalarValue, LookAheadSelection};
use std::marker::PhantomData;

pub mod macro_helpers;

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
    fn preload_children(
        self_models: &Vec<T>,
        context: &C,
        look_ahead: &LookAheadSelection<DefaultScalarValue>,
    );
}

pub trait CreateMutation<C: Context, T, Q> {
    fn create(
        context: &C,
        self_model: T,
    ) -> Q;
}

pub trait UpdateMutation<C: Context, T, Q> {
    fn update(
        context: &C,
        self_model: T,
    ) -> Q;
}


pub trait DeleteMutation<C: Context, T, Q> {
    fn delete(
        context: &C,
        id: T,
    ) -> Q;
}