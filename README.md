
# DSS Streaming Home

## Build
Building this project on Windows, Mac, or Linux requires a somewhat recent version of Rust. I haven't really tested much beyond the current stable as of writing this (1.55.0), but it does use async/await
to deal with fetching resources from the API.

```bash
cargo run --release
```

Arrow Keys - Navigation
Esc - Close window

## TODO Improvements
- Cache images locally to free up memory when not in use, but not require as much future network bandwidth.
- Texture atlases/arrays for tile images so we don't have to send as many draw calls. Texture atlases are probably more viable for older hardware, but requires some rectangle packing fun and such. Texture arrays would be a cleaner way to do it without having deal with all the issues of texture atlases, but requires some more modern features.
- Probably rework the renderer a bit, so it is more organized, like moving into separate render passes and such.