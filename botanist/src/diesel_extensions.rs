#[cfg(feature = "postgres_prefix_search")]
pub mod prefix_search {
    use std::marker::PhantomData;

    use diesel::pg::Pg;
    use diesel::expression::{Expression, AsExpression, AppearsOnTable};
    use diesel::sql_types::{Float, Text, Bool, Integer};
    use diesel::query_builder::{QueryFragment, AstPass, Query, AsQuery};
    use diesel::types::{SingleValue};
    use diesel::result::{QueryResult};
    use diesel::serialize::ToSql;

    // https://www.postgresql.org/docs/current/textsearch-intro.html#TEXTSEARCH-MATCHING
    sql_function!(to_tsquery, TextSearchQuery, (query: Text) -> Text);
    sql_function!(to_tsvector, TextSearchVector, (input: Text) -> Text);

    // https://www.postgresql.org/docs/current/textsearch-intro.html#TEXTSEARCH-MATCHING
    diesel_infix_operator!(Matches, " @@ ", Bool, backend: Pg);

    pub fn matches<T, U>(left: T, right: U) -> Matches<T, U::Expression> where
        T: Expression,
        U: AsExpression<T::SqlType>,
    {
        Matches::new(left, right.as_expression())
    }

    #[derive(QueryId)]
    pub struct Position<T, E> {
        substring: String,
        expr: E,
        _marker: PhantomData<T>,
    }

    pub fn position<T, E: AsExpression<T>>(expr: E, substring: String) -> Position<T, E::Expression> {
        Position {
            expr: expr.as_expression(),
            substring,
            _marker: PhantomData
        }
    }

    impl<T, E> Expression for Position<T, E> {
        type SqlType = Integer;
    }

    impl<QS, T, E> AppearsOnTable<QS> for Position<T, E> where Position<T, E>: Expression {}

    impl<T, E: QueryFragment<Pg>> QueryFragment<Pg> for Position<T, E> {
        fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
            out.push_sql("POSITION(");
            out.push_bind_param::<Text, _>(&self.substring)?;
            out.push_sql(" in ");
            (&self.expr).walk_ast(out.reborrow())?;
            out.push_sql(")");
            Ok(())
        }
    }
}

#[cfg(not(feature = "postgres_prefix_search"))]
pub mod prefix_search {}
