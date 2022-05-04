use factorio_web_calculator::data::*;

use std::{collections::{HashMap, HashSet}, iter::Iterator, path::{PathBuf, Path}, fs::File, io::{BufWriter, Write}};
use serde_json::{Value, from_reader, to_writer, from_value, json};
use clap::Parser;
use image::{RgbaImage, ImageBuffer, io::Reader, Rgba, Pixel, GenericImageView, imageops::overlay, ImageFormat};

#[derive(Debug, Parser)]
#[clap(about, long_about = None)]
struct CliParameters {
    /// Path to Factorio installation dir
    #[clap(short, long, parse(from_os_str), value_name = "FACTORIO_DIR")]
    factorio_dir: PathBuf,
    /// Output directory
    #[clap(short, long, parse(from_os_str), value_name = "OUT")]
    output_dir: PathBuf,
    /// Input file to read data from
    #[clap(short, long, parse(from_os_str), value_name = "IN")]
    input_file: PathBuf,
    /// Use expensive recipes instead of normal
    #[clap(short, long)]
    expensive: bool
}

struct PathResolver {
    core_path: PathBuf,
    base_path: PathBuf,
    gen_path: PathBuf
}

impl PathResolver {
    fn new(core_path: PathBuf, base_path: PathBuf, out_dir: &Path) -> Self {
        Self{core_path, base_path, gen_path: out_dir.join("generated/generated-icons/")}
    }

    fn resolve(&self, name: &str) -> PathBuf {
        if name.starts_with("__core__") {
            self.core_path.join(&name[9..])
        } else if name.starts_with("__base__") {
            self.base_path.join(&name[9..])
        } else {
            self.gen_path.join(name)
        }
    }
}

struct SpriteSheet {
    sheet: RgbaImage,
    size: usize,
    pos: (usize, usize)
}

impl SpriteSheet {
    fn new(len: usize) -> Self {
        let mut i = 1;
        while (i * i) < len {
            i += 1
        }
        let image = RgbaImage::new((i * 64) as u32, (i * 64) as u32);
        Self{sheet: image, size: i * 64, pos: (0, 0)}
    }

    fn add_sprite(&mut self, image: RgbaImage) -> (usize, usize) {
        overlay(&mut self.sheet, &image, self.pos.0 as i64, self.pos.1 as i64);
        self.advance()
    }

    fn advance(&mut self) -> (usize, usize) {
        let old_pos = self.pos;
        if self.pos.0 + 64 >= self.size {
            self.pos.0 = 0;
            self.pos.1 += 64
        } else {
            self.pos.0 += 64
        }
        old_pos
    }

    fn write(&self, path: impl AsRef<Path>) {
        println!("Writing spritesheet to {}", path.as_ref().to_str().unwrap());
        let mut file = File::create(path).unwrap();
        self.sheet.write_to(&mut file, ImageFormat::Png).unwrap();
    }
}

fn main() {
    // Init //
    let params = CliParameters::parse();

    let difficulty = params.expensive.then(|| "expensive").unwrap_or("normal");
    
    if !params.factorio_dir.exists() {
        panic!("{} does not exist", params.factorio_dir.to_str().unwrap());
    }

    let core_path = params.factorio_dir.join("data/core");
    let base_path = params.factorio_dir.join("data/base");
    let out_dir = params.output_dir;
    let out_file_path = out_dir.join("generated/processed-data.json");

    let path_resolver = PathResolver::new(core_path, base_path, &out_dir);

    // Json data parse //

    println!("Parsing input data");
    let in_file = File::open(params.input_file).unwrap();
    let json_data: Value = from_reader(in_file).unwrap();

    let game_data = get_data(difficulty, &json_data);
    println!("Done parsing data, writing to {}", out_file_path.to_str().unwrap());

    let out_file = File::create(out_file_path).unwrap();
    to_writer(out_file, &game_data).unwrap();

    // Icons //

    let mut simple_icons: HashMap<String, HashSet<String>> = HashMap::new();    // map path to vec of items that use this icon
    let mut complex_icons: HashMap<String, Vec<IconData>> = HashMap::new(); // map name of item to icon data

    println!("Processing icons for items");
    game_data.items.iter().for_each(|(_, item)| {
        match &item.icon {
            Icon::Simple(icon) => {simple_icons.entry(icon.clone()).or_insert_with(HashSet::new).insert(format!("item-{}", &item.name));},
            Icon::Icons(icons) => {complex_icons.insert(format!("item-{}", &item.name), icons.clone());},
        };
    });

    println!("Processing icons for assembling machines");
    game_data.assembling_machines.iter().for_each(|(_, item)| {
        match &item.icon {
            Icon::Simple(icon) => {simple_icons.entry(icon.clone()).or_insert_with(HashSet::new).insert(format!("assembling-machine-{}", &item.name));},
            Icon::Icons(icons) => {complex_icons.insert(format!("assembling-machine-{}", &item.name), icons.clone());},
        }
    });

    println!("Processing complex icons");
    let complex_icons: HashMap<String, RgbaImage> = complex_icons.into_iter().map(|(k, icons)| generate_complex_icon(k, icons, &path_resolver)).collect();

    println!("Writing complex icons");
    for (name, icon_image) in &complex_icons {
        let mut path = path_resolver.resolve(name);
        path.set_extension("png");
        let mut file = File::create(path).unwrap();
        icon_image.write_to(&mut file, ImageFormat::Png).unwrap();
    }

    // Spritesheet //

    println!("Generating spritesheet");
    let mut simple_icons: HashMap<RgbaImage, HashSet<String>> = simple_icons.into_iter().map(|(path, items)| {
        let image = Reader::open(path_resolver.resolve(&path))
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8()
            .view(0, 0, 64, 64)
            .to_image();
        (image, items)
    }).collect();

    simple_icons
        .extend(complex_icons
            .into_iter()
            .map(|(k, v)| (v, [k].into())));

    let mut spritesheet = SpriteSheet::new(simple_icons.len());

    let icons: HashMap<(usize, usize), HashSet<String>> = simple_icons.into_iter().map(|(image, name)| {
        (spritesheet.add_sprite(image), name)
    }).collect();

    spritesheet.write(out_dir.join("generated/spritesheet.png"));

    // Mapping //

    println!("Generating mapping");
    let spritesheet_mapping = icons
        .into_iter()
        .fold(HashMap::new(), |mut mapping, (pos, names)| {
        for name in names {
            mapping.insert(name, pos);
        }
        mapping
    });

    {
        let path = out_dir.join("generated/spritesheet-mapping.json");
        println!("Writing generated mapping to {}", path.to_str().unwrap());
        let mapping_file = File::create(path).unwrap();
        to_writer(mapping_file, &spritesheet_mapping).unwrap();
    }

    // CSS mapping //
    
    {
        println!("Generating css styles");
        let mut out_file = BufWriter::new(File::create(out_dir.join("generated/icon-style.css")).unwrap());
        out_file.write_fmt(format_args!(".target-button {{ width: 64px; height: 64px; background-image: url(\"spritesheet.png\") }}")).unwrap();
        for (name, pos) in &spritesheet_mapping {
            out_file.write_fmt(format_args!(".icon-{} {{ background-position-x: -{}px; background-position-y: -{}px }}\n", name, pos.0, pos.1)).unwrap();
        }
    }
}

