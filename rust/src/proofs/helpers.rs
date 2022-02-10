use std::collections::btree_map::BTreeMap;
use std::path::PathBuf;
use std::slice::from_raw_parts;

use anyhow::{ensure, Result};
use ffi_toolkit::{c_str_to_pbuf, c_str_to_rust_str};
use filecoin_proofs_api::{PrivateReplicaInfo, PublicReplicaInfo, SectorId, PrivateSectorPathInfo};

use super::types::{fil_PrivateReplicaInfo, fil_PublicReplicaInfo, fil_RegisteredPoStProof};
use crate::proofs::types::{
    fil_PartitionProof, fil_PartitionSnarkProof, fil_PoStProof, PartitionProof,
    PartitionSnarkProof, PoStProof,
};

#[derive(Debug, Clone)]
struct PublicReplicaInfoTmp {
    pub registered_proof: fil_RegisteredPoStProof,
    pub comm_r: [u8; 32],
    pub sector_id: u64,
}

#[allow(clippy::type_complexity)]
pub unsafe fn to_public_replica_info_map(
    replicas_ptr: *const fil_PublicReplicaInfo,
    replicas_len: libc::size_t,
) -> Result<BTreeMap<SectorId, PublicReplicaInfo>> {
    use rayon::prelude::*;

    ensure!(!replicas_ptr.is_null(), "replicas_ptr must not be null");

    let mut replicas = Vec::new();

    for ffi_info in from_raw_parts(replicas_ptr, replicas_len) {
        replicas.push(PublicReplicaInfoTmp {
            sector_id: ffi_info.sector_id,
            registered_proof: ffi_info.registered_proof,
            comm_r: ffi_info.comm_r,
        });
    }

    let map = replicas
        .into_par_iter()
        .map(|info| {
            let PublicReplicaInfoTmp {
                registered_proof,
                comm_r,
                sector_id,
            } = info;

            (
                SectorId::from(sector_id),
                PublicReplicaInfo::new(registered_proof.into(), comm_r),
            )
        })
        .collect();

    Ok(map)
}

#[derive(Debug, Clone)]
struct PrivateReplicaInfoTmp {
    pub registered_proof: fil_RegisteredPoStProof,
    pub cache_dir_path: std::path::PathBuf,
    pub cache_in_oss: bool,
    pub cache_sector_path_info: PrivateSectorPathInfo,
    pub comm_r: [u8; 32],
    pub replica_path: std::path::PathBuf,
    pub replica_in_oss: bool,
    pub replica_sector_path_info: PrivateSectorPathInfo,
    pub sector_id: u64,
}

