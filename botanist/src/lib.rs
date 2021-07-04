#[macro_use]
extern crate diesel;

use juniper::FieldError;
use std::marker::PhantomData;

pub mod internal;
pub mod macro_helpers;

#[cfg(feature = "postgres_text_search")]
pub mod text_search;

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

    fn get_connection<'a>(&'a self) -> Self::Connection;
}

pub trait QueryModifier<T, R, C: Context> {
    fn modify_query(query: T, context: &C) -> Result<R, FieldError>;
}
