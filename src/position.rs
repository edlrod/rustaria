/// Continuous world position (camera, entities).
pub type Position = (f32, f32);

/// Integer tile grid coordinate.
pub type TilePosition = (i32, i32);

/// Integer chunk grid coordinate: `TilePosition / CHUNK_SIZE`.
pub type ChunkPosition = (i32, i32);
