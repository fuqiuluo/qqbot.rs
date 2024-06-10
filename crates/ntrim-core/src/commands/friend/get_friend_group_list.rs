use std::collections::HashMap;
use anyhow::Error;
use bytes::{Buf, Bytes, BytesMut};
use jcers::{JceGet, JcePut};
use ntrim_macros::command;
use crate::{await_response, jce, pb};
use crate::commands::troop::GroupMemberPermission;
use crate::jce::{next_request_id, pack_uni_request_data};

#[derive(Debug, Default, Clone)]
pub struct FriendListResponse {
    /// 好友列表
    pub friends: Vec<FriendInfo>,
    /// 好友分组
    pub friend_groups: HashMap<i16, FriendGroupInfo>,
    /// 好友数量
    pub total_count: i16,
    /// 在线好友数量
    pub online_friend_count: i16,
}

/// 好友信息
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "sql", derive(sqlx::FromRow))]
pub struct FriendInfo {
    pub uin: i64,
    pub nick: String,
    pub remark: String,
    pub face_id: i16,
    pub group_id: i16,
}

/// 好友分组信息
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "sql", derive(sqlx::FromRow))]
pub struct FriendGroupInfo {
    pub group_id: i16,
    pub group_name: String,
    pub friend_count: i32,
    pub online_friend_count: i32,
    pub seq_id: i16,
}

struct GetFriendGroupListCodec;

#[command("friendlist.getFriendGroupList", "_get_friend_list", Service)]
impl GetFriendGroupListCodec {
    async fn generate(
        bot: &Arc<Bot>,
        friend_start_index: i16,
        friend_list_count: i16,
        group_start_index: i16,
        group_list_count: i16,
    ) -> Option<Vec<u8>> {
        let mut d50 = BytesMut::new();
        prost::Message::encode(
            &pb::friendlist::D50ReqBody {
                appid: 1002,
                req_music_switch: 1,
                req_mutualmark_alienation: 1,
                req_ksing_switch: 1,
                req_mutualmark_lbsshare: 1,
                ..Default::default()
            },
            &mut d50,
        )
            .expect("failed to encode pb");
        let req = crate::jce::friendlist::get_friend_group_list::FriendListRequest {
            reqtype: 3,
            if_reflush: if friend_start_index <= 0 { 0 } else { 1 },
            uin: bot.unique_id,
            start_index: friend_start_index,
            friend_count: friend_list_count,
            group_id: 0,
            if_get_group_info: 1,
            group_start_index: group_start_index as u8,
            group_count: group_list_count as u8,
            if_get_msf_group: 0,
            if_show_term_type: 1,
            version: 27,
            uin_list: vec![],
            app_type: 0,
            if_get_dov_id: 0,
            if_get_both_flag: 0,
            d50: Bytes::from(d50),
            d6b: Bytes::new(),
            sns_type_list: vec![13580, 13581, 13582],
        };
        let buf = jce::RequestDataVersion3 {
            map: HashMap::from([("FL".to_string(), pack_uni_request_data(&req.freeze()))]),
        };
        let pkt = jce::RequestPacket {
            i_version: 3,
            c_packet_type: 0x003,
            i_request_id: 1921334514,
            s_servant_name: "mqq.IMService.FriendListServiceServantObj".to_string(),
            s_func_name: "GetFriendListReq".to_string(),
            s_buffer: buf.freeze(),
            context: Default::default(),
            status: Default::default(),
            ..Default::default()
        };
        Some(pkt.freeze().to_vec())
    }


    // friendlist.getFriendGroupList
    pub fn decode_friend_group_list_response(
        mut payload: Bytes,
    ) -> Result<FriendListResponse, Error> {
        let mut request: jce::RequestPacket =
            jcers::from_buf(&mut payload).map_err(Error::from)?;
        let mut data: jce::RequestDataVersion3 =
            jcers::from_buf(&mut request.s_buffer).map_err(Error::from)?;
        let mut fl_resp = data.map.remove("FLRESP").ok_or_else(|| {
            Error::msg("decode_friend_group_list_response FLRESP not found")
        })?;
        fl_resp.advance(1);
        let resp: jce::friendlist::get_troop_member_list::FriendListResponse = jcers::from_buf(&mut fl_resp).map_err(Error::from)?;
        Ok(FriendListResponse {
            total_count: resp.total_friend_count,
            online_friend_count: resp.online_friend_count,
            friends: resp
                .friend_info_list
                .into_iter()
                .map(|f| FriendInfo {
                    uin: f.friend_uin,
                    nick: f.nick,
                    remark: f.remark,
                    face_id: f.face_id,
                    group_id: f.group_id as i16,
                })
                .collect(),
            friend_groups: resp
                .group_info_list
                .into_iter()
                .map(|g| {
                    (
                        g.group_id as i16,
                        FriendGroupInfo {
                            group_id: g.group_id as i16,
                            group_name: g.group_name,
                            friend_count: g.friend_count,
                            online_friend_count: g.online_friend_count,
                            seq_id: g.seq_id as i16,
                        },
                    )
                })
                .collect(),
        })
    }

    async fn parse(bot: &Arc<Bot>, data: Vec<u8>) -> Option<FriendListResponse> {
        match Self::decode_friend_group_list_response(Bytes::from(data)) {
            Ok(resp) => {
                return Some(resp)
            }
            Err(e) => {
                error!("Failed to parse friend group list: {}", e);
                None
            }
        }
    }
}

impl Bot {
    pub async fn get_friend_list(self: &Arc<Self>) -> Result<FriendListResponse, Error> {
        let mut output = FriendListResponse::default();
        loop {
            match await_response!(tokio::time::Duration::from_secs(5), async {
                let rx = Bot::_get_friend_list(self, output.friends.len() as i16, 150, 0, 0).await;
                if let Some(rx) = rx {
                    rx.await.map_err(|e| Error::new(e))
                } else {
                    Err(Error::msg("Unable to get_friend_list: tcp connection exception"))
                }
            }, |value| {
                Ok(value)
            }, |e| {
                Err(e)
            }) {
                Ok(Some(resp)) => {
                    output.friend_groups.extend(resp.friend_groups);
                    output.friends.extend(resp.friends);
                    output.total_count = resp.total_count;
                    if output.friends.len() as i16 >= resp.total_count {
                        break;
                    }
                }
                Err(e) => {
                    return Err(e);
                }
                Ok(None) => {
                    warn!("get_friend_list: no more data");
                    break;
                }
            }
        }
        #[cfg(feature = "sql")]
        if crate::db::is_initialized() {
            let bot_id = self.unique_id;
            let mut output = output.clone();
            tokio::spawn(async move {
                let pool = crate::db::PG_POOL.get().unwrap();
                for friend_info in output.friends.into_iter() {
                    FriendListResponse::insert(pool, bot_id, friend_info).await
                        .expect("Failed to insert friend");
                }
                for (_, group_info) in output.friend_groups.into_iter() {
                    FriendListResponse::insert_group(pool, bot_id, group_info).await
                        .expect("Failed to insert friend group");
                }
            });
        }
        Ok(output)
    }
}