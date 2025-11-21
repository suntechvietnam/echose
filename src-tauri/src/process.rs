use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::process::Child;

pub type ProcessStore = Arc<Mutex<HashMap<String, Child>>>;

