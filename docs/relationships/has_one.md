# Has One

```rust
pub struct Hero {
    ...
    pub location: HasOne<Uuid, location, Location>
}
```

`HasOne` fields are an abstraction over Diesel model fields (i.e database forign keys). The `HasOne` type takes the type of the underlying field (in this case a `Uuid`), the referenced models schema and the referenced model itself.

At compile time, this is removed and the Diesel model `Hero` will be given a simple `Uuid` location field. However, our GraphQL schema will show `location` to have a type of `Location` and querying for a `Hero`s `location` will allow access to the full type!

`HasOne` takes the form of `HasOne<Primary Key Type, Type Schema, Type Diesel Model>`.