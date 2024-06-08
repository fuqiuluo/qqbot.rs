use log::info;
use ntrim_macros::command;
use crate::pb::trpc::status::{SilenceState, SsoHeartBeatRequest, SsoHeartBeatResponse};

struct HeartBeatCodec;

#[command("Heartbeat.Alive", "send_heartbeat", Protobuf, Heartbeat)]
impl HeartBeatCodec {
    async fn generate(bot: &Arc<Bot>) -> Option<Vec<u8>> { None }

    async fn parse(bot: &Arc<Bot>, data: Vec<u8>) -> Option<()> { None }
}