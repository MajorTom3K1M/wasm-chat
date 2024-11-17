use super::Message;
use js_sys::Date;
use js_sys::Math::random;
use js_sys::Promise;
use js_sys::JSON;
use serde_wasm_bindgen::from_value;
use serde_wasm_bindgen::to_value;
// use stdweb::Value;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console::info;
use web_sys::console::time_stamp;
use yew::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

#[wasm_bindgen]
extern "C" {
    pub type PubNub;

    #[wasm_bindgen(constructor)]
    fn new(key: JsValue) -> PubNub;

    #[wasm_bindgen(method)]
    fn addListener(this: &PubNub, events: js_sys::Object);

    #[wasm_bindgen(method)]
    fn subscribe(this: &PubNub, channels: JsValue);

    #[wasm_bindgen(method)]
    fn publish(this: &PubNub, payload: JsValue);

    #[wasm_bindgen(method, structural, js_name = "unsubscribe")]
    fn unsubscribe(this: &PubNub, channels: JsValue);

    #[wasm_bindgen(method)]
    fn unsubscribeAll(this: &PubNub);

    #[wasm_bindgen(method, structural)]
    fn setUUID(this: &PubNub, uuid: String);

    #[wasm_bindgen(method, structural)]
    fn getSubscribedChannels(this: &PubNub) -> js_sys::Array;

    #[wasm_bindgen(method, structural, js_name = "objects.getAllUUIDMetadata")]
    fn get_all_uuid_metadata(this: &PubNub, params: JsValue) -> Promise;

    #[wasm_bindgen(method, structural, js_name = "objects.removeUUIDMetadata")]
    fn remove_uuid_metadata(this: &PubNub, params: JsValue) -> Promise;

    #[wasm_bindgen(method, structural, js_name = "hereNow")]
    fn here_now(this: &PubNub, params: JsValue) -> Promise;
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Initialize {
    pub publish_key: String,
    pub subscribe_key: String,
    pub heartbeat_interval: u32,
}
pub struct PubnubService {
    lib: Rc<PubNub>,
    // chat: Option<Value>,
    channel: Option<String>,
    username: Option<String>,
}

#[derive(Serialize)]
pub struct Channels {
    pub channels: Vec<String>,
    pub withPresence: bool,
}

#[derive(Serialize)]
pub struct UnsubscribeChannels {
    pub channels: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct EventStatus {
    pub category: String,
    pub operation: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PubNubMessage {
    channel: String,
    publisher: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Presence {
    pub uuid: String,
}

#[derive(Serialize, Deserialize)]
pub struct HereNowParameters {
    pub channels: Vec<String>,
    #[serde(rename = "includeUUIDs")]
    pub include_uuids: bool,
    #[serde(rename = "includeState")]
    pub include_state: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HereNowResponse {
    #[serde(rename = "totalChannels")]
    pub total_channels: Option<u32>,
    #[serde(rename = "totalOccupancy")]
    pub total_occupancy: Option<u32>,
    pub channels: Option<std::collections::HashMap<String, ChannelOccupancy>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelOccupancy {
    pub name: String,
    pub occupancy: Option<u32>,
    pub occupants: Option<Vec<Occupants>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct  Occupants {
    pub uuid: String,
    pub state: Option<String>,
}

impl PubnubService {
    pub fn new() -> Self {
        let init = Initialize {
            publish_key: "pub-c-bda71cb0-c615-4fa3-9d91-da409870c219".to_owned(),
            subscribe_key: "sub-c-0719525a-9783-4fd2-baf1-384eb78d5845".to_owned(),
            heartbeat_interval: 6
        };
        // let chat_engine = PubNub::new(to_value(&init).unwrap());
        let chat_engine = Rc::new(PubNub::new(to_value(&init).unwrap()));

        PubnubService::setup_disconnect_handler(chat_engine.clone());

        PubnubService {
            lib: chat_engine,
            // chat: None,
            channel: None,
            username: None,
        }
    }

    pub fn send_message(&mut self, msg: &str) {
        let lib = self.lib.as_ref();

        let payload = JSON::parse(&format!(
            r#"{{
            "channel": "{}",
            "message": "{}"
        }}"#,
            self.channel.as_ref().unwrap(),
            msg
        ));

        lib.publish(payload.unwrap());
    }

    pub fn connect(
        &mut self,
        topic: &str,
        nickname: &str,
        onmessage: Callback<Message>,
        onoffline: Callback<String>,
        ononline: Callback<String>,
    ) -> () {
        let lib = self.lib.as_ref();

        lib.setUUID(nickname.to_owned());
        self.username = Some(nickname.to_owned());

        let chat_callback = move |text: String, source: String| {
            let msg = Message {
                text: text,
                from: source,
            };
            onmessage.emit(msg);
        };
        let useroffline_callback = move |username: String| {
            onoffline.emit(username);
        };
        let useronline_callback = move |username: String| {
            ononline.emit(username);
        };

        let arr = lib.getSubscribedChannels();

        if arr.index_of(&JsValue::from_str(topic), 0) < 0 {
            let channels = Channels {
                channels: vec![topic.to_owned()],
                withPresence: true,
            };
            self.channel = Some(topic.to_owned());
            
            
            lib.subscribe(to_value(&channels).unwrap());

            let event_handler = js_sys::Object::new();

            let message_closure = Closure::wrap(Box::new(move |message: JsValue| {
                let msg: PubNubMessage = from_value(message).unwrap();
                chat_callback(msg.message, msg.publisher);
            }) as Box<dyn FnMut(JsValue)>);
            let _ = js_sys::Reflect::set(
                &event_handler,
                &"message".into(),
                &message_closure.as_ref().unchecked_ref(),
            );
            message_closure.forget();

            let presence_closure = Closure::wrap(Box::new(move |presence: JsValue| {
                info!("Presence event: {:?}", presence);
                let pres: Presence = from_value(presence).unwrap();
                useronline_callback(pres.uuid);
            }) as Box<dyn FnMut(JsValue)>);
            let _ = js_sys::Reflect::set(
                &event_handler,
                &"presence".into(),
                &presence_closure.as_ref().unchecked_ref(),
            );
            presence_closure.forget();

            lib.addListener(event_handler);
        }
    }

    pub fn disconnect(&mut self) {
        let lib = self.lib.as_ref();
        self.username = None;
        self.channel = None;
        // lib.setUUID("Mike".to_owned());
        let channels = UnsubscribeChannels {
            channels: vec!["chatengine-demo-chat".to_owned()],
        };
        lib.unsubscribe(to_value(&channels).unwrap());
    }

    fn setup_disconnect_handler(lib: Rc<PubNub>) {
        info!("Setting up disconnect handler");
        let disconnect_closure = Closure::wrap(Box::new(move |_: Event| {
            info!("Disconnecting");
            lib.unsubscribeAll();
        }) as Box<dyn FnMut(Event)>);

        let window = web_sys::window().expect("no global `window` exists");

        window
            .add_event_listener_with_callback("beforeunload", disconnect_closure.as_ref().unchecked_ref())
            .expect("failed to add unload event listener");

        disconnect_closure.forget();
        info!("Disconnect handler setup");
    }

    pub fn fetch_uuid_metadata(&self, callback: Callback<HashMap<String, String>>) {
        let lib = self.lib.as_ref();

        let params = js_sys::Object::new(); // Pass any required parameters
        let promise = lib.get_all_uuid_metadata(params.into());

        let closure = Closure::wrap(Box::new(move |result: JsValue| {
            if let Ok(data) = from_value::<Vec<Presence>>(result) {
                let mut metadata_map = HashMap::new();
                for presence in data {
                    metadata_map.insert(presence.uuid.clone(), presence.uuid.clone()); // Example
                }
                callback.emit(metadata_map);
            }
        }) as Box<dyn FnMut(JsValue)>);

        let _ = promise.then(&closure);
        closure.forget();
    }

    pub fn fetch_here_now(
        &self,
        params: HereNowParameters,
        callback: Callback<Result<HereNowResponse, JsValue>>,
    ) {
        let lib = self.lib.as_ref();

        let js_params = to_value(&params).expect("Failed to serialize HereNowParameters");
        let promise = lib.here_now(js_params);

        let closure = Closure::wrap(Box::new(move |result: JsValue| {
            info!("HereNow response: {:?}", result);
            let response: Result<HereNowResponse, JsValue> = from_value(result)
                .map_err(|e| JsValue::from_str(&e.to_string()));
            callback.emit(response);
        }) as Box<dyn FnMut(JsValue)>);

        let _ = promise.then(&closure);
        closure.forget();
    }
}
