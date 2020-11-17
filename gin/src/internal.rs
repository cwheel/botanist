#![allow(non_camel_case_types)]

use diesel::query_builder::BoxedSelectStatement;
use diesel::result::Error;
use juniper::{DefaultScalarValue, LookAheadSelection, FieldResult, Executor};
use juniper::Context as JuniperContext;
use crate::Context as GinContext;

pub trait __internal__Preloadable<C: JuniperContext + GinContext, T> {
    fn preload_children(
        self_models: &Vec<T>,
        context: &C,
        look_ahead: &LookAheadSelection<DefaultScalarValue>,
    ) -> Result<(), Error>;
}

pub trait __internal__CreateMutation<C: JuniperContext + GinContext, T, Q> {
    fn create(
        context: &C,
        self_model: T,
    ) -> FieldResult<Q>;
}

pub trait __internal__UpdateMutation<C: JuniperContext + GinContext, T, Q> {
    fn update(
        context: &C,
        self_model: T,
    ) -> FieldResult<Q>;
}

pub trait __internal__DeleteMutation<C: JuniperContext + GinContext, T, Q> {
    fn delete(
        context: &C,
        id: T,
    ) -> FieldResult<Q>;
}

pub trait __internal__RootResolver<C: JuniperContext + GinContext, T, Q, S> {
    fn resolve_single(
        context: &C,
        id: T,
    ) -> FieldResult<Q>;

    fn resolve_multiple(
        context: &C,
        executor: &Executor<C, S>,
        ids: Vec<T>,
        first: Option<i32>,
        offset: Option<i32>
    ) -> FieldResult<Vec<Q>>;
}

pub trait __internal__QueryModifier<T> {
    fn maybe_modify_query(query: T) -> T;
}