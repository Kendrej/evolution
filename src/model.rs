use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::brain::WEIGHT_COUNT;
use crate::tribe::TRIBE_COLORS;

const MODELS_DIR: &str = "models";
const COMPARISONS_DIR: &str = "comparisons";
const MAIN_FILE: &str = "main.json";
const LATEST_FILE: &str = "latest.json";
const FINAL_FILE: &str = "final.json";
const NAME_FILE: &str = "name.txt";

// Zapisana populacja: wagi wszystkich zywych agentow, osobno dla kazdego plemienia.
// Nazwe i zrodlo ma tylko glowny model, bo tylko on trafia do gita bez swojego folderu.
#[derive(Clone, Serialize, Deserialize)]
pub struct Model{
    pub seed: u32,
    pub step: u64,
    pub weight_count: usize,
    pub tribes: Vec<Vec<Vec<f32>>>,
    #[serde(default)]
    pub parents: Vec<Vec<Vec<f32>>>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub source: String
}

impl Model{
    fn save(&self, path: &Path) -> io::Result<()>{
        if let Some(parent) = path.parent(){
            fs::create_dir_all(parent)?;
        }
        // Najpierw plik tymczasowy, zeby przerwany zapis nie zepsul poprzedniego modelu
        let temporary = path.with_extension("tmp");
        fs::write(&temporary, serde_json::to_string(self)?)?;
        fs::rename(&temporary, path)
    }

    pub fn load(path: &Path) -> Result<Model, String>{
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let model: Model = serde_json::from_str(&text).map_err(|e| e.to_string())?;

        if model.tribes.len() != TRIBE_COLORS.len(){
            return Err(format!("Zla liczba plemion: {}", model.tribes.len()));
        }
        if model.weight_count != WEIGHT_COUNT
            || model.tribes.iter().chain(&model.parents).flatten().any(|weights| weights.len() != WEIGHT_COUNT){
            return Err(format!("Model pasuje do innej sieci, oczekiwano {} wag", WEIGHT_COUNT));
        }
        Ok(model)
    }
}

pub struct ModelEntry{
    pub path: PathBuf,
    pub id: String,
    pub name: String,
    pub starred: bool,
    pub step: Option<u64>
}

pub struct ModelStore{
    root: PathBuf
}

impl ModelStore{
    pub fn new() -> ModelStore{
        ModelStore{
            root: PathBuf::from(MODELS_DIR)
        }
    }

    fn main_path(&self) -> PathBuf{
        self.root.join(MAIN_FILE)
    }

