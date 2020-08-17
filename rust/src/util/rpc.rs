use std::{thread, time};

use filecoin_webapi::polling::PollingState;
use log::{info, trace};
use reqwest::blocking::Client;
use serde::ser::Serialize;
use serde_json::value::from_value;
use serde_json::{json, Value};
use std::env;

lazy_static! {
    static ref REQWEST_CLIENT: Client = Client::new();
    static ref HOST: String = env::var("FFI_REMOTE_COMMIT2_BASE_URL").unwrap();
}

#[allow(dead_code)]
pub(crate) fn webapi_post<T: Serialize + ?Sized>(path: &str, json: &T) -> Result<Value, String> {
    let post = REQWEST_CLIENT.post(&format!("http://{}/{}", &*HOST, path));
    let response = post
        .json(json)
        .send()
        .map_err(|e| format!("{:?}", e))?
        .text()
        .map_err(|e| format!("{:?}", e))?;
    let value: Value = serde_json::from_str(&response).map_err(|e| format!("{:?}", e))?;

    if value.get("Err").is_some() {
        return Err(format!("{:?}", value));
    }

    return Ok(value);
}

#[allow(dead_code)]
pub(crate) fn webapi_post_polling<T: Serialize + ?Sized>(path: &str, json: &T) -> Result<Value, String> {
    let state: PollingState = from_value(webapi_post(path, json)?).map_err(|e| format!("{:?}", e))?;
    info!("webapi_post_polling request state: {:?}", state);
    let proc_id = match state {
        PollingState::Started(val) => val,
        _ => {
            return Err(format!("webapi_post_polling response error: {:?}", state));
        }
    };

    loop {
        let poll_state: PollingState =
            from_value(webapi_post("sys/query_state", &json!(proc_id))?).map_err(|e| format!("{:?}", e))?;
        trace!("webapi_post_polling poll_state: {:?}", poll_state);
        match poll_state {
            PollingState::Done(result) => return Ok(result),
            PollingState::Pending => {}
            e @ _ => return Err(format!("poll_state error: {:?}", e)),
        }

        // sleep 30s
        let time = time::Duration::from_secs(30);
        thread::sleep(time);
    }
}

#[allow(dead_code)]
macro_rules! webapi_post_polling {
    ($path:literal, $json:expr) => {
        crate::util::rpc::webapi_post_polling($path, $json);
    };
}
