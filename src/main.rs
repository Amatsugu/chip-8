pub mod chip8;
pub mod chip8_display;
pub mod tests;

use bevy::{
	image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor},
	prelude::*,
	window::PresentMode,
};
use chip8_display::Chip8Plugin;

fn main() {
	// let mut emu = Chip8Display::new(bytes);
	App::new()
		.add_plugins((
			DefaultPlugins
				.set(WindowPlugin {
					primary_window: Some(Window {
						title: "Chip 8".into(),
						name: Some("Chip8".into()),
						#[cfg(debug_assertions)]
						resolution: (640., 320.).into(),
						present_mode: PresentMode::AutoNoVsync,
						..default()
					}),
					..default()
				})
				.set(ImagePlugin {
					default_sampler: ImageSamplerDescriptor {
						address_mode_u: ImageAddressMode::Repeat,
						address_mode_v: ImageAddressMode::Repeat,
						mag_filter: ImageFilterMode::Nearest,
						..default()
					},
				})
				.set(AssetPlugin {
					// #[cfg(not(debug_assertions))]
					watch_for_changes_override: Some(true),
					..Default::default()
				}),
			Chip8Plugin,
		))
		.run();
}
