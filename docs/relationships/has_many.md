# Has Many

```rust
pub struct Hero {
    ...
    pub enemies: HasMany<enemy, enemy::hero, Enemy>
}
```

`HasMany` fields are pure abstraction - that is they result in no change to the underlying Diesel model at compile time. `HasMany`s only serve to generate resolvers capable of returning more than one model at a time.

In the example above, `HasMany` takes the `enemy` schema, the matching forign key and the `Enemy` Diesel model.