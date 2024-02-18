#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod background_image;
mod raw_image;
mod remapper;
mod world_map;
pub use app::App;

pub fn rect_map<'a, T>(
    width: usize,
    height: usize,
    pixel_for: impl Fn(usize, usize) -> T + 'a,
) -> impl Iterator<Item = T> + 'a {
    (0..height)
        .flat_map(move |row| (0..width).map(move |col| (col, row)))
        .map(move |(col, row)| pixel_for(col, row))
}
