use std::{env, fs, str};

use bevy::{asset::RenderAssetUsages, prelude::*};
use image::ImageBuffer;
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
	}
}

#[derive(Resource)]
struct DisplayImage(pub Handle<Image>);

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, cpu: Res<Chip8CPU>) {
	commands.spawn(Camera2d);
	let img_data = render_image(
		cpu.0.display.clone(),
		cpu.0.high_res,
		LinearRgba::BLACK,
		LinearRgba::BLUE,
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
		LinearRgba::BLACK,
		LinearRgba::BLUE,
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
	while !cpu.0.need_draw {
		cpu.0.tick();
	}
}

fn chip_input(mut cpu: ResMut<Chip8CPU>) {}

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
