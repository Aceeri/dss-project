
# DSS Streaming Home

## Build
Building this project on Windows, Mac, or Linux requires a somewhat recent version of Rust. I haven't really tested much beyond the current stable as of writing this (1.55.0), but it does use async/await
to deal with fetching resources from the API so probably somewhere around 1.45.0+ is necessary?

```bash
cargo run --release
```

Arrow Keys - Navigation
F11 - Fullscreen (kind of buggy, need to fix inner size issues when going exclusive)
Esc - Close window

## TODO Improvements
- Cache images locally to free up memory when not in use, but not require as much future network bandwidth.
- Load images asynchronously into the renderer as well.
- Texture atlases/arrays for tile images so we don't have to send as many draw calls. Texture atlases are probably more viable for older hardware, but requires some rectangle packing fun and such. Texture arrays would be a cleaner way to do it without having deal with all the issues of texture atlases, but requires some more modern features.
- Rework the renderer a bit needs some more organization and some separation from UI elements.
- Anti-aliasing would be good to add at some point, whether that is just FXAA or something more advanced like SMAA, would need to research more into this though.