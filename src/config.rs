use serde::{Deserialize, Serialize};
use std::{env, str::FromStr};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::{HashMap, VecDeque};
use log::{info, warn};
use reqwest;
use std::fs;

use plonky2::plonk::circuit_data::{CommonCircuitData, VerifierCircuitData};
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use plonky2::plonk::proof::CompressedProofWithPublicInputs;
use plonky2::util::serialization::DefaultGateSerializer;

#[derive(Serialize, Debug, Deserialize)]
pub struct VerifyProof {
    pub tx_id: String,
    pub verify_type: u8,
}

#[derive(Serialize, Debug)]
pub struct PostVerificationResult {
    pub tx_id: String,
    pub is_valid: bool,
}

pub struct Config {
    pub port: u16,
    pub workers: usize,
    pub delete_files: bool,
    pub u_port: u16,
}

impl Config {
    pub fn init() -> Self {
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .expect("PORT must be a number");
        let workers = env::var("WORKERS")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .expect("WORKERS must be a number");
        let delete_files = env::var("DELETE_FILES")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .expect("DELETE_FILES must be a boolean");
        let u_port: u16 = env::var("UPORT")
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .expect("UPort must be a number");
        Config {
            port,
            workers,
            delete_files,
            u_port,
        }
    }
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Config {
            port: self.port,
            workers: self.workers,
            delete_files: self.delete_files,
            u_port: self.u_port,
        }
    }
}


#[derive(Deserialize, Debug)]
pub struct ProofDataPlonky2 {
    pub tx_id: String,
    pub proof_file_path: String,
    pub common_data_file_path: String,
    pub verifier_data_file_path: String,
}

#[derive(Deserialize, Debug)]
pub struct Plonky2Proof {
    pub proof_file_path: String,
    pub common_data_file_path: String,
    pub verifier_data_file_path: String,
}

pub async fn verify(_proof: &Plonky2Proof) -> bool {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    let common_data_bytes = fs::read(&_proof.common_data_file_path).unwrap();
    let verifier_data_bytes = fs::read(&_proof.verifier_data_file_path).unwrap();
    let compressed_proof_bytes = fs::read(&_proof.proof_file_path).unwrap();
    let default_gate_serializer = DefaultGateSerializer;

    let common_data = CommonCircuitData::<F, D>::from_bytes(common_data_bytes[32..].to_vec(), &default_gate_serializer).unwrap();
    let compressed_proof = CompressedProofWithPublicInputs::<F, C, D>::from_bytes(compressed_proof_bytes[32..].to_vec(), &common_data).unwrap();
    let verifier_data = VerifierCircuitData::<F, C, D>::from_bytes(verifier_data_bytes[32..].to_vec(), &default_gate_serializer).unwrap();

    match verifier_data.verify_compressed(compressed_proof) {
        Ok(_) => {
            info!("Verification Successful!");
            return true;
        }
        Err(e) => {
            warn!("Verification Failed: {:?}", e);
            return false;
        }
    }
}

pub async fn process_verification_queue(
    queue: Arc<Mutex<VecDeque<VerifyProof>>>,
    _plonky2_hashmap: Arc<Mutex<HashMap<String, Plonky2Proof>>>,
) {
    loop {
        let mut queue = queue.lock().await;

        if queue.is_empty() {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            continue;
        }

        let verification_proof = queue.pop_front().unwrap();
        info!("Processing verification proof: {:?}", verification_proof);
        let plonky2_hashmap = _plonky2_hashmap.lock().await;
        let plonky2_proof = plonky2_hashmap.get(&verification_proof.tx_id).unwrap();
        let is_valid = verify(plonky2_proof).await;
        // Send POST request to the other server on successful verification
        let config = Config::init();
        let port = config.u_port;
        let url_str = format!("http://127.0.0.1:{}/submit-result", port.to_string());
        info!("Sending verification proof to: {}", url_str);
        let url = reqwest::Url::from_str(&url_str).expect("Failed to parse URL");
        let client = reqwest::Client::new();
        let map = PostVerificationResult {
            tx_id: verification_proof.tx_id,
            is_valid,
        };
        let response = client
            .post(url)
            .json(&map)
            .send()
            .await
            .expect("Failed to send POST request");
        info!("Response: {:?}", response);
        if response.status().is_success() {
            info!("Verification proof sent successfully!");
        } else {
            warn!("Failed to send verification proof: {}", response.status());
        }
    }
}

