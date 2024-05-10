use std::io::{BufReader, Read};

use flate2::{
    bufread::{DeflateDecoder, DeflateEncoder},
    Compression,
};
use http::StatusCode;
use megamind::models::{Response, SearchResponse, SongResponse};
use redis::{pipe, AsyncCommands};
use serde_json::{from_slice, to_vec};

use crate::{AppState, ErrIntermediate};

macro_rules! cache_endpoint {
    ($resource:ident, $response:ty, $param:ident, $param_type:ty) => {
        impl AppState {
            pub async fn $resource(
                &self,
                $param: $param_type,
            ) -> Result<$response, ErrIntermediate> {
                let mut conn = self.redis.get_async_connection().await?;
                let key = format!("{}/{}", stringify!($resource), $param);
                if conn.exists(&key).await? {
                    let data: Vec<u8> = conn.get(&key).await?;
                    let mut decompressor = DeflateDecoder::new(data.as_slice());
                    let mut decompressed = Vec::new();
                    decompressor.read_to_end(&mut decompressed)?;
                    Ok(from_slice(&decompressed)?)
                } else {
                    match self.megamind.$resource($param).await? {
                        Response::Success { meta: _, response } => {
                            let data = to_vec(&response)?;
                            let mut compressor = DeflateEncoder::new(
                                BufReader::new(data.as_slice()),
                                Compression::best(),
                            );
                            let mut compressed = Vec::new();
                            compressor.read_to_end(&mut compressed)?;
                            pipe()
                                .set(&key, compressed)
                                .expire(&key, 600)
                                .query_async(&mut conn)
                                .await?;
                            Ok(response)
                        }
                        Response::Error { meta, response } => {
                            let mut full_error = meta.message;
                            if let Some(resp) = response {
                                full_error.push_str(&resp.error);
                            }
                            Err(ErrIntermediate::new(
                                full_error,
                                StatusCode::from_u16(meta.status)
                                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                            ))
                        }
                        Response::Other {
                            error,
                            error_description,
                        } => Err(ErrIntermediate::new(
                            format!("{}: {}", error, error_description),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        )),
                    }
                }
            }
        }
    };
}

cache_endpoint!(song, SongResponse, id, u32);
cache_endpoint!(search, SearchResponse, query, &str);
