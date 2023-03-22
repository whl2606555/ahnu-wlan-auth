use ahnu_wlan_auth::AhnuWlanAuthenticator;
use config::{Config, File, FileFormat};
use fast_log::Config as FastlogConfig;

#[tokio::main]
async fn main() {
    // 初始化配置文件
    let config = Config::builder()
        .add_source(File::new("config", FileFormat::Toml))
        .build()
        .unwrap();

    fast_log::init(FastlogConfig::new().console().chan_len(Some(100000)).level(log::LevelFilter::Info)).unwrap();

    let author = AhnuWlanAuthenticator::new(
        config.get("login.username").unwrap(),
        config.get("login.password").unwrap(),
        config.get("login.url").unwrap(),
    );

    loop {
        if !AhnuWlanAuthenticator::is_web_avail().await {
            log::info!("网络不可用，尝试登录中...");
            match author.try_auth().await {
                Ok(_) => {
                    log::info!("登录成功");
                },
                Err(e) => {
                    log::error!("尝试登录校园网失败: {}", e);
                }
            }
        }else{
            log::info!("网络可用，无需登录");
        }

        tokio::time::sleep(std::time::Duration::from_secs(5))
        .await;
    }
}
