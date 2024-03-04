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
use clap::*;
use serde::{Deserialize, Serialize};

pub const DEFAULT_DB_PATH: &str = "./gymtracker";

#[derive(Debug, Parser, PartialEq)]
#[clap(
    name = "gymtracker",
    version = "1.0",
    about = "A simple application to track workout's"
)]
pub struct GymtrackerArgs {
    #[clap(subcommand)]
    pub user_method: MethodType,
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum MethodType {
    /// Print workout logs for user defined.
    ReadLogs { username: String },
    /// Print a workout log for date specified.
    ReadDate { username: String, date: String },
    /// Create, or Insert workout log to database.
    Write {
        /// User's full name, i.e First\ Last
        username: String,
        /// Date of training session, i.e 00-00-0000
        date: String,
        /// Time of training session, i.e 00:00-00:00
        time: String,
        /// Weight of user in lbs, i.e 000.0
        body_weight: f32,
        /// Muscle's trained during session, i.e Back,\ Biceps
        muscle_group: String,
        /// Intensity of training session, range from 1-10
        intensity: u8,
    },
}

#[derive(Debug, Clone, Copy, View, ViewSchema, PartialEq)]
#[view(collection = WorkoutInputs, key = String, value = (String, String, f32, String, u8), name = "by-user-name")]
pub struct UserView;
impl CollectionMapReduce for UserView {
    fn map<'doc>(
        &self,
        document: CollectionDocument<WorkoutInputs>,
    ) -> ViewMapResult<'doc, Self::View> {
        document.header.emit_key_and_value(
            document.contents.username,
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
        let mut username = &mappings[0].key;
        let mut workout_info: &(String, String, f32, String, u8) = &mappings[0].value;
        for mapping in mappings.iter() {
            if &mapping.key == username {
                username = &mapping.key;
                workout_info = &mapping.value;
            }
        }
        Ok(workout_info.clone())
    }
}

#[derive(Debug, Clone, Copy, View, ViewSchema, PartialEq)]
#[view(collection = WorkoutInputs, key = String, value = (String, String, f32, String, u8), name = "by-date")]
pub struct DateView;
impl CollectionMapReduce for DateView {
    fn map<'doc>(
        &self,
        document: CollectionDocument<WorkoutInputs>,
    ) -> ViewMapResult<'doc, Self::View> {
        document.header.emit_key_and_value(
            document.contents.date,
            (
                document.contents.username,
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
        let mut date = &mappings[0].key;
        let mut workout_info: &(String, String, f32, String, u8) = &mappings[0].value;
        for mapping in mappings.iter() {
            if &mapping.key == date {
                date = &mapping.key;
                workout_info = &mapping.value;
            }
        }
        Ok(workout_info.clone())
    }
}

struct WriteInputsForCLI {
    username: String,
    date: String,         //00-00-0000
    time: String,         //00:00-00:00
    body_weight: f32,     //000.0LBS ('merica)
    muscle_group: String, //Back, Bicep
    intensity: u8,        //1-10 intensity of training (be real)
}

#[derive(Collection, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[collection(name= "workout-data", views = [UserView, DateView])]
pub struct WorkoutInputs {
    username: String,
    date: String,         //00-00-0000
    time: String,         //00:00-00:00
    body_weight: f32,     //000.0LBS ('merica)
    muscle_group: String, //Back, Bicep
    intensity: u8,        //1-10 intensity of training (be real)
}

impl WorkoutInputs {
    pub fn insert<C: Connection>(
        connection: &C,
        username: String,
        date: String,
        time: String,
        body_weight: f32,
        muscle_group: String,
        intensity: u8,
    ) -> Result<(), bonsaidb::core::Error> {
        WorkoutInputs {
            username,
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

fn insert_data(
    WriteInputsForCLI {
        username,
        date,
        time,
        body_weight,
        muscle_group,
        intensity,
    }: WriteInputsForCLI,
) -> Result<()> {
    let storage_connection =
        open_storage(&DEFAULT_DB_PATH.to_string()).expect("Failed to create new database.");
    let workout_connection = storage_connection
        .create_database::<WorkoutInputs>("workout-data", true)
        .expect("Failed to initalize storage connection.");

    WorkoutInputs::insert(
        &workout_connection,
        username,
        date,
        time,
        body_weight,
        muscle_group,
        intensity,
    )
    .expect("Failed to insert into database. Check inputs.");
    println!("Workout data has successfuly been inputed into the database.");
    Ok(())
}

fn print_all_data(username: &str) -> Result<()> {
    let storage_connection =
        open_storage(&DEFAULT_DB_PATH.to_string()).expect("Failed to create new database.");
    let workout_db = storage_connection.database::<WorkoutInputs>("workout-data")?;
    let user_data = workout_db
        .view::<UserView>()
        .with_key(username)
        .query_with_docs()?;
    for mapping in &user_data {
        let data = WorkoutInputs::document_contents(mapping.document)?;
        println!(
            "Retrieved workout tracked for user {}:

            date: {}
            time: {}
            body weight: {}
            muscle group trained: {}
            intensity of workout: {}
            ",
            data.username,
            data.date,
            data.time,
            data.body_weight,
            data.muscle_group,
            data.intensity
        );
    }
    Ok(())
}

// Need to match user
fn print_specific_day(username: &str, date: &str) -> Result<()> {
    let storage_connection =
        open_storage(&DEFAULT_DB_PATH.to_string()).expect("Failed to create new database.");
    let workout_db = storage_connection.database::<WorkoutInputs>("workout-data")?;

    let date_specific_data = workout_db
        .view::<DateView>()
        .with_key(date)
        .query_with_docs()?;
    for mapping in &date_specific_data {
        let data = WorkoutInputs::document_contents(mapping.document)?;
        if username == mapping.value.0 {
            println!(
                "Retrieved workout tracked for user {}:

            date: {}
            time: {}
            body weight: {}
            muscle group trained: {}
            intensity of workout: {}
            ",
                data.username,
                data.date,
                data.time,
                data.body_weight,
                data.muscle_group,
                data.intensity
            );
        }
    }
    Ok(())
}

use crate::MethodType::{ReadDate, ReadLogs, Write};

fn run(args: GymtrackerArgs) {
    match args.user_method {
        ReadLogs { username } => print_all_data(&username),
        ReadDate { username, date } => print_specific_day(&username, &date),
        Write {
            username,
            date,
            time,
            body_weight,
            muscle_group,
            intensity,
        } => insert_data(WriteInputsForCLI {
            username,
            date,
            time,
            body_weight,
            muscle_group,
            intensity,
        }),
    }
    .unwrap();
}

fn main() {
    let args = GymtrackerArgs::parse();

    run(args);
}
