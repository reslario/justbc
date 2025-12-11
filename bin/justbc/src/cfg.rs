use {
    input::binds::Bindings,
    serde::{
        de::{self, IntoDeserializer},
        Deserialize,
        Serialize,
    },
    std::{fs, path::Path},
    structopt::StructOpt,
    structopt_toml::StructOptToml,
    tui::style::Color,
};

#[derive(StructOpt, StructOptToml, Serialize, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    #[structopt(flatten)]
    pub state: StateConfig,
    #[serde(default)]
    #[structopt(flatten)]
    pub gfx: Graphics,
}

impl Config {
    pub fn load(cfg_file: impl AsRef<Path>) -> crate::Result<Config> {
        if cfg_file.as_ref().exists() {
            let cfg = fs::read_to_string(&cfg_file)?;
            Config::from_args_with_toml(&cfg).map_err(<_>::into)
        } else {
            Config::from_args_safe().map_err(<_>::into)
        }
    }

    pub fn save(&self, cfg_file: impl AsRef<Path>) -> crate::Result {
        use std::io::Write;

        if let Some(dir) = cfg_file.as_ref().parent() {
            fs::create_dir_all(dir)?
        }

        let mut file = fs::File::create(cfg_file)?;
        let toml = toml::to_string_pretty(self)?;
        file.write_all(toml.as_ref())?;

        Ok(())
    }
}

#[derive(StructOpt, StructOptToml, Serialize, Deserialize)]
pub struct StateConfig {
    #[serde(default)]
    #[structopt(flatten)]
    pub general: General,
    #[structopt(skip)]
    pub bindings: Option<Bindings>,
}

#[derive(StructOpt, StructOptToml, Serialize, Deserialize)]
pub struct General {
    /// The initial playback volume, from 0 to 1
    #[structopt(long, default_value = "0.5")]
    pub volume: f32,
}

#[derive(StructOpt, StructOptToml, Serialize, Deserialize)]
pub struct Graphics {
    /// The interval between updates, in milliseconds
    #[structopt(long, default_value = "33")]
    pub(super) refresh: u64,
    /// The accent colour to use for highlighting
    #[serde(with = "ColorDef")]
    #[structopt(
        long,
        default_value = "cyan",
        parse(try_from_str = parse_color)
    )]
    pub accent: Color,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Color", rename_all = "kebab-case")]
enum ColorDef {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb(u8, u8, u8),
    Indexed(u8),
}

fn parse_color(s: &str) -> Result<Color, de::value::Error> {
    ColorDef::deserialize(s.into_deserializer())
}