pub unsafe fn to_private_replica_info_map(
    replicas_ptr: *const fil_PrivateReplicaInfo,
    replicas_len: libc::size_t,
) -> Result<BTreeMap<SectorId, PrivateReplicaInfo>> {
    use rayon::prelude::*;

    ensure!(!replicas_ptr.is_null(), "replicas_ptr must not be null");

    let replicas: Vec<_> = from_raw_parts(replicas_ptr, replicas_len)
        .iter()
        .map(|ffi_info| {
            let cache_dir_path = c_str_to_pbuf(ffi_info.cache_dir_path);
            let replica_path = c_str_to_rust_str(ffi_info.replica_path).to_string();

            let replica_sector_path_info = PrivateSectorPathInfo {
                endpoints: c_str_to_rust_str(ffi_info.replica_sector_path_info.endpoints as *mut libc::c_char).to_string(),
                landed_dir: c_str_to_pbuf(ffi_info.replica_sector_path_info.landed_dir),
                access_key: c_str_to_rust_str(ffi_info.replica_sector_path_info.access_key as *mut libc::c_char).to_string(),
                secret_key: c_str_to_rust_str(ffi_info.replica_sector_path_info.secret_key as *mut libc::c_char).to_string(),
                bucket_name: c_str_to_rust_str(ffi_info.replica_sector_path_info.bucket_name as *mut libc::c_char).to_string(),
                sector_name: c_str_to_rust_str(ffi_info.replica_sector_path_info.sector_name as *mut libc::c_char).to_string(),
                region: c_str_to_rust_str(ffi_info.replica_sector_path_info.region as *mut libc::c_char).to_string(),
                multi_ranges: ffi_info.replica_sector_path_info.multi_ranges,
            };

            let cache_sector_path_info = PrivateSectorPathInfo {
                endpoints: c_str_to_rust_str(ffi_info.cache_sector_path_info.endpoints as *mut libc::c_char).to_string(),
                landed_dir: c_str_to_pbuf(ffi_info.cache_sector_path_info.landed_dir),
                access_key: c_str_to_rust_str(ffi_info.cache_sector_path_info.access_key as *mut libc::c_char).to_string(),
                secret_key: c_str_to_rust_str(ffi_info.cache_sector_path_info.secret_key as *mut libc::c_char).to_string(),
                bucket_name: c_str_to_rust_str(ffi_info.cache_sector_path_info.bucket_name as *mut libc::c_char).to_string(),
                sector_name: c_str_to_rust_str(ffi_info.cache_sector_path_info.sector_name as *mut libc::c_char).to_string(),
                region: c_str_to_rust_str(ffi_info.cache_sector_path_info.region as *mut libc::c_char).to_string(),
                multi_ranges: ffi_info.cache_sector_path_info.multi_ranges,
            };

            PrivateReplicaInfoTmp {
                registered_proof: ffi_info.registered_proof,
                cache_dir_path,
                cache_in_oss: ffi_info.cache_in_oss,
                cache_sector_path_info: cache_sector_path_info,
                comm_r: ffi_info.comm_r,
                replica_path: PathBuf::from(replica_path),
                replica_in_oss: ffi_info.replica_in_oss,
                replica_sector_path_info: replica_sector_path_info,
                sector_id: ffi_info.sector_id,
            }
        })
        .collect();

    let map = replicas
        .into_par_iter()
        .map(|info| {
            let PrivateReplicaInfoTmp {
                registered_proof,
                cache_dir_path,
                cache_in_oss,
                cache_sector_path_info,
                comm_r,
                replica_path,
                replica_in_oss,
                replica_sector_path_info,
                sector_id,
            } = info;

            (
                SectorId::from(sector_id),
                /*
                PrivateReplicaInfo::new(
                    registered_proof.into(),
                    comm_r,
                    cache_dir_path,
                    replica_path,
                ),
                */
                PrivateReplicaInfo::new_with_oss_config(
                    registered_proof.into(),
                    replica_path,
                    replica_in_oss,
                    replica_sector_path_info,
                    comm_r,
                    cache_dir_path,
                    cache_in_oss,
                    cache_sector_path_info,
                ),
            )
        })
        .collect();

    Ok(map)
}

pub unsafe fn c_to_rust_post_proofs(
    post_proofs_ptr: *const fil_PoStProof,
    post_proofs_len: libc::size_t,
) -> Result<Vec<PoStProof>> {
    ensure!(
        !post_proofs_ptr.is_null(),
        "post_proofs_ptr must not be null"
    );

    let out = from_raw_parts(post_proofs_ptr, post_proofs_len)
        .iter()
        .map(|fpp| PoStProof {
            registered_proof: fpp.registered_proof.into(),
            proof: from_raw_parts(fpp.proof_ptr, fpp.proof_len).to_vec(),
        })
        .collect();

    Ok(out)
}

pub unsafe fn c_to_rust_vanilla_partition_proofs(
    partition_proofs_ptr: *const fil_PartitionProof,
    partition_proofs_len: libc::size_t,
) -> Result<Vec<PartitionProof>> {
    ensure!(
        !partition_proofs_ptr.is_null(),
        "partition_proofs_ptr must not be null"
    );

    let out = from_raw_parts(partition_proofs_ptr, partition_proofs_len)
        .iter()
        .map(|fpp| PartitionProof {
            proof: from_raw_parts(fpp.proof_ptr, fpp.proof_len).to_vec(),
        })
        .collect();

    Ok(out)
}

pub unsafe fn c_to_rust_partition_proofs(
    partition_proofs_ptr: *const fil_PartitionSnarkProof,
    partition_proofs_len: libc::size_t,
) -> Result<Vec<PartitionSnarkProof>> {
    ensure!(
        !partition_proofs_ptr.is_null(),
        "partition_proofs_ptr must not be null"
    );

    let out = from_raw_parts(partition_proofs_ptr, partition_proofs_len)
        .iter()
        .map(|fpp| PartitionSnarkProof {
            registered_proof: fpp.registered_proof.into(),
            proof: from_raw_parts(fpp.proof_ptr, fpp.proof_len).to_vec(),
        })
        .collect();

    Ok(out)
}
