use json_store_rs::home_dir;
use json_store_rs::JsonStore;
use json_store_rs::JsonStoreError;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
use std::io::Write;
use std::path::PathBuf;

const DB_FILE_NAME: &str = ".trainingtracker.json";

#[derive(Serialize, Deserialize, Clone)]
struct TrainingDetails {
    name: String,
    rest_days_remaining: u8,
    default_rest_days: u8,
    done_count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
struct DBFile {
    last_run: time::OffsetDateTime,
    trainings: Vec<TrainingDetails>,
}

impl Default for DBFile {
    fn default() -> Self {
        Self {
            last_run: time::OffsetDateTime::now_local().unwrap(),
            trainings: vec![],
        }
    }
}

impl JsonStore for DBFile {
    fn db_file_path() -> PathBuf {
        let mut root_path = home_dir().unwrap_or_default();
        root_path.push(DB_FILE_NAME);
        root_path
    }
}

fn main() {
    let mut path = PathBuf::new();
    path.push(DB_FILE_NAME);
    let mut db: DBFile = DBFile::load().unwrap_or_else(|err| match err {
        JsonStoreError::FileNotFound => JsonStore::setup().unwrap(),
        JsonStoreError::PathNotValid => panic!("!PathNotValid!"),
        JsonStoreError::FilecontentNotValid => unreachable!(),
        JsonStoreError::FilecontentNotValid_CreatedBackupfile => {
            panic!("Filecontent was not valid.\nBackup file created.\nplease review")
        }
        JsonStoreError::FilecontentNotValid_CouldNotCreateBackupfile => {
            panic!("Filecontent was not valid.\nNO BACKUP FILE CREATED!")
        }
    });
    db.trainings.sort_by(|a, b| {
        if a.rest_days_remaining > b.rest_days_remaining {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    });

    let now = time::OffsetDateTime::now_local()
        .unwrap()
        .replace_time(time::macros::time!(0:00));

    println!("Last run on {:?}", db.last_run);
    println!("\nTodays date is: {:?}", now);
    let days_past = now - db.last_run;
    db.last_run = now;
    println!("The difference is: {:?} days", days_past.whole_days());

    reduce_training_rest_days_remaining_by(days_past.whole_days() as u8, &mut db.trainings);

    db.trainings
        .iter()
        .for_each(|training| println!("{:9?} - {}", training.name, training.rest_days_remaining));
    print!("\nWhat training did you do?\n(Default: None; Training name for not-listed; /* TODO: '-1' for 'did this yesterday' */)\nDue are: ");
    db.trainings
        .iter()
        .filter(|training| training.rest_days_remaining == 0)
        .for_each(|training| print!("{:?}, ", training.name));

    let _ = std::io::stdout().flush();
    let selected_training_str = std::io::stdin().lines().next().unwrap().unwrap();
    let selected_training: &mut TrainingDetails = if selected_training_str.is_empty() {
        print!("No training selected, abort...");
        return;
    } else if let Some(selected_training) = db
        .trainings
        .iter_mut()
        .find(|training| training.name == selected_training_str)
    {
        selected_training
    } else {
        print!("Invalid training {:?}, abort...", selected_training_str);
        return;
    };

    selected_training.rest_days_remaining = selected_training.default_rest_days;
    selected_training.done_count = selected_training.done_count.saturating_add(1);
    let _ = db.write();
}

fn reduce_training_rest_days_remaining_by(reduce_by: u8, trainings: &mut [TrainingDetails]) {
    trainings.iter_mut().for_each(|training| {
        training.rest_days_remaining = training.rest_days_remaining.saturating_sub(reduce_by);
    })
}
