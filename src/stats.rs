use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::time::{SystemTime, UNIX_EPOCH};

const STATS_INTERVAL: u64 = 1000;

#[derive(Clone, Default)]
struct TribeStats{
    total_births: u32,
    total_deaths: u32,
    total_respawns: u32,
    births: u32,
    deaths: u32,
    respawns: u32,
    lifetime_sum: u64,
    food_eaten: u32
}

pub struct Stats{
    seed: u32,
    step: u64,
    tribes: Vec<TribeStats>,
    file: Option<BufWriter<File>>
}

impl Stats{
    pub fn new(seed: u32, num_tribes: usize, inheritance: bool) -> Stats{
        Stats{
            seed,
            step: 0,
            tribes: vec![TribeStats::default(); num_tribes],
            file: Stats::create_file(seed, inheritance)
        }
    }

    fn create_file(seed: u32, inheritance: bool) -> Option<BufWriter<File>>{
        let time = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_secs());
        let mode = if inheritance { "inherit" } else { "random" };
        let path = format!("runs/run_{}_{}_seed_{}_{}.csv", time, std::process::id(), seed, mode);

        let result = fs::create_dir_all("runs")
            .and_then(|_| File::create(&path))
            .and_then(|file| {
                let mut writer = BufWriter::new(file);
                writeln!(writer, "step,tribe,alive,births,deaths,respawns,avg_lifetime,food_eaten,owned_tiles")?;
                writer.flush()?;
                Ok(writer)
            });

        match result{
            Ok(writer) => {
                println!("Statystyki: {}", path);
                Some(writer)
            }
            Err(e) => {
                eprintln!("Nie udalo sie utworzyc pliku statystyk: {}", e);
                None
            }
        }
    }

    pub fn add_birth(&mut self, tribe: usize){
        self.tribes[tribe].total_births += 1;
        self.tribes[tribe].births += 1;
    }

    pub fn add_death(&mut self, tribe: usize, lifetime: u32){
        self.tribes[tribe].total_deaths += 1;
        self.tribes[tribe].deaths += 1;
        self.tribes[tribe].lifetime_sum += lifetime as u64;
    }

    pub fn add_respawn(&mut self, tribe: usize){
        self.tribes[tribe].total_respawns += 1;
        self.tribes[tribe].respawns += 1;
    }

    pub fn add_food(&mut self, tribe: usize){
        self.tribes[tribe].food_eaten += 1;
    }

    pub fn tick(&mut self) -> bool{
        self.step += 1;
        self.step % STATS_INTERVAL == 0
    }

    pub fn write_row(&mut self, tribe: usize, alive: usize, owned_tiles: usize){
        let t = &mut self.tribes[tribe];
        let avg_lifetime = if t.deaths > 0 {
            t.lifetime_sum as f32 / t.deaths as f32
        } else {
            0.0
        };

        if let Some(file) = &mut self.file{
            let row = format!("{},{},{},{},{},{},{:.1},{},{}", self.step, tribe, alive, t.births, t.deaths, t.respawns, avg_lifetime, t.food_eaten, owned_tiles);
            if let Err(e) = writeln!(file, "{}", row).and_then(|_| file.flush()){
                eprintln!("Nie udalo sie zapisac statystyk: {}", e);
                self.file = None;
            }
        }

        t.births = 0;
        t.deaths = 0;
        t.respawns = 0;
        t.lifetime_sum = 0;
        t.food_eaten = 0;
    }

    pub fn seed(&self) -> u32{
        self.seed
    }

    pub fn total_births(&self, tribe: usize) -> u32{
        self.tribes[tribe].total_births
    }

    pub fn total_deaths(&self, tribe: usize) -> u32{
        self.tribes[tribe].total_deaths
    }

    pub fn total_respawns(&self, tribe: usize) -> u32{
        self.tribes[tribe].total_respawns
    }
}
