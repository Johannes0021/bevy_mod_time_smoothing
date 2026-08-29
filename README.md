# Note
The plugin removes `bevy_time::TimeReceiver` to make sure it is the only system reading from it.
I don't like this, but I don't know a better way.

Check after Bevy updates that `TimeReceiver` is still used the same way.
Currently, it is only used in bevy_time:
https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_time/src/lib.rs

# Example
Run:
```sh
cargo run --example hello_world
```
