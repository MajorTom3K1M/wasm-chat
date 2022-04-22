use super::Message;
use js_sys::JSON;
use stdweb::Value;
use web_sys::console::info;
use yew::prelude::*;
use wasm_bindgen::prelude::*;

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

    #[wasm_bindgen(method, structural)]
    fn setUUID(this: &PubNub, uuid: String);

    #[wasm_bindgen(method, structural)]
    fn getSubscribedChannels(this: &PubNub) -> js_sys::Array;
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Initialize {
    pub publish_key: String,
    pub subscribe_key: String,
}
pub struct PubnubService {
    lib: Option<PubNub>,
    chat: Option<Value>,
    channel: Option<String>
}

#[derive(Serialize)]
pub struct Channels {
    pub channels: Vec<String>,
    pub withPresence: bool
}

#[derive(Debug, Deserialize, Serialize)]
struct EventStatus {
    pub category: String,
    pub operation: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PubNubMessage {
    channel: String,
    publisher: String,
    message: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Presence {
    pub uuid: String
}

impl PubnubService {
    pub fn new() -> Self {
        let init = Initialize {
            publish_key: "pub-c-c05c98be-8fc6-4a7f-8353-853b4157c8a4".to_owned(),
            subscribe_key: "sub-c-79016488-a6ae-11ec-b6b1-728be7898cef".to_owned(),
        };
        let chat_engine = PubNub::new(JsValue::from_serde(&init).unwrap());
        PubnubService {
            lib: Some(chat_engine),
            chat: None,
            channel: None
        }
    }

    pub fn send_message(&mut self, msg: &str) {
        let lib = self.lib.as_ref().expect("No pubnub library!");

        let payload = JSON::parse(&format!(r#"{{
            "channel": "{}",
            "message": "{}"
        }}"#, self.channel.as_ref().unwrap(), msg));

        lib.publish(payload.unwrap());
    }

    pub fn connect(
        &mut self,
        topic: &str,
        nickname: &str,
        onmessage: Callback<Message>,
        onoffline: Callback<String>,
        ononline: Callback<String>
    ) -> () {
        let lib = self.lib.as_ref().expect("No pubnub library!");

        lib.setUUID(nickname.to_string());

        let chat_callback = move |text: String, source: String| {
            let msg = Message {
                text: text,
                from: source
            };
            onmessage.emit(msg);
        };
        let useroffline_callback = move | username: String | {
            onoffline.emit(username);
        };
        let useronline_callback = move |username: String| {
            ononline.emit(username);
        };

        let arr = lib.getSubscribedChannels();

        if arr.index_of(&JsValue::from_str(topic), 0) < 0 {
            let channels = Channels {
                channels: vec![topic.to_owned()],
                withPresence: true
            };
            self.channel = Some(topic.to_owned());

            lib.subscribe(JsValue::from_serde(&channels).unwrap());
    
            let event_handler = js_sys::Object::new();
            let _ = js_sys::Reflect::set(
                &event_handler, 
                &"status".into(), 
                &Closure::wrap(Box::new(move |status_event: JsValue| { 
                    let events: EventStatus = status_event.into_serde::<EventStatus>().unwrap();
                    info!("{:?}", events.category == "PNConnectedCategory");
                }) as Box<dyn FnMut(JsValue)>).into_js_value()
            );
            let _ = js_sys::Reflect::set(
                &event_handler,
                &"message".into(),
                &Closure::wrap(Box::new(move |message: JsValue| {
                    let msg: PubNubMessage = message.into_serde::<PubNubMessage>().unwrap();
                    chat_callback(msg.message, msg.publisher);
                }) as Box<dyn FnMut(JsValue)>).into_js_value()
            );
            let _ = js_sys::Reflect::set(
                &event_handler,
                &"presence".into(),
                &Closure::wrap(Box::new(move |presence: JsValue| {
                    let pres: Presence = presence.into_serde::<Presence>().unwrap();
                    useronline_callback(pres.uuid);
                }) as Box<dyn FnMut(JsValue)>).into_js_value()
            );
            lib.addListener(event_handler);
        }
    }
}