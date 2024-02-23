// A simple application to track workouts that I've done.
use anyhow::Result;
use bonsaidb::{
    core::{
        connection::StorageConnection,
        document::{CollectionDocument, Emit},
        schema::{
            Collection, CollectionMapReduce, ReduceResult, View, ViewMapResult, ViewMappedValue,
            ViewSchema,
        },
    },
    local::{
        config::{Builder, StorageConfiguration},
        Storage,
    },
};

use serde::{Deserialize, Serialize};

pub const DEFAULT_DB_PATH: &str = "./gymtracker";

#[derive(Debug, Clone, View, ViewSchema, PartialEq)]
#[view(collection = WorkoutInputs, key = String, value = (String, String, u8, String, u8), name = "by-name")]
pub struct UserView;
impl CollectionMapReduce for UserView {
    fn map<'doc>(
        &self,
        document: CollectionDocument<WorkoutInputs>,
    ) -> ViewMapResult<'doc, Self::View> {
        document.header.emit_key_and_value(
            document.contents.name,
            (
                document.contents.date,
                document.contents.time,
                document.contents.body_weight,
                document.contents.muscle_group,
                document.contents.intensity,
            ),
        )
    }

    fn reduce(
        &self,
        mappings: &[ViewMappedValue<'_, Self>],
        _rereduce: bool,
    ) -> ReduceResult<Self::View> {
        let mut user = mappings[0].key;
        let mut workout_info: (String, String, u8, String, u8) = mappings[0].value;
        for mapping in mappings.iter() {
            if mapping.key == user {
                user = mapping.key;
                workout_info = mapping.value;
            }
        }
        Ok(workout_info)
    }
}

#[derive(Collection, Serialize, Deserialize, Clone, Debug)]
#[collection(name= "workout-data", views = [UserView])]
pub struct WorkoutInputs {
    name: String,
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

// fn get_data(
//     key: &UserView,
//     storage_connection: &Storage,
// ) -> Result<CollectionDocument<WorkoutInputs>> {
//     let db = storage_connection.database::<WorkoutInputs>("workout-data")?;
//     WorkoutInputs::get(&key, &db)
//         .map_err(|e| anyhow::anyhow!("failed to open document: {e:?}"))?
//         .ok_or(anyhow::anyhow!(
//             "failed to retrieve workout data for user '{:?}' at database path '{:?}'",
//             &key,
//             &storage_connection.path()
//         ))
// }

// fn insert_test_data(account_connection: &Database) -> Result<()> {
//     let key = User {
//         name: "Bryant".to_string(),
//     };
//     WorkoutInputs {
//         date: "2-22-2024".to_string(),
//         time: "7:00-9:00".to_string(),
//         body_weight: 190,
//         muscle_group: "Legs".to_string(),
//         intensity: 6,
//     }
//     .insert_into(&key, account_connection)?;
//     Ok(())
// }

fn main() -> Result<()> {
    let storage_connection =
        open_storage(&DEFAULT_DB_PATH.to_string()).expect("Failed to create new database.");
    let workout_connection =
        storage_connection.create_database::<WorkoutInputs>("workout-data", true)?;

    insert_test_data(&workout_connection)?;

    let key = User {
        name: "Andrew O".to_string(),
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
        name: "Andrew O".to_string(),
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
