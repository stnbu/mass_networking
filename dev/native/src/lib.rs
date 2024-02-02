use bevy::prelude::*;
use bevy::window::WindowResolution;
use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone)]
enum Side {
    Left,
    Right,
}

#[derive(Clone)]
struct Resolution(u32, u32);

impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{},{}", self.0, self.1)
    }
}

fn parse_resolution(
    s: &str,
) -> Result<Resolution, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let pos = s
        .find(',')
        .ok_or_else(|| format!("Resolution not in \"<width>,<height>\" format"))?;
    Ok(Resolution(s[..pos].parse()?, s[pos + 1..].parse()?))
}

#[derive(Parser, Resource, Clone)]
struct Args {
    #[arg(
        long,
        short,
        value_enum,
        help = "Which side of the screen to place the window",
        value_name = "LEFT|RIGHT",
        default_value_t = Side::Left
    )]
    pub side: Side,
    #[arg(
        long,
        short,
        value_parser = parse_resolution,
        help = "Your display's resolution",
        value_name = "WIDTH,HEIGHT",
        default_value_t = Resolution(3456, 2234))]
    pub resolution: Resolution,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            side: Side::Left,
            resolution: Resolution(3456, 2234),
        }
    }
}

pub struct SizedWindowPlugin;

impl Plugin for SizedWindowPlugin {
    fn build(&self, app: &mut App) {
        let args = Args::parse();
        // FIXME fussing with resolution
        let (width, height) = (
            args.resolution.0 as f32 / 2.5,
            args.resolution.1 as f32 / 2.5,
        );
        let x_position = match args.side {
            Side::Left => 0,
            Side::Right => width as i32,
        };
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(width, height).with_scale_factor_override(1.0),
                position: WindowPosition::new(IVec2::new(x_position, 0)),
                ..default()
            }),
            ..default()
        }));
    }
}
