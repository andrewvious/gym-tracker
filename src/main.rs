// A simple application to track workouts that I've done.
use anyhow::Result;
use bonsaidb::{
    core::{
        connection::{Connection, StorageConnection},
        document::{CollectionDocument, Emit},
        schema::{
            Collection, CollectionMapReduce, ReduceResult, SerializedCollection, SerializedView,
            View, ViewMapResult, ViewMappedValue, ViewSchema,
        },
    },
    local::{
        config::{Builder, StorageConfiguration},
        Storage,
    },
};

use serde::{Deserialize, Serialize};

pub const DEFAULT_DB_PATH: &str = "./gymtracker";

#[derive(Debug, Clone, Copy, View, ViewSchema, PartialEq)]
#[view(collection = WorkoutInputs, key = String, value = (String, String, f32, String, u8), name = "by-name")]
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
        let mut user = &mappings[0].key;
        let mut workout_info: &(String, String, f32, String, u8) = &mappings[0].value;
        for mapping in mappings.iter() {
            if &mapping.key == user {
                user = &mapping.key;
                workout_info = &mapping.value;
            }
        }
        Ok(workout_info.clone())
    }
}

#[derive(Collection, Serialize, Deserialize, Clone, Debug)]
#[collection(name= "workout-data", views = [UserView])]
pub struct WorkoutInputs {
    name: String,
    date: String,         //00-00-0000
    time: String,         //00:00-00:00
    body_weight: f32,     //000.0LBS ('merica)
    muscle_group: String, //Back, Bicep
    intensity: u8,        //1-10 intensity of training (be real)
}

impl WorkoutInputs {
    pub fn insert<C: Connection>(
        connection: &C,
        name: String,
        date: String,
        time: String,
        body_weight: f32,
        muscle_group: String,
        intensity: u8,
    ) -> Result<(), bonsaidb::core::Error> {
        WorkoutInputs {
            name,
            date,
            time,
            body_weight,
            muscle_group,
            intensity,
        }
        .push_into(connection)?;
        Ok(())
    }
}

fn open_storage(path: &String) -> Result<Storage> {
    Ok(Storage::open(
        StorageConfiguration::new(path).with_schema::<WorkoutInputs>()?,
    )?)
}

//Going to update this to reflect the method I used in the `print_more` test fn.
#[allow(dead_code)]
fn get_workout_data(
    storage_connection: &Storage,
) -> Result<(String, (String, String, f32, String, u8))> {
    let workout_db = storage_connection.database::<WorkoutInputs>("workout-data")?;

    let workout_view = UserView::entries(&workout_db).ascending().query()?;

    let workout_doc = workout_view
        .first() //this is where I need to get full scope of all data inserted
        .expect("Found empty data for user inputed, insert data and try again.");

    let name = &workout_doc.key;
    let (date, time, body_weight, muscle_group, intensity) = &workout_doc.value;
    Ok((
        name.to_string(),
        (
            date.to_string(),
            time.to_string(),
            *body_weight,
            muscle_group.to_string(),
            *intensity,
        ),
    ))
}

fn main() -> Result<()> {
    let storage_connection =
        open_storage(&DEFAULT_DB_PATH.to_string()).expect("Failed to create new database.");
    let workout_connection =
        storage_connection.create_database::<WorkoutInputs>("workout-data", true)?;

    WorkoutInputs::insert(
        &workout_connection,
        "Andrew O".to_string(),
        "2-20-2024".to_string(),
        "19:30-22:30".to_string(),
        138.0,
        "Back, Arms".to_string(),
        8,
    )?;

    Ok(())
}

#[test]
fn print_more() -> Result<()> {
    let storage_connection =
        open_storage(&DEFAULT_DB_PATH.to_string()).expect("Failed to create new database.");
    let workout_db = storage_connection.database::<WorkoutInputs>("workout-data")?;
    let user_data = workout_db
        .view::<UserView>()
        .with_key("Andrew O")
        .query_with_docs()?;
    for mapping in &user_data {
        let data = WorkoutInputs::document_contents(mapping.document)?;
        println!(
            "Retrieved workout tracked for {}: 

            \"date: {}\" 
            \"time: {}\" 
            \"body weight: {}\" 
            \"muscle group trained: {}\" 
            \"intensity of workout: {}\"
            ",
            data.name, data.date, data.time, data.body_weight, data.muscle_group, data.intensity
        );
    }
    Ok(())
}
