use crate::config::{Plonky2Proof, VerifyProof};
use lazy_static::lazy_static;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static! {
    pub static ref PLONKY2_HASHMAP: Arc<Mutex<HashMap<String, Plonky2Proof>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref VERIFY_QUEUE: Arc<Mutex<VecDeque<VerifyProof>>> =
        Arc::new(Mutex::new(VecDeque::new()));
}