fn generate_complex_icon(name: String, icons: Vec<IconData>, resolver: &PathResolver) -> (String, RgbaImage) {
    let mut result = ImageBuffer::from_pixel(64, 64, [0, 0, 0, 0].into());
    for icon_data in icons {
        let icon_path = resolver.resolve(&icon_data.icon);
        let mut icon_image = Reader::open(icon_path)
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8()
            .view(0, 0, 64, 64)
            .to_image();
        icon_image.pixels_mut().map(|p| tint_pixel(p, &icon_data.tint)).for_each(drop);
        overlay(&mut result, &icon_image, 0, 0)
        
    }

    (name, result)
}

fn tint_pixel(pixel: &mut Rgba<u8>, tint: &TintColor) {
    // FIXME: barrels are broken
    let channels_a = pixel.channels_mut();
    channels_a[0] = ((channels_a[0] as f32 * (tint.r * 255.0)) / 255.0) as u8;
    channels_a[1] = ((channels_a[1] as f32 * (tint.g * 255.0)) / 255.0) as u8;
    channels_a[2] = ((channels_a[2] as f32 * (tint.b * 255.0)) / 255.0) as u8;
    channels_a[3] = ((channels_a[3] as f32 * (tint.a * 255.0)) / 255.0) as u8;
}

fn get_data(difficulty: &str, json_data: &Value) -> GameData {

    println!("Processing items");
    let items: HashMap<String, Item> = json_data["item"]
        .as_object()
        .cloned()
        .unwrap()
        .into_iter()
        .map(|(k, v)| (k, from_value(v).unwrap()))
        .collect();

    println!("Processing assembling machines");
    let assembling_machines: HashMap<String, AssemblingMachine> = json_data["assembling-machine"]
        .as_object()
        .cloned()
        .unwrap()
        .into_iter()
        .map(|(k, v)| (k, from_value(v).unwrap()))
        .collect();
    println!("Processing furnaces");
    let furnaces: HashMap<String, AssemblingMachine> = from_value(json_data["furnace"].clone()).unwrap();
    println!("Merging furnaces and assembling machines");
    let assembling_machines: HashMap<String, AssemblingMachine> = {
        assembling_machines.into_iter().chain(furnaces.into_iter()).collect()
    };

    println!("Processing item groups and subgroups");
    let item_groups: HashMap<String, ItemGroup> = from_value(json_data["item-group"].clone()).unwrap();
    let item_subgroups: HashMap<String, ItemSubGroup> = from_value(json_data["item-subgroup"].clone()).unwrap();

    println!("Processing recipes");
    let recipes: HashMap<String, Recipe> = from_value(json_data["recipe"].clone()).unwrap();

    println!("Processing mining drills");
    let mining_drills: HashMap<String, MiningDrill> = from_value(json_data["mining-drill"].clone()).unwrap();

    println!("Processing offshore pumps");
    let offshore_pumps: HashMap<String, OffshorePump> = from_value(json_data["offshore-pump"].clone()).unwrap();

    GameData{items, recipes, assembling_machines, item_groups, item_subgroups, mining_drills, offshore_pumps}
}
