#[cfg(feature = "postgres_prefix_search")]
pub mod text_search {
    use diesel::pg::Pg;
    use diesel::expression::{Expression, AsExpression};
    use diesel::sql_types::{Float, Text, Bool};

    sql_function!(to_tsquery, TextSearchQuery, (query: Text) -> Text);
    sql_function!(to_tsvector, TextSearchVector, (input: Text) -> Text);

    diesel_infix_operator!(Distance, " <-> ", Float, backend: Pg);
    diesel_infix_operator!(Matches, " @@ ", Bool, backend: Pg);

    pub fn matches<T, U>(left: T, right: U) -> Matches<T, U::Expression> where
        T: Expression,
        U: AsExpression<T::SqlType>,
    {
        Matches::new(left, right.as_expression())
    }

    pub fn distance<T, U>(left: T, right: U) -> Distance<T, U::Expression> where
        T: Expression,
        U: AsExpression<T::SqlType>,
    {
        Distance::new(left, right.as_expression())
    }
}

#[cfg(not(feature = "postgres_prefix_search"))]
pub mod text_search {}
