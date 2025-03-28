use std::{env, fs};

use bevy::{asset::RenderAssetUsages, prelude::*};
use image::ImageBuffer;
use iyes_perf_ui::{PerfUiPlugin, prelude::PerfUiEntryFPS};
use rayon::prelude::*;

use crate::chip8::Chip8;

pub struct Chip8Plugin;

#[derive(Resource)]
pub struct Chip8CPU(pub Chip8);

impl Plugin for Chip8Plugin {
	fn build(&self, app: &mut bevy::app::App) {
		let args: Vec<String> = env::args().collect();
		if args.len() < 2 {
			println!("No file provided");
			return;
		}
		let path = &args[1];
		let file = fs::read(path);

		let bytes = file.expect("Failed to read file");

		let mut cpu = Chip8::new();
		cpu.load_code(bytes);

		app.insert_resource(Chip8CPU(cpu));
		app.add_systems(Startup, setup);
		app.add_systems(Update, (chip_input, chip_tick, chip_render).chain());

		#[cfg(debug_assertions)]
		{
			use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
			app.add_plugins(FrameTimeDiagnosticsPlugin::default())
				.add_plugins(PerfUiPlugin);
		}
	}
}

#[derive(Resource)]
struct DisplayImage(pub Handle<Image>);

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, cpu: Res<Chip8CPU>) {
	#[cfg(debug_assertions)]
	commands.spawn(PerfUiEntryFPS::default());
	commands.spawn(Camera2d);
	let img_data = render_image(
		cpu.0.display.clone(),
		cpu.0.high_res,
		LinearRgba::BLACK,
		LinearRgba::BLACK,
	);
	let handle = images.add(Image::from_dynamic(
		img_data.into(),
		true,
		RenderAssetUsages::RENDER_WORLD,
	));
	let mut sprite = Sprite::from_image(handle.clone());
	sprite.custom_size = Some(Vec2::new(640., 320.));
	commands.spawn(sprite);
	commands.insert_resource(DisplayImage(handle));
}

fn chip_render(mut cpu: ResMut<Chip8CPU>, mut images: ResMut<Assets<Image>>, img: Res<DisplayImage>) {
	let img_data = render_image(
		cpu.0.display.clone(),
		cpu.0.high_res,
		LinearRgba::rgb(89. / 255., 0., 36. / 255.),
		LinearRgba::rgb(255. / 255., 0., 100. / 255.),
	);
	images.insert(
		img.0.id(),
		Image::from_dynamic(img_data.into(), true, RenderAssetUsages::RENDER_WORLD),
	);
	cpu.0.need_draw = false;
}

fn chip_tick(mut cpu: ResMut<Chip8CPU>) {
	if cpu.0.is_halted {
		return;
	}
	cpu.0.run(4);
}

fn chip_input(mut cpu: ResMut<Chip8CPU>, key: Res<ButtonInput<KeyCode>>) {
	cpu.0.set_key(0x1, key.pressed(KeyCode::Digit1));
	cpu.0.set_key(0x2, key.pressed(KeyCode::Digit2));
	cpu.0.set_key(0x3, key.pressed(KeyCode::Digit3));
	cpu.0.set_key(0xC, key.pressed(KeyCode::Digit4));

	cpu.0.set_key(0x4, key.pressed(KeyCode::KeyQ));
	cpu.0.set_key(0x5, key.pressed(KeyCode::KeyW));
	cpu.0.set_key(0x6, key.pressed(KeyCode::KeyE));
	cpu.0.set_key(0xD, key.pressed(KeyCode::KeyR));

	cpu.0.set_key(0x7, key.pressed(KeyCode::KeyA));
	cpu.0.set_key(0x8, key.pressed(KeyCode::KeyS));
	cpu.0.set_key(0x9, key.pressed(KeyCode::KeyD));
	cpu.0.set_key(0xE, key.pressed(KeyCode::KeyF));

	cpu.0.set_key(0xA, key.pressed(KeyCode::KeyZ));
	cpu.0.set_key(0x0, key.pressed(KeyCode::KeyX));
	cpu.0.set_key(0xB, key.pressed(KeyCode::KeyC));
	cpu.0.set_key(0xF, key.pressed(KeyCode::KeyV));
}

pub fn render_image(
	data: [u128; 64],
	high_res: bool,
	color1: LinearRgba,
	color2: LinearRgba,
) -> ImageBuffer<image::Rgba<u8>, Vec<u8>> {
	let mut image = ImageBuffer::new(128, 64);

	image.par_enumerate_pixels_mut().for_each(|(x, y, pixel)| {
		if !high_res {
			let line = &data[(y / 2) as usize] >> 64;
			let mask = (1 as u64).rotate_left(63 - (x / 2));
			let col = if line as u64 & mask == 0 { color1 } else { color2 };
			*pixel = to_pixel(&col);
		} else {
			let line = &data[y as usize];
			let mask = (1 as u128).rotate_left(128 - x);
			let col = if line & mask == 0 { color1 } else { color2 };
			*pixel = to_pixel(&col);
		}
	});
	return image;
}

fn to_pixel(col: &LinearRgba) -> image::Rgba<u8> {
	return image::Rgba([
		(col.red * 255.0) as u8,
		(col.green * 255.0) as u8,
		(col.blue * 255.0) as u8,
		255,
	]);
}
