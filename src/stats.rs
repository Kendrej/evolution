use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const STATS_INTERVAL: u64 = 1000;
const LATEST_DIR: &str = "runs/latest";
const TRAINING_DIR: &str = "runs/training";
const COMPARISONS_DIR: &str = "runs/comparisons";

#[derive(Clone, Default)]
struct TribeStats{
    total_births: u32,
    total_deaths: u32,
    total_respawns: u32,
    total_kills: u32,
    kills: u32,
    attacks: u32,
    hits: u32,
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
    pub fn new(seed: u32, num_tribes: usize, mode: Option<&str>, is_comparison: bool) -> Stats{
        Stats{
            seed,
            step: 0,
            tribes: vec![TribeStats::default(); num_tribes],
            file: mode.and_then(|mode| Stats::create_file(seed, mode, is_comparison))
        }
    }

    fn create_file(seed: u32, mode: &str, is_comparison: bool) -> Option<BufWriter<File>>{
        if !is_comparison{
            if let Err(e) = archive_runs(Path::new(LATEST_DIR), Path::new(TRAINING_DIR)){
                eprintln!("Nie udalo sie przeniesc poprzedniego przebiegu: {}", e);
            }
        }

        let dir = if is_comparison { COMPARISONS_DIR } else { LATEST_DIR };
        let time = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_secs());
        let path = format!("{}/run_{}_{}_seed_{}_{}.csv", dir, time, std::process::id(), seed, mode);

        let result = fs::create_dir_all(dir)
            .and_then(|_| File::create(&path))
            .and_then(|file| {
                let mut writer = BufWriter::new(file);
                writeln!(writer, "step,tribe,alive,births,deaths,respawns,avg_lifetime,food_eaten,owned_tiles,in_base,ready_in_base,ready,kills,attacks,hits")?;
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

    pub fn add_kill(&mut self, tribe: usize){
        self.tribes[tribe].total_kills += 1;
        self.tribes[tribe].kills += 1;
    }

    pub fn add_attack(&mut self, tribe: usize){
        self.tribes[tribe].attacks += 1;
    }

    pub fn add_hit(&mut self, tribe: usize){
        self.tribes[tribe].hits += 1;
    }

    pub fn add_food(&mut self, tribe: usize){
        self.tribes[tribe].food_eaten += 1;
    }

    pub fn tick(&mut self) -> bool{
        self.step += 1;
        self.step % STATS_INTERVAL == 0
    }

    pub fn write_row(&mut self, tribe: usize, alive: usize, owned_tiles: usize, in_base: usize, ready_in_base: usize, ready: usize){
        let t = &mut self.tribes[tribe];
        let avg_lifetime = if t.deaths > 0 {
            t.lifetime_sum as f32 / t.deaths as f32
        } else {
            0.0
        };

        if let Some(file) = &mut self.file{
            let row = format!("{},{},{},{},{},{},{:.1},{},{},{},{},{},{},{},{}", self.step, tribe, alive, t.births, t.deaths, t.respawns, avg_lifetime, t.food_eaten, owned_tiles, in_base, ready_in_base, ready, t.kills, t.attacks, t.hits);
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
        t.kills = 0;
        t.attacks = 0;
        t.hits = 0;
    }

    pub fn step(&self) -> u64{
        self.step
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

    pub fn total_kills(&self, tribe: usize) -> u32{
        self.tribes[tribe].total_kills
    }
}

fn has_no_measurements(path: &Path) -> bool{
    path.extension().is_some_and(|extension| extension == "csv")
        && fs::read_to_string(path).is_ok_and(|text| text.lines().count() <= 1)
}

fn archive_runs(from: &Path, to: &Path) -> io::Result<()>{
    if !from.exists(){
        return Ok(());
    }

    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)?{
        let entry = entry?;
        let path = entry.path();
        if has_no_measurements(&path){
            fs::remove_file(&path)?;
        }
        else{
            fs::rename(&path, to.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn archive_moves_runs_and_drops_files_without_measurements(){
        let root = std::env::temp_dir().join(format!("ewolucja_test_{}_archive", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let (from, to) = (root.join("latest"), root.join("training"));
        fs::create_dir_all(&from).unwrap();
        fs::write(from.join("run.csv"), "step\n1000\n").unwrap();
        fs::write(from.join("run.png"), "wykres").unwrap();
        fs::write(from.join("empty.csv"), "step\n").unwrap();

        archive_runs(&from, &to).unwrap();

        assert!(to.join("run.csv").exists());
        assert!(to.join("run.png").exists());
        assert!(!to.join("empty.csv").exists());
        assert_eq!(fs::read_dir(&from).unwrap().count(), 0);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn archive_without_a_previous_run_does_nothing(){
        let root = std::env::temp_dir().join(format!("ewolucja_test_{}_no_archive", std::process::id()));
        assert!(archive_runs(&root.join("latest"), &root.join("training")).is_ok());
        assert!(!root.exists());
    }
}
