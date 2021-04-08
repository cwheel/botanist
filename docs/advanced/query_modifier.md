# Query Modifiers
Query modifiers provide you a last minute chance to modify the query of any _root resolver_ before its query is run. These query modifiers provide a convenient time to perform query level authentication. To setup a query modifier, update the `botanist_object` attribute to include `ModifiesQuery = true`. It should look like the following:

```rust
#[botanist_object(Context = Context, ModifiesQuery = true)]
```

Next, implement the trait `QueryModifier` on the Diesel model in question:

```rust
impl<'a> QueryModifier<HeroQuery<'a>, Context> for Hero {
    fn modify_query(query: HeroQuery<'a>, context: &Context) -> HeroQuery<'a> {
        query
    }
}
```
::: tip BoxedSelectQuery
The `HeroQuery` type is automatically generated for convenience. A helper type will be generated in the form of `<Model Name>Query`.
:::

Generally, you'll want to modify the query here instead of just returning it as is. Any standard functions from the Diesel DSL will work here, `HeroQuery` is just a `BoxedSelectQuery`.