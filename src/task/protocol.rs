use super::model::*;
use super::Result;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;

lazy_static! {
    /// Last seq of request packet
    static ref LAST_SEQ: AtomicU64 = AtomicU64::new(1u64);
}

/// Host request
// Implement Debug for error handling.
#[derive(Default, Serialize)]
pub struct Request {
    /// Request sequence
    pub seq: u64,
    /// Packet size
    pub size: u32,
    /// Payload
    pub payload: Vec<u8>,
}

/// Agent response
#[derive(Debug, Default, Deserialize)]
pub struct Response {
    /// Response sequence
    pub ack: u64,
    /// Response size
    pub size: u32,
    /// Status code
    pub code: u16,
    /// Payload
    pub payload: Vec<u8>,
}
use crate::models::pay::{ElectricityBill, ElectricityBillRequest};

/// Response payload
#[derive(Serialize)]
pub enum RequestPayload {
    AgentInfo(AgentInfoRequest),
    ElectricityBill(ElectricityBillRequest),
    ActivityList(ActivityListRequest),
    ScoreList(CourseScoreRequest),
}

/// Response payload
#[derive(Deserialize)]
pub enum ResponsePayload {
    AgentInfo(AgentInfo),
    ElectricityBill(ElectricityBill),
    ActivityList(Vec<Activity>),
    ScoreList(Vec<CourseScore>),
}

impl Request {
    pub fn new(payload: RequestPayload) -> Self {
        let seq = LAST_SEQ.fetch_add(1, Ordering::Relaxed);
        let payload = bincode::serialize(&payload).unwrap();

        Self {
            seq,
            // We will not construct a message more than 2^32 bytes
            size: payload.len() as u32,
            payload,
        }
    }
}

impl Response {
    async fn read_header(buffer: &mut BufReader<OwnedReadHalf>) -> Result<Self> {
        // Default response header is 14 bytes.
        let mut response = Response::default();

        // Read the control fields
        response.ack = buffer.read_u64().await?;
        response.size = buffer.read_u32().await?;
        response.code = buffer.read_u16().await?;

        Ok(response)
    }

    pub async fn from_stream(buffer: &mut BufReader<OwnedReadHalf>) -> Result<Self> {
        let mut response = Self::read_header(buffer).await?;

        if response.size == 0 {
            return Ok(response);
        }
        response.payload = vec![0u8; response.size as usize];
        // Read body
        let mut p = 0usize; // read len
        while p < response.size as usize {
            let mut read_currently = response.size as usize - p;
            if read_currently > 2048 {
                read_currently = 2048usize;
            }
            p += buffer
                .read_exact(&mut response.payload[p..(p + read_currently)])
                .await?;
        }
        Ok(response)
    }

    pub async fn is_ok(&self) -> bool {
        self.code == 0
    }

    pub fn payload(self) -> Result<ResponsePayload> {
        Ok(bincode::deserialize(&self.payload)?)
    }
}
