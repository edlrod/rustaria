use serde::Deserialize;

include!(concat!(env!("OUT_DIR"), "/tiles.rs"));

#[derive(Clone, Debug)]
pub struct TileInfo {
    pub color: Option<[u8; 3]>,
    pub solid: bool,
    pub light: f32,
    pub opaque: bool,
    pub blocks_sky: bool,
}

// Tile properties loaded from the config file, indexed by [`Tile`].
#[derive(Clone, Debug)]
pub struct TileSet {
    infos: Vec<TileInfo>,
}

impl TileSet {
    // Parse tile properties from TOML source.
    pub fn from_toml(source: &str) -> Result<Self, String> {
        let file: TileConfigFile = toml::from_str(source).map_err(|e| e.to_string())?;

        let mut infos: Vec<Option<TileInfo>> = (0..Tile::COUNT).map(|_| None).collect();

        for def in file.tile {
            let tile = Tile::from_name(&def.name).ok_or_else(|| {
                format!(
                    "unknown tile '{}'; rebuild after editing tiles.toml",
                    def.name
                )
            })?;
            let slot = &mut infos[tile as usize];
            if slot.is_some() {
                return Err(format!("duplicate tile '{}'", def.name));
            }
            let color = match &def.color {
                Some(hex) => Some(parse_color(hex)?),
                None => None,
            };
            *slot = Some(TileInfo {
                color,
                solid: def.solid,
                light: def.light,
                opaque: def.opaque,
                blocks_sky: def.blocks_sky,
            });
        }

        let infos = infos
            .into_iter()
            .enumerate()
            .map(|(i, info)| {
                info.ok_or_else(|| format!("tile config is missing '{}'", Tile::ALL[i].name()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { infos })
    }

    // Properties of a tile.
    pub fn info(&self, tile: Tile) -> &TileInfo {
        &self.infos[tile as usize]
    }

    // All tile definitions, in id order.
    pub fn infos(&self) -> &[TileInfo] {
        &self.infos
    }

    // Whether a tile blocks entity movement.
    pub fn is_solid(&self, tile: Tile) -> bool {
        self.info(tile).solid
    }
}

pub const DEFAULT_PATH: &str = "assets/tiles.toml";

pub fn load(path: &str) -> Result<TileSet, String> {
    match std::fs::read_to_string(path) {
        Ok(source) => TileSet::from_toml(&source).map_err(|e| format!("{path}: {e}")),
        Err(err) => {
            panic!("could not read {path}: {err}")
        }
    }
}

#[derive(Deserialize)]
struct TileConfigFile {
    #[serde(default)]
    tile: Vec<TileDef>,
}

#[derive(Deserialize)]
struct TileDef {
    name: String,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    solid: bool,
    #[serde(default)]
    light: f32,
    #[serde(default)]
    opaque: bool,
    #[serde(default)]
    blocks_sky: bool,
}

// Parse a `"#rrggbb"` (or `"rrggbb"`) color into RGB bytes.
fn parse_color(hex: &str) -> Result<[u8; 3], String> {
    let digits = hex.strip_prefix('#').unwrap_or(hex);
    if digits.len() != 6 || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!(
            "invalid color '{hex}', expected six hex digits like \"#4ca64c\""
        ));
    }
    let byte = |range: std::ops::Range<usize>| {
        u8::from_str_radix(&digits[range], 16).expect("validated hex digits")
    };
    Ok([byte(0..2), byte(2..4), byte(4..6)])
}
