use std::{env, fs};

use bevy::{asset::RenderAssetUsages, prelude::*};
use image::ImageBuffer;
use rayon::prelude::*;

use crate::chip8::{Chip8, DISPLAY_HEIGHT_HIGHRES, DISPLAY_WIDTH, DISPLAY_WIDTH_HIGHRES};

const FPS: f32 = 60.;
pub struct Chip8Plugin;

#[derive(Resource)]
pub struct Chip8CPU(pub Chip8, pub Timer);

impl Plugin for Chip8Plugin
{
	fn build(&self, app: &mut bevy::app::App)
	{
		let args: Vec<String> = env::args().collect();
		if args.len() < 2
		{
			println!("No file provided");
			return;
		}
		let path = &args[1];
		let file = fs::read(path);

		let bytes = file.expect("Failed to read file");

		let mut cpu = Chip8::new();
		cpu.load_code(bytes);

		app.insert_resource(Chip8CPU(cpu, Timer::from_seconds(1.0 / FPS, TimerMode::Repeating)));
		app.add_systems(Startup, setup);
		app.add_systems(Update, (chip_input, chip_tick, chip_render).chain());

		// app.add_plugins(FrameTimeDiagnosticsPlugin::default());
	}
}

#[derive(Resource)]
struct DisplayImage(pub Handle<Image>);

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, cpu: Res<Chip8CPU>)
{
	// commands.spawn(DiagnosticsOverlay::fps());
	commands.spawn(Camera2d);
	let img_data = render_image(cpu.0.display, cpu.0.high_res, LinearRgba::BLACK, LinearRgba::BLACK);
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

fn chip_render(mut cpu: ResMut<Chip8CPU>, mut images: ResMut<Assets<Image>>, img: Res<DisplayImage>, time: Res<Time>)
{
	if !cpu.1.tick(time.delta()).just_finished()
	{
		return;
	}
	let img_data = render_image(
		cpu.0.display,
		cpu.0.high_res,
		LinearRgba::rgb(89. / 255., 0., 36. / 255.),
		LinearRgba::rgb(1., 0., 100. / 255.),
	);
	images
		.insert(
			img.0.id(),
			Image::from_dynamic(img_data.into(), true, RenderAssetUsages::RENDER_WORLD),
		)
		.expect("Failed to insert image");
	cpu.0.vblank();
}

fn chip_tick(mut cpu: ResMut<Chip8CPU>)
{
	if cpu.0.is_halted
	{
		return;
	}
	cpu.0.run(60);
}

fn chip_input(mut cpu: ResMut<Chip8CPU>, key: Res<ButtonInput<KeyCode>>)
{
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
) -> ImageBuffer<image::Rgba<u8>, Vec<u8>>
{
	#[cfg(feature = "tracing")]
	let _ = info_span!("Render Image").entered();
	let mut image = ImageBuffer::new(DISPLAY_WIDTH_HIGHRES as u32, DISPLAY_HEIGHT_HIGHRES as u32);

	image.par_enumerate_pixels_mut().for_each(|(x, y, pixel)| {
		if !high_res
		{
			let line = data[(y / 2) as usize] >> DISPLAY_WIDTH;
			let mask = 1_u64 << ((DISPLAY_WIDTH as u32) - 1 - (x / 2));
			let col = if line as u64 & mask == 0 { color1 } else { color2 };
			*pixel = to_pixel(&col);
		}
		else
		{
			let line = data[y as usize];
			let mask = 1_u128 << ((DISPLAY_WIDTH_HIGHRES as u32) - x);
			let col = if line & mask == 0 { color1 } else { color2 };
			*pixel = to_pixel(&col);
		}
	});
	image
}

fn to_pixel(col: &LinearRgba) -> image::Rgba<u8>
{
	image::Rgba([
		(col.red * 255.0) as u8,
		(col.green * 255.0) as u8,
		(col.blue * 255.0) as u8,
		255,
	])
}
