use std::{fmt};
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::SeqCst;
use anyhow::Error;
use bitflags::bitflags;
use log::{warn};
use ntrim_tools::tokiort;
use crate::client::qsecurity::QSecurity;
use crate::client::trpc::TrpcClient;
use crate::servlet::olpush::OlPushServlet;
use crate::servlet::register::RegisterProxyServlet;
use crate::session::SsoSession;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BotStatus: u32 {
        /// 在线
        const Online =  0b00000001;
        /// 离线
        const Offline = 0b00000010;
        /// 冻结
        const Freeze =  0b00000100;
    }
}

pub struct Bot {
    pub unique_id: i64,
    /// TCP client.
    pub client: Arc<TrpcClient>,
    /// Bot status.
    pub status: AtomicU32,
}

impl Bot {
    pub async fn new(
        session: SsoSession,
        qsec_mod: Arc<dyn QSecurity>,
    ) -> Result<Arc<Self>, Error> {
        let unique_id = session.uin;
        let client = TrpcClient::new(session, qsec_mod).await?;

        let bot = Arc::new(Self {
            unique_id,
            client,
            status: AtomicU32::new(BotStatus::Offline.bits()),
        });
        RegisterProxyServlet::initialize(&bot).await;
        OlPushServlet::initialize(&bot).await;

        if std::env::var("AUTO_RECONNECT").map_or(true, |v| v == "1") {
            Self::auto_reconnect(&bot).await;
        }
        if std::env::var("AUTO_REFRESH_SESSION").map_or(true, |v| v == "1") {
            Self::auto_refresh_session(&bot).await;
        }

        Ok(bot)
    }

    pub fn set_online(&self) {
        let mut status = BotStatus::from_bits(self.status.load(SeqCst)).unwrap();
        status.set(BotStatus::Online, true);
        status.set(BotStatus::Offline, false);
        self.status.store(status.bits(), SeqCst);
    }

    pub async fn set_offline(&self) {
        warn!("Bot status change to offline");
        let mut status = BotStatus::from_bits(self.status.load(SeqCst)).unwrap();
        status.set(BotStatus::Online, false);
        status.set(BotStatus::Offline, true);
        self.status.store(status.bits(), SeqCst);
        self.client.set_lost().await;
    }

    pub async fn is_online(&self) -> bool {
        self.client.is_connected().await &&
            BotStatus::from_bits(self.status.load(SeqCst)).unwrap().contains(BotStatus::Online)
    }
}

impl fmt::Debug for Bot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let uin = tokiort::global_tokio_runtime().block_on(async move {
            self.client.session.read().await.uin
        });
        f.debug_struct("Bot")
            .field("uin", &uin)
            .finish()
    }
}