# Note
The plugin removes `bevy_time::TimeReceiver` to make sure it is the only system reading from it.
I don't like this, but I don't know a better way.

# Example
Run:
```sh
cargo run --example hello_world
```
