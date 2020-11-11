use diesel::result::Error;
use juniper::{Context, DefaultScalarValue, LookAheadSelection, FieldResult};
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
    ) -> Result<(), Error>;
}

pub trait CreateMutation<C: Context, T, Q> {
    fn create(
        context: &C,
        self_model: T,
    ) -> FieldResult<Q>;
}

pub trait UpdateMutation<C: Context, T, Q> {
    fn update(
        context: &C,
        self_model: T,
    ) -> FieldResult<Q>;
}

pub trait DeleteMutation<C: Context, T, Q> {
    fn delete(
        context: &C,
        id: T,
    ) -> FieldResult<Q>;
}