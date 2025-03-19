// Bitmap uniform
// struct BitMap {
//     //data: array<u32, 256>, 
//     mode: u32,
// };

// @group(0) @binding(100)
// var<uniform> bitmap: BitMap;

@fragment
fn main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32> {
    let x: u32 = u32(frag_pos.x);
    let y: u32 = u32(frag_pos.y);

	return vec4(frag_pos.x/128.0, 0.0, 0.0, 0.0);
    
    // // Determine grid mode
    // let is_large = bitmap.mode == 1;
    
    // // let row = if is_large { y / 128 } else { y / 64 };
    // let row = y / 64 ;
    // let bit = x % 128;  // 128 bits per row in large mode

    // let index = row * 4;
    
    // let low1  = bitmap.data[index];
    // let low2  = bitmap.data[index + 1];
    // let high1 = bitmap.data[index + 2];
    // let high2 = bitmap.data[index + 3];

    // Reconstruct u128
    // let value = (u128(high2) << 96) | (u128(high1) << 64) | (u128(low2) << 32) | u128(low1);

    // // Extract bit
    // let mask = u128(1) << bit;
    // let is_set = (value & mask) != 0;

    // if is_set {
	// 	 return vec4<f32>(1.0, 1.0, 1.0, 1.0); 
	// }
	// else {
	// 	return vec4<f32>(0.0, 0.0, 0.0, 1.0); 
	// };
}