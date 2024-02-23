use serde::Deserialize;
use serde::Serialize;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
enum TrainingTodo {
    Back,
    Chest,
    Schoulders,
    Neck,
    Arms,
    Abs,
}

#[derive(Serialize, Deserialize)]
struct TrainingDetails {
    training_todo: TrainingTodo,
    rest_days_remaining: u8,
    default_rest_days: u8,
    done_count: usize
}

#[derive(Serialize, Deserialize)]
struct DBFile {
    last_run: time::OffsetDateTime,
    trainings: Vec<TrainingDetails>,
}

fn main() {
    let mut db_file: DBFile =
        serde_json::from_str(std::str::from_utf8(&std::fs::read("db.json").unwrap()).unwrap())
            .unwrap();

    let now = time::OffsetDateTime::now_local().unwrap().replace_time(time::macros::time!(0:00));

    println!("Last run on {:?}", db_file.last_run);
    db_file
        .trainings
        .iter()
        .for_each(|training| println!("{:9?} - {}", training.training_todo, training.rest_days_remaining));
    println!("\nTodays date is: {:?}", now);
    let days_past = now - db_file.last_run;
    db_file.last_run = now;
    println!("The difference is: {:?} days", days_past.whole_days());
    db_file
        .trainings
        .iter()
        .for_each(|training| println!("{:9?} - {}", training.training_todo, training.rest_days_remaining));

    reduce_training_rest_days_remaining_by(days_past.whole_days() as u8, &mut db_file.trainings);
    let _ = std::fs::write("db.json", serde_json::to_string(&db_file).unwrap());
    filter_trainings(&mut db_file.trainings);
    println!("\nWhat training did you do? (Default: None)");
    db_file
        .trainings
        .iter()
        .enumerate()
        .for_each(|(i, training)| print!("{:?} ({}), ", training.training_todo, i + 1));

    let _ = std::io::stdout().flush();
    let training_done_str = std::io::stdin().lines().next().unwrap().unwrap();
    let training_done: u8 = if training_done_str.is_empty() {
        return;
    } else {
        training_done_str.parse().unwrap()
    };

    // reduce done training
    let mut db_file: DBFile =
        serde_json::from_str(std::str::from_utf8(&std::fs::read("db.json").unwrap()).unwrap())
            .unwrap();
    let training_done = db_file
        .trainings
        .get_mut((training_done - 1) as usize)
        .unwrap();
    training_done.rest_days_remaining = training_done.default_rest_days;
    training_done.done_count = training_done.saturating_add(1);
    let _ = std::fs::write("db.json", serde_json::to_string_pretty(&db_file).unwrap());
}

fn reduce_training_rest_days_remaining_by(reduce_by: u8, trainings: &mut Vec<TrainingDetails>) {
    trainings.iter_mut().for_each(|training| {
        training.rest_days_remaining = training.rest_days_remaining.saturating_sub(reduce_by);
    })
}

fn filter_trainings(trainings: &mut Vec<TrainingDetails>) {
    trainings.retain(|training| training.rest_days_remaining == 0);
}
