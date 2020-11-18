use std::marker::PhantomData;

pub mod macro_helpers;
pub mod internal;

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

pub trait Context {
    type DB;
    type Connection;

    fn get_connection<'a>(&'a self) -> &'a Self::Connection;
}

pub trait QueryModifier<T, C: Context> {
    fn modify_query(query: T, context: &C) -> T;
}