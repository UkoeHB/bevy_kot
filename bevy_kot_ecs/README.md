# ECS

[`ReactCommands`] makes it easy to register and unregister ECS hooks.

### Resource Mutation

Add a reactive resource:
```rust
#[derive(ReactResource)]
struct Counter(u32);

app.add_plugins(ReactPlugin)
    .add_react_resource(Counter);
```

Mutate the counter:
```rust
fn increment(mut rcommands: ReactCommands, mut counter: ReactResMut<Counter>)
{
    counter.get_mut(&mut rcommands).0 += 1;
}
```

React to the counter:
```rust
fn setup(mut rcommands: ReactCommands)
{
    rcommands.on(resource_mutation::<Counter>(),
        |counter: ReactRes<Counter>|
        {
            println!("count: {}", counter.0);
        }
    );
}
```

### Events

React events can be sent like this:
```rust
rcommands.send(0u32);
```

And then reacted to with the [`ReactEvents`] reader:
```rust
rcommands.on(event::<u32>(),
    |mut events: ReactEvents<u32>|
    {
        for event in events.iter()
        {
            println!("react u32: {}", event);
        }
    }
);
```

### All Reaction Triggers

The available reaction triggers are:
- [`resource_mutation<R: ReactResource>()`]
- [`insertion<C: ReactComponent>()`]
- [`mutation<C: ReactComponent>()`]
- [`removal<C: ReactComponent>()`]
- [`entity_insertion<C: ReactComponent>(entity)`]
- [`entity_mutation<C: ReactComponent>(entity)`]
- [`entity_removal<C: ReactComponent>(entity)`]
- [`event<E>()`]

A reactor can be associated with multiple reaction triggers:
```rust
rcommands.on((resource_mutation::<A>(), event::<u32>()),
    |a: ReactRes<A>, mut e: ReactEvents<u32>|
    {
        //...
    }
);
```

Note that entity-agnostic triggers can only be grouped with each other, since they require an `In<Entity>` system parameter:
```rust
#[derive(ReactComponent)]
struct A;
#[derive(ReactComponent)]
struct B;

rcommands.on((insertion::<A>(), removal::<B>()),
    |In(entity): In<Entity>, a: Query<&React<A>>, b: Query<Removed<React<B>>|
    {
        //...
    }
);
```

### Despawns

Despawns can be reacted to with the [`ReactCommands::on_despawn()`] method, which takes a `FnOnce` closure instead of `FnMut`:
```rust
rcommands.on_despawn(entity, move || println!("entity despawned: {}", entity));
```
