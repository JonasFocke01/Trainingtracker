use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
use std::io::Write;

const DB_FILE_NAME: &str = ".trainingtracker.json";

#[derive(Serialize, Deserialize)]
struct TrainingDetails {
    name: String,
    rest_days_remaining: u8,
    default_rest_days: u8,
    done_count: usize,
}

#[derive(Serialize, Deserialize)]
struct DBFile {
    last_run: time::OffsetDateTime,
    trainings: Vec<TrainingDetails>,
}

fn main() {
    let mut db_file: DBFile =
        serde_json::from_str(std::str::from_utf8(&std::fs::read(DB_FILE_NAME).unwrap()).unwrap())
            .unwrap();
    db_file.trainings.sort_by(|a, b| {
        if a.rest_days_remaining > b.rest_days_remaining {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    });

    let now = time::OffsetDateTime::now_local()
        .unwrap()
        .replace_time(time::macros::time!(0:00));

    println!("Last run on {:?}", db_file.last_run);
    db_file
        .trainings
        .iter()
        .for_each(|training| println!("{:9?} - {}", training.name, training.rest_days_remaining));
    println!("\nTodays date is: {:?}", now);
    let days_past = now - db_file.last_run;
    db_file.last_run = now;
    println!("The difference is: {:?} days", days_past.whole_days());
    db_file
        .trainings
        .iter()
        .for_each(|training| println!("{:9?} - {}", training.name, training.rest_days_remaining));

    reduce_training_rest_days_remaining_by(days_past.whole_days() as u8, &mut db_file.trainings);
    let _ = std::fs::write(
        DB_FILE_NAME,
        serde_json::to_string_pretty(&db_file).unwrap(),
    );
    print!("\nWhat training did you do?\n(Default: None; Training name for not-listed; /* TODO: '-1' for 'did this yesterday' */)\nDue are: ");
    db_file
        .trainings
        .iter()
        .filter(|training| training.rest_days_remaining == 0)
        .for_each(|training| print!("{:?}, ", training.name));

    let _ = std::io::stdout().flush();
    let selected_training_str = std::io::stdin().lines().next().unwrap().unwrap();
    let selected_training: &mut TrainingDetails = if selected_training_str.is_empty() {
        print!("No training selected, abort...");
        return;
    } else if let Some(selected_training) = db_file
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
    let _ = std::fs::write(
        DB_FILE_NAME,
        serde_json::to_string_pretty(&db_file).unwrap(),
    );
}

fn reduce_training_rest_days_remaining_by(reduce_by: u8, trainings: &mut [TrainingDetails]) {
    trainings.iter_mut().for_each(|training| {
        training.rest_days_remaining = training.rest_days_remaining.saturating_sub(reduce_by);
    })
}
