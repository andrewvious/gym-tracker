// A simple application to track workouts that I've done.
// Requires args in form of Strings/or custom types to be inputed via CLI, which will be stored in a database
// (possibly SQLite because it's well documented and versital)
use anyhow::{Ok, Result};
use bonsaidb::{
    core::{
        connection::StorageConnection,
        document::CollectionDocument,
        key::Key,
        schema::{Collection, SerializedCollection},
    },
    local::{
        config::{Builder, StorageConfiguration},
        Database, Storage, StorageNonBlocking,
    },
};

use serde::{Deserialize, Serialize};

pub const DEFAULT_DB_PATH: &str = "./gymtracker";

// Key to pull user's relative data.
#[derive(Key, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct User {
    pub name: String,
}

#[derive(Collection, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[collection(name= "workout-data", primary_key= User)]
pub struct WorkoutInputs {
    date: String,         //00-00-0000
    time: String,         //00:00-00:00
    body_weight: u8,      //000LBS ('merica)
    muscle_group: String, //Back, Bicep
    intensity: u8,        //1-10 intensity of training (be real)
}

fn open_storage(path: &String) -> Result<Storage> {
    Ok(Storage::open(
        StorageConfiguration::new(path).with_schema::<WorkoutInputs>()?,
    )?)
}

fn get_data(key: &User, storage_connection: &Storage) -> Result<CollectionDocument<WorkoutInputs>> {
    let db = storage_connection.database::<WorkoutInputs>("workout-data")?;
    WorkoutInputs::get(&key, &db)
        .map_err(|e| anyhow::anyhow!("failed to open document: {e:?}"))?
        .ok_or(anyhow::anyhow!(
            "failed to retrieve workout data for user '{:?}' at database path '{:?}'",
            &key,
            &storage_connection.path()
        ))
}

fn insert_test_data(account_connection: &Database) -> Result<()> {
    let key = User {
        name: "Andrewvios".to_string(),
    };
    WorkoutInputs {
        date: "2-22-2024".to_string(),
        time: "9:30-12:00".to_string(),
        body_weight: 138,
        muscle_group: "Back, Biceps, Shoulders".to_string(),
        intensity: 9,
    }
    .insert_into(&key, account_connection)?;
    Ok(())
}

fn main() -> Result<()> {
    let storage_connection =
        open_storage(&DEFAULT_DB_PATH.to_string()).expect("Failed to create new database.");
    let workout_connection =
        storage_connection.create_database::<WorkoutInputs>("workout-data", true)?;

    insert_test_data(&workout_connection)?;

    let key = User {
        name: "Andrewvious".to_string(),
    };

    let retrieved = get_data(&key, &storage_connection)?;

    print!("{:#?}", retrieved);

    Ok(())
}

#[test]
fn read_db() -> Result<()> {
    let storage_connection =
        open_storage(&DEFAULT_DB_PATH.to_string()).expect("Failed to create new database.");
    let key = User {
        name: "Andrewvious".to_string(),
    };

    let retrieved = get_data(&key, &storage_connection)?;

    print!("{:#?}", retrieved);

    Ok(())
}

#[test]
fn push_db() -> Result<()> {
    let storage_connection =
        open_storage(&DEFAULT_DB_PATH.to_string()).expect("Failed to create new database.");
    let workout_connection =
        storage_connection.create_database::<WorkoutInputs>("workout-data", true)?;

    insert_test_data(&workout_connection)?;
    Ok(())
}
