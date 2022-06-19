use std::collections::btree_map::BTreeMap;

use anyhow::Result;
use filecoin_proofs_api::{self as api, SectorId};
use filecoin_proofs_api::{
    PrivateReplicaInfo as api_PrivateReplicaInfo,
    PrivateSectorPathInfo as api_PrivateSectorPathInfo,
    // PublicReplicaInfo as api_PublicReplicaInfo,
};
use safer_ffi::prelude::*;

use crate::util::types::{
    as_path_buf,
};

use super::types::{
    PrivateReplicaInfo, PrivateSectorPathInfo, PublicReplicaInfo, RegisteredPoStProof,
};

#[derive(Debug, Clone)]
struct PublicReplicaInfoTmp {
    pub registered_proof: RegisteredPoStProof,
    pub comm_r: [u8; 32],
    pub sector_id: u64,
}

pub fn to_public_replica_info_map(
    replicas: c_slice::Ref<PublicReplicaInfo>,
) -> BTreeMap<SectorId, api::PublicReplicaInfo> {
    use rayon::prelude::*;

    let replicas = replicas
        .iter()
        .map(|ffi_info| PublicReplicaInfoTmp {
            sector_id: ffi_info.sector_id,
            registered_proof: ffi_info.registered_proof,
            comm_r: ffi_info.comm_r,
        })
        .collect::<Vec<_>>();

    replicas
        .into_par_iter()
        .map(|info| {
            let PublicReplicaInfoTmp {
                registered_proof,
                comm_r,
                sector_id,
            } = info;

            (
                SectorId::from(sector_id),
                api::PublicReplicaInfo::new(registered_proof.into(), comm_r),
            )
        })
        .collect()
}

#[derive(Debug)]
struct PrivateReplicaInfoTmp {
    pub registered_proof: RegisteredPoStProof,
    pub cache_dir_path: std::path::PathBuf,
    pub cache_in_oss: bool,
    pub cache_sector_path_info: PrivateSectorPathInfo,
    pub comm_r: [u8; 32],
    pub replica_path: std::path::PathBuf,
    pub replica_in_oss: bool,
    pub replica_sector_path_info: PrivateSectorPathInfo,
    pub sector_id: u64,
}

pub fn to_private_replica_info_map(
    replicas: c_slice::Ref<PrivateReplicaInfo>,
) -> Result<BTreeMap<SectorId, api::PrivateReplicaInfo>> {
    use rayon::prelude::*;

    let replicas: Vec<_> = replicas
        .iter()
        .map(|ffi_info| {
            Ok(PrivateReplicaInfoTmp {
                registered_proof: ffi_info.registered_proof,
                cache_dir_path: as_path_buf(&ffi_info.cache_dir_path).unwrap(),
                cache_in_oss: ffi_info.cache_in_oss,
                cache_sector_path_info: ffi_info.cache_sector_path_info.clone(),
                comm_r: ffi_info.comm_r,
                replica_path: as_path_buf(&ffi_info.replica_path).unwrap(),
                replica_in_oss: ffi_info.replica_in_oss,
                replica_sector_path_info: ffi_info.replica_sector_path_info.clone(),
                sector_id: ffi_info.sector_id,
            })
        })
        .collect::<Result<_>>()?;

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

            let api_replica_sector_path_info = api_PrivateSectorPathInfo {
                endpoints: String::from_utf8(replica_sector_path_info.endpoints.to_vec()).unwrap(),
                landed_dir: as_path_buf(&replica_sector_path_info.landed_dir).unwrap(),
                access_key: String::from_utf8(replica_sector_path_info.access_key.to_vec())
                    .unwrap(),
                secret_key: String::from_utf8(replica_sector_path_info.secret_key.to_vec())
                    .unwrap(),
                bucket_name: String::from_utf8(replica_sector_path_info.bucket_name.to_vec())
                    .unwrap(),
                sector_name: String::from_utf8(replica_sector_path_info.sector_name.to_vec())
                    .unwrap(),
                region: String::from_utf8(replica_sector_path_info.region.to_vec()).unwrap(),
                multi_ranges: replica_sector_path_info.multi_ranges,
            };

            let api_cache_sector_path_info = api_PrivateSectorPathInfo {
                endpoints: String::from_utf8(cache_sector_path_info.endpoints.to_vec()).unwrap(),
                landed_dir: as_path_buf(&cache_sector_path_info.landed_dir).unwrap(),
                access_key: String::from_utf8(cache_sector_path_info.access_key.to_vec()).unwrap(),
                secret_key: String::from_utf8(cache_sector_path_info.secret_key.to_vec()).unwrap(),
                bucket_name: String::from_utf8(cache_sector_path_info.bucket_name.to_vec())
                    .unwrap(),
                sector_name: String::from_utf8(cache_sector_path_info.sector_name.to_vec())
                    .unwrap(),
                region: String::from_utf8(cache_sector_path_info.region.to_vec()).unwrap(),
                multi_ranges: cache_sector_path_info.multi_ranges,
            };

            (
                SectorId::from(sector_id),
                api_PrivateReplicaInfo::new_with_oss_config(
                    registered_proof.into(),
                    replica_path,
                    replica_in_oss,
                    api_replica_sector_path_info,
                    comm_r,
                    cache_dir_path,
                    cache_in_oss,
                    api_cache_sector_path_info,
                ),
            )
        })
        .collect();

    Ok(map)
}
