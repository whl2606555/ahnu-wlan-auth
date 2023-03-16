use std::str::FromStr;

use regex::Regex;

lazy_static::lazy_static! {
    static ref JSONP_REGEX:regex::Regex = Regex::new("dr1003\\((.+)\\)").unwrap();
    static ref HTTP_CLIENT: reqwest::Client = {
        reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36")
    .build()
    .unwrap()
    };
}

pub struct AhnuWlanAuthenticator {
    username: String,
    password: String,
    login_url: String,
}

impl AhnuWlanAuthenticator {
    pub fn new(username: String, password: String, login_url: String) -> Self {
        Self {
            username,
            password,
            login_url,
        }
    }

    pub async fn try_auth(&self) -> Result<(), String> {
        let response = HTTP_CLIENT
            .get(format!("http://{}/eportal/portal/login", self.login_url))
            .query(&[
                ("callback", "dr1003"),
                ("login_method", "1"),
                ("user_account", self.username.as_str()),
                ("user_password", self.password.as_str()),
            ])
            .send()
            .await
            .map_err(|_| "HTTP请求失败")?
            .text()
            .await
            .map_err(|_| "HTTP响应非TEXT类型")?;

        #[cfg(debug_assertions)]
        {
            log::info!("ResponseText: {:?}", response);
        }

        let response_json = JSONP_REGEX
            .captures(&response)
            .ok_or(format!("响应不正确: {}", response))?;

        let response_json = response_json
            .get(1)
            .ok_or(format!("响应不正确: {}", response))?
            .as_str();

        #[cfg(debug_assertions)]
        {
            log::info!("ResponseJson: {:?}", response_json)
        }

        let response_json =
            serde_json::Value::from_str(response_json).map_err(|_| "Json格式不正确")?;
        if response_json["result"].as_i64().unwrap_or(0) != 1 {
            #[cfg(debug_assertions)]
            {
                log::error!("result: {}", response_json["result"].as_i64().unwrap_or(-1));
                log::error!("登录失败: {}", response_json.to_string());
            }

            return Err(response_json["msg"].as_str().unwrap_or("").to_owned());
        }

        log::info!("登录成功!返回信息: {}", response_json.to_string());
        Ok(())
    }

    pub async fn is_web_avail() -> bool {
        let response = match HTTP_CLIENT
            .get("http://acs.m.taobao.com/gw/mtop.common.getTimestamp/")
            .send()
            .await
        {
            Ok(v) => v,
            Err(_) => return false,
        };
        #[cfg(debug_assertions)]
        {
            log::info!("StatusCode: {}", response.status());
        }

        if response.status().is_redirection() {
            return false;
        }
        true
    }
}
