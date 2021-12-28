# Shadercrab
A simple single buffer shadertoy emulator

### Installation:
With cargo (rust's package manager):
`cargo install shadercrab`

From source (Also needs cargo):
Clone this repo, and run `cargo build --release` to compile the program

### Usage:
If installed with cargo and it's in path:
`shadercrab "path/to/shader"`

From source:
`cargo run --release -- "path/to/shader"`

Shadercrab takes in a single file path as an argument.
This shader is then compiled and displayed to the output window.
When the R key is pressed or when the file is modified, the shader is reloaded.

The shader is according to how shadertoy handles the "main" tab, which means it needs this function:
`mainImage(out vec4 fragColor, in vec2 fragCoord)`
where
 - `fragColor` is the output color for the pixel, in sRGB space
 - `fragCoord` is the pixel coordinate, (0, 0) at the bottom left, `(window_width, window_height)` at the top right

The following constants are also defined:
 - `float iTime` is the time elapsed since the shader was (re)loaded, in seconds
 - `int iFrame` is the number of frames that have been rendered
 - `vec3 iResolution` where xy is the resolution of the window, and z is the aspect ration (y / x)
 - `vec4 iMouse` where xy are the mouse cursor position,  in pixel coords and zw the state of whether the mouse buttons are held down.
 The mouse position can be changed by dragging the mouse

# License
Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

# Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.