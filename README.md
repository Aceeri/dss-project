
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

## TODO Improvements (Stuff I would've liked to do given more time)
- Menu state could do with some decoupling I think, I tried to keep the position hierarchy simple, but I think it might be a bit slow given the nature of how the parent -> child position transferring works.
- Cache images locally to free up memory when not in use, but not require as much future network bandwidth.
- Texture atlases/arrays for tile images so we don't have to send as many draw calls. Texture atlases are probably more viable for older hardware, but requires some rectangle packing fun and such. Texture arrays would be a cleaner way to do it without having deal with all the issues of texture atlases, but requires some more modern features.
- Rework the renderer a bit needs some more organization and some separation from UI elements.
- Image grabber could also probably be improved a good bit, I didn't go super deep into it aside from getting it to just run on a separate thread and grab all of the images needed. I'm not sure how gracefully it handles issues, would need to test that.
- Anti-aliasing would be good to add at some point, whether that is just FXAA, would need to research more into this though.