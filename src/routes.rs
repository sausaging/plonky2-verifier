use actix_web::{Responder, HttpResponse, get, post, web};
use serde::{Deserialize, Serialize};
use crate::config::{Plonky2Proof, ProofDataPlonky2, VerifyProof};
use std::sync::Arc;
use tokio::sync::Mutex;
use log::{info, warn};
use std::collections::{HashMap, VecDeque};

#[derive(Serialize, Deserialize)]
pub struct PingSingle {
    pub success: bool,
}

#[derive(Serialize)]
pub struct SubmitionResult {
    pub is_submitted: bool,
}

#[get("/ping-single")]
async fn ping_single() -> impl Responder {
    HttpResponse::Ok().json(PingSingle { success: true })
}

#[post("/plonky2-verify")]
async fn verify_plonky2(
    plonky2_hashmap: web::Data<Arc<Mutex<HashMap<String, Plonky2Proof>>>>,
    data: web::Json<ProofDataPlonky2>,
) -> impl Responder {
    info!("{:?}", data);
    let mut plonky2_hashmap = plonky2_hashmap.lock().await;
    let proof_data = data.into_inner();
    plonky2_hashmap.insert(
        proof_data.tx_id.clone(),
        Plonky2Proof {
            proof_file_path: proof_data.proof_file_path.clone(),
            common_data_file_path: proof_data.common_data_file_path.clone(),
            verifier_data_file_path: proof_data.verifier_data_file_path.clone(),
        },
    );
    HttpResponse::Ok().json(SubmitionResult { is_submitted: true })
}

#[post("/verify")]
async fn verify(
    queue: web::Data<Arc<Mutex<VecDeque<VerifyProof>>>>,
    plonky2_hashmap: web::Data<Arc<Mutex<HashMap<String, Plonky2Proof>>>>,
    data: web::Json<VerifyProof>,
) -> impl Responder {
    info!("{:?}", data);
    let proof_data = data.into_inner();
    let mut verify_queue = queue.lock().await;
    let plonk2_hashmap = plonky2_hashmap.lock().await;
    match plonk2_hashmap.get(&proof_data.tx_id) {
        Some(_plonky2_proof) => {
            verify_queue.push_back(proof_data);
        }
        None => {
            warn!("Invalid Jolt proof ID");
            return HttpResponse::Ok().json(SubmitionResult {
                is_submitted: false,
            });
        }
    }
    HttpResponse::Ok().json(SubmitionResult { is_submitted: true })
}
