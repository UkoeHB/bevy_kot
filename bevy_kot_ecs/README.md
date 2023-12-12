# ECS

[`ReactCommands`] makes it easy to register and unregister ECS hooks.

### Reaction Triggers

Reactors are registered using 'reaction triggers':
```rust
fn setup(mut rcommands: ReactCommands)
{
    rcommands.on(resource_mutation::<A>(),
        |counter: ReactRes<A>|
        {
            //...
        }
    );
}
```

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
fn setup(mut rcommands: ReactCommands)
{
    rcommands.on((resource_mutation::<A>(), entity_insertion<B>(entity)),
        move |a: ReactRes<A>, q: Query<&B>|
        {
            q.get(entity);
            //...etc.
        }
    );
}
```

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

### Components

```rust
#[derive(ReactComponent)]
struct Health(u16);

fn setup(mut rcommands: ReactCommands)
{
    let entity = rcommands.commands().spawn_empty().id();
    rcommands.insert(entity, Health(0u16));

    rcommands.on(entity_mutation::<Health>(entity)
        |q: Query<&React<Health>>|
        {
            let health = q.get(entity).unwrap();
            println!("health: {}", health.0);
        }
    );
}

fn add_health(mut rcommands: ReactCommands, mut q: Query<&mut React<Health>>)
{
    for health in q.iter_mut()
    {
        health.get_mut(&mut rcommands).0 += 10;
    }
}
```

Note that entity-agnostic triggers can only be grouped with each other, since they require an `In<Entity>` system parameter:
```rust
#[derive(ReactComponent)]
struct A;
#[derive(ReactComponent)]
struct B;

rcommands.on((insertion::<A>(), removal::<B>()),
    |In(entity): In<Entity>, a: Query<(Option<&React<A>>, Option<&React<B>>)>|
    {
        //...
    }
);
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

### Despawns

React to despawns with the [`ReactCommands::on_despawn()`] method:
```rust
rcommands.on_despawn(entity, move || println!("entity despawned: {}", entity));
```