    fn run_id(seed: u32, mode: &str) -> String{
        let time = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_secs());
        format!("{}_seed_{}_{}", time, seed, mode)
    }

    // Glowny model na poczatku, potem populacje od najnowszej
    pub fn list(&self) -> Vec<ModelEntry>{
        let main = Model::load(&self.main_path()).ok();
        let main_source = main.as_ref().map(|m| m.source.as_str());

        let mut ids: Vec<String> = fs::read_dir(&self.root)
            .map(|entries| entries.flatten()
                .filter(|entry| entry.path().is_dir())
                .filter_map(|entry| entry.file_name().into_string().ok())
                .filter(|name| name != COMPARISONS_DIR)
                .collect())
            .unwrap_or_default();
        ids.sort();
        ids.reverse();

        let mut entries = Vec::new();
        for id in ids{
            let dir = self.root.join(&id);
            if let Some(path) = newest_model(&dir){
                let name = fs::read_to_string(dir.join(NAME_FILE)).unwrap_or_else(|_| id.clone());
                let starred = main_source == Some(id.as_str());
                let step = Model::load(&path).ok().map(|model| model.step);
                entries.push(ModelEntry{ path, id, name, starred, step });
            }
        }

        // Na innym komputerze moze byc sam glowny model z gita, bez folderu populacji
        if let Some(main) = &main{
            if !entries.iter().any(|entry| entry.starred){
                entries.push(ModelEntry{
                    path: self.main_path(),
                    id: main.source.clone(),
                    name: main.name.clone(),
                    starred: true,
                    step: Some(main.step)
                });
            }
        }

        entries.sort_by_key(|entry| !entry.starred);
        entries
    }

    fn ensure_folder(&self, entry: &ModelEntry) -> Result<(), String>{
        let dir = self.root.join(&entry.id);
        if entry.path == self.main_path(){
            Model::load(&entry.path)?.save(&dir.join(LATEST_FILE)).map_err(|e| e.to_string())?;
            fs::write(dir.join(NAME_FILE), &entry.name).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn set_main(&self, entry: &ModelEntry) -> Result<(), String>{
        let mut model = Model::load(&entry.path)?;
        model.name = entry.name.clone();
        model.source = entry.id.clone();
        model.save(&self.main_path()).map_err(|e| e.to_string())
    }

    pub fn clear_main(&self, entry: &ModelEntry) -> Result<(), String>{
        self.ensure_folder(entry)?;
        fs::remove_file(self.main_path()).map_err(|e| e.to_string())
    }

    pub fn rename(&self, entry: &ModelEntry, name: &str) -> Result<(), String>{
        self.ensure_folder(entry)?;
        fs::write(self.root.join(&entry.id).join(NAME_FILE), name).map_err(|e| e.to_string())?;

        if entry.starred{
            let mut model = Model::load(&self.main_path())?;
            model.name = name.to_string();
            model.save(&self.main_path()).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn new_population(&self, seed: u32, mode: &str, name: &str) -> ModelSaver{
        let id = ModelStore::run_id(seed, mode);
        let dir = self.root.join(&id);
        let result = fs::create_dir_all(&dir).and_then(|_| fs::write(dir.join(NAME_FILE), name));
        if let Err(e) = result{
            eprintln!("Nie udalo sie zapisac nazwy populacji: {}", e);
        }
        self.saver(dir, id, name)
    }

    pub fn resume_population(&self, id: &str, name: &str) -> ModelSaver{
        self.saver(self.root.join(id), id.to_string(), name)
    }

    fn saver(&self, dir: PathBuf, id: String, name: &str) -> ModelSaver{
        ModelSaver{
            dir,
            id,
            name: name.to_string(),
            main_path: Some(self.main_path())
        }
    }

    // Przebiegi porownawcze nie pojawiaja sie w menu i nigdy nie zmieniaja glownego modelu
    pub fn comparison(&self, seed: u32, mode: &str) -> ModelSaver{
        let id = ModelStore::run_id(seed, mode);
        ModelSaver{
            dir: self.root.join(COMPARISONS_DIR).join(&id),
            id,
            name: String::new(),
            main_path: None
        }
    }
}

// Wielkosc liter sie nie liczy, zeby na liscie nie bylo prawie takich samych nazw.
// Populacja zmieniajaca nazwe podaje swoje id, bo nie koliduje sama ze soba.
pub fn is_name_taken(entries: &[ModelEntry], name: &str, own_id: Option<&str>) -> bool{
    entries.iter().any(|entry| Some(entry.id.as_str()) != own_id && entry.name.trim().eq_ignore_ascii_case(name.trim()))
}

// Liczba kroków w czytelnej postaci, np. 1 234 567
pub fn format_steps(steps: u64) -> String{
    let digits = steps.to_string();
    let mut result = String::new();
    for (index, digit) in digits.chars().enumerate(){
        if index > 0 && (digits.len() - index) % 3 == 0{
            result.push(' ');
        }
        result.push(digit);
    }
    result
}

fn newest_model(dir: &Path) -> Option<PathBuf>{
    [LATEST_FILE, FINAL_FILE].iter()
        .map(|file_name| dir.join(file_name))
        .filter_map(|path| {
            let modified = fs::metadata(&path).and_then(|m| m.modified()).ok()?;
            Some((modified, path))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

// Zapisuje model jednej populacji, a jesli jest ona glowna, takze main.json
pub struct ModelSaver{
    dir: PathBuf,
    id: String,
    name: String,
    main_path: Option<PathBuf>
}

impl ModelSaver{
    pub fn save_latest(&self, model: &Model){
        self.save(model, LATEST_FILE);
    }

    pub fn save_final(&self, model: &Model){
        self.save(model, FINAL_FILE);
    }

    fn save(&self, model: &Model, file_name: &str){
        let mut result = model.save(&self.dir.join(file_name));

        if let Some(main_path) = &self.main_path{
            let is_main = Model::load(main_path).is_ok_and(|main| main.source == self.id);
            if is_main{
                let mut main = model.clone();
                main.name = self.name.clone();
                main.source = self.id.clone();
                result = result.and(main.save(main_path));
            }
        }

        if let Err(e) = result{
            eprintln!("Nie udalo sie zapisac modelu: {}", e);
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    fn test_store(name: &str) -> ModelStore{
        let root = std::env::temp_dir().join(format!("ewolucja_test_{}_{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&root);
        ModelStore{ root }
    }

    fn sample_model(step: u64) -> Model{
        Model{
            seed: 7,
            step,
            weight_count: WEIGHT_COUNT,
            tribes: vec![vec![vec![0.5; WEIGHT_COUNT]], Vec::new(), Vec::new(), Vec::new()],
            parents: vec![vec![vec![0.25; WEIGHT_COUNT]], Vec::new(), Vec::new(), Vec::new()],
            name: String::new(),
            source: String::new()
        }
    }

    fn entry_named<'a>(entries: &'a [ModelEntry], name: &str) -> &'a ModelEntry{
        entries.iter().find(|entry| entry.name == name).unwrap()
    }

    #[test]
    fn saved_model_loads_with_the_same_weights(){
        let store = test_store("round_trip");
        let path = store.root.join("model.json");
        sample_model(1000).save(&path).unwrap();
        let loaded = Model::load(&path).unwrap();

        assert_eq!(loaded.step, 1000);
        assert_eq!(loaded.tribes[0][0], vec![0.5; WEIGHT_COUNT]);
        assert_eq!(loaded.parents[0][0], vec![0.25; WEIGHT_COUNT]);
        fs::remove_dir_all(&store.root).unwrap();
    }

    #[test]
    fn model_for_a_different_network_is_rejected(){
        let store = test_store("wrong_size");
        let path = store.root.join("model.json");
        let mut model = sample_model(1);
        model.tribes[0][0].pop();
        model.save(&path).unwrap();

        assert!(Model::load(&path).is_err());
        fs::remove_dir_all(&store.root).unwrap();
    }

    #[test]
    fn parents_for_a_different_network_are_rejected(){
        let store = test_store("wrong_parent_size");
        let path = store.root.join("model.json");
        let mut model = sample_model(1);
        model.parents[0][0].pop();
        model.save(&path).unwrap();

        assert!(Model::load(&path).is_err());
        fs::remove_dir_all(&store.root).unwrap();
    }

    #[test]
    fn steps_are_grouped_by_thousands(){
        assert_eq!(format_steps(0), "0");
        assert_eq!(format_steps(999), "999");
        assert_eq!(format_steps(1000), "1 000");
        assert_eq!(format_steps(1234567), "1 234 567");
    }

    #[test]
    fn list_shows_how_many_steps_a_model_has(){
        let store = test_store("steps");
        store.new_population(1, "inherit", "Pierwsza").save_latest(&sample_model(4321));

        assert_eq!(store.list()[0].step, Some(4321));
        fs::remove_dir_all(&store.root).unwrap();
    }

    #[test]
    fn population_keeps_its_name_after_renaming(){
        let store = test_store("rename");
        store.new_population(1, "inherit", "Pierwsza").save_latest(&sample_model(1));

        let entries = store.list();
        store.rename(entry_named(&entries, "Pierwsza"), "Druga").unwrap();

        assert_eq!(store.list()[0].name, "Druga");
        fs::remove_dir_all(&store.root).unwrap();
    }

    #[test]
    fn name_of_another_population_is_taken(){
        let store = test_store("taken");
        store.new_population(1, "inherit", "Pierwsza").save_latest(&sample_model(1));
        let entries = store.list();

        assert!(is_name_taken(&entries, "Pierwsza", None));
        assert!(is_name_taken(&entries, " pierwsza ", None));
        assert!(!is_name_taken(&entries, "Druga", None));
        fs::remove_dir_all(&store.root).unwrap();
    }

    #[test]
    fn population_can_keep_its_own_name_when_renaming(){
        let store = test_store("own_name");
        store.new_population(1, "inherit", "Pierwsza").save_latest(&sample_model(1));
        store.new_population(2, "inherit", "Druga").save_latest(&sample_model(1));
        let entries = store.list();
        let first = entry_named(&entries, "Pierwsza");

        assert!(!is_name_taken(&entries, "PIERWSZA", Some(&first.id)));
        assert!(is_name_taken(&entries, "Druga", Some(&first.id)));
        fs::remove_dir_all(&store.root).unwrap();
    }

    #[test]
    fn only_the_starred_population_updates_the_main_file(){
        let store = test_store("star");
        let first = store.new_population(1, "inherit", "Pierwsza");
        let second = store.new_population(2, "inherit", "Druga");
        first.save_latest(&sample_model(10));
        second.save_latest(&sample_model(20));

        let entries = store.list();
        store.set_main(entry_named(&entries, "Pierwsza")).unwrap();

        second.save_latest(&sample_model(200));
        assert_eq!(Model::load(&store.main_path()).unwrap().step, 10);

        first.save_latest(&sample_model(100));
        let main = Model::load(&store.main_path()).unwrap();
        assert_eq!(main.step, 100);
        assert_eq!(main.name, "Pierwsza");

        let entries = store.list();
        assert!(entries[0].starred);
        assert_eq!(entries[0].name, "Pierwsza");
        assert_eq!(entries.iter().filter(|entry| entry.starred).count(), 1);
        fs::remove_dir_all(&store.root).unwrap();
    }

    #[test]
    fn main_file_alone_is_listed_and_survives_removing_the_star(){
        let store = test_store("alone");
        let population = store.new_population(1, "inherit", "Glowna");
        population.save_latest(&sample_model(5));
        let entries = store.list();
        store.set_main(&entries[0]).unwrap();
        fs::remove_dir_all(store.root.join(&entries[0].id)).unwrap();

        let entries = store.list();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, store.main_path());
        assert_eq!(entries[0].name, "Glowna");

        store.clear_main(&entries[0]).unwrap();

        let entries = store.list();
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].starred);
        assert_eq!(entries[0].name, "Glowna");
        assert!(!store.main_path().exists());
        fs::remove_dir_all(&store.root).unwrap();
    }

    #[test]
    fn comparison_runs_stay_out_of_the_list(){
        let store = test_store("comparison");
        store.comparison(1, "inherit").save_final(&sample_model(1));

        assert!(store.list().is_empty());
        fs::remove_dir_all(&store.root).unwrap();
    }
}
