#![recursion_limit = "512"]

extern crate strum;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate wasm_bindgen;

extern crate yew;
#[macro_use]
extern crate log;
#[macro_use]
extern crate stdweb;

extern crate web_sys;

extern crate js_sys;

use services::PubnubService;
use std::collections::HashSet;
use yew::prelude::*;
use web_sys::{ InputEvent, KeyboardEvent, HtmlInputElement };

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub text: String,
    pub from: String
}

pub struct Model {
    alias: String,
    pending_text: String,
    messages: Vec<Message>,
    users: HashSet<String>,
    pubnub: PubnubService
}

#[derive(Debug)]
pub enum Msg {
    SendChat,
    AddMessage(Message),
    Connect,
    EnterName(String),
    UserOffline(String),
    UserOnline(String),
    UpdatePendingText(String),
    NoOp
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Model {
            messages: Vec::new(),
            alias: "".to_owned(),
            users: HashSet::new(),
            pending_text: "".to_owned(),
            pubnub: PubnubService::new()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddMessage(msg) => {
                self.messages.push(msg);
            },
            Msg::UserOnline(nick) => {
                info!("Adding user {:?}", nick);
                self.users.insert(nick);
            },
            Msg::UserOffline(nick) => {
                info!("Removing user {:?}", nick);
                self.users.remove(&nick);
            },
            Msg::SendChat => {
                info!("Called send chat!");
                if self.pending_text != "" {
                    self.pubnub.send_message(&self.pending_text);
                    self.pending_text = "".to_owned();
                }
            },
            Msg::Connect => {
                info!("Connected");
                
                let on_message = ctx.link().callback(|msg| Msg::AddMessage(msg));
                let onoffline = ctx.link().callback(|user| Msg::UserOffline(user));
                let ononline = ctx.link().callback(|user| Msg::UserOnline(user));
                
                self.pubnub.connect(
                    "chatengine-demo-chat",
                    &self.alias, 
                    on_message, 
                    onoffline, 
                    ononline
                );
            },
            Msg::EnterName(n) => {
                info!("Enter Name {:?}", n);
                self.alias = n;
            },
            Msg::UpdatePendingText(s) => {
                self.pending_text = s;
            },
            Msg::NoOp => {}
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="wrapper">
                <div class="chat-text">
                    <h1>{ "Messages" }</h1><br/>
                    <ul class="message-list">
                        { for self.messages.iter().enumerate().map(view_message) }
                    </ul>
                </div>
                <div class="users">
                    <h1>{ "Users" }</h1>
                    <ul class="user-list">
                        { for self.users.iter().enumerate().map(view_user) }
                    </ul>
                </div>
                <div class="connect">
                    <input placeholder="Your Name" 
                        value={ self.alias.to_string() }  
                        oninput={ ctx.link().callback(|e: InputEvent| { 
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::EnterName(input.value())
                        }) }
                    />
                    <button onclick={ ctx.link().callback(|_| Msg::Connect) }>{ "Connect" }</button>
                </div>
                <div class="text-entry">
                    <input 
                        placeholder="Message Text"
                        class="pending-text"
                        value={ self.pending_text.to_string() }
                        oninput={ ctx.link().callback(|e: InputEvent| { 
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::UpdatePendingText(input.value())
                        }) }
                        onkeypress={ ctx.link().callback(|e: KeyboardEvent| {
                            if e.key() == "Enter" {
                                Msg::SendChat
                            } else {
                                Msg::NoOp
                            }
                        }) }
                    />
                </div>
            </div>
        }
    }
}

fn view_message((_idx, message): (usize, &Message)) -> Html {
    html! {
        <li>
            <label>
                <span class="sender">{"["}{&message.from}{"]"}</span>
                <span class="chatmsg">{&message.text}</span>
            </label>
        </li>
    }
}

fn view_user((_idx, user): (usize, &String)) -> Html {
    html! {
        <li>
            <label>{ user }</label>
        </li>
    }
}

pub mod services;