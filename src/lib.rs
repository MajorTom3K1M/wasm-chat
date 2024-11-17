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
// extern crate stdweb;

extern crate web_sys;

extern crate js_sys;

extern crate serde_wasm_bindgen;

extern crate wasm_bindgen_futures;

use services::{HereNowResponse, PubnubService};
use std::collections::HashSet;
use wasm_bindgen::JsValue;
use web_sys::{HtmlInputElement, InputEvent, KeyboardEvent};
use yew::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub text: String,
    pub from: String,
}

pub struct Model {
    alias: String,
    thisUser: String,
    pending_text: String,
    messages: Vec<Message>,
    users: HashSet<String>,
    pubnub: PubnubService,
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
    FetchHereNow,
    HereNowFetched(Result<HereNowResponse, JsValue>),
    NoOp,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Model {
            messages: Vec::new(),
            thisUser: "".to_owned(),
            alias: "".to_owned(),
            users: HashSet::new(),
            pending_text: "".to_owned(),
            pubnub: PubnubService::new(),
        } 
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        info!("Update: {:?}", msg);
        match msg {
            Msg::AddMessage(msg) => {
                self.messages.push(msg);
            }
            Msg::UserOnline(nick) => {
                info!("Adding user {:?}", nick);
                self.users.insert(nick);
            }
            Msg::UserOffline(nick) => {
                info!("Removing user {:?}", nick);
                self.users.remove(&nick);
            }
            Msg::SendChat => {
                info!("Called send chat!");
                if self.pending_text != "" {
                    self.pubnub.send_message(&self.pending_text);
                    self.pending_text = "".to_owned();
                }
            }
            Msg::Connect => {
                info!("Connected");

                let on_message = ctx.link().callback(|msg| Msg::AddMessage(msg));
                let onoffline = ctx.link().callback(|user| Msg::UserOffline(user));
                let ononline = ctx.link().callback(|user| Msg::UserOnline(user));
                self.thisUser = self.alias.clone();

                self.pubnub.connect(
                    "chatengine-demo-chat",
                    &self.alias,
                    on_message,
                    onoffline,
                    ononline,
                );

                ctx.link().send_message(Msg::FetchHereNow);
            }
            Msg::EnterName(n) => {
                self.alias = n;
            }
            Msg::UpdatePendingText(s) => {
                self.pending_text = s;
            }
            Msg::FetchHereNow => {
                let callback = ctx.link().callback(|response| Msg::HereNowFetched(response));
                let params = services::HereNowParameters {
                    channels: vec!["chatengine-demo-chat".to_owned()],
                    include_uuids: true,
                    include_state: false,
                };

                self.pubnub.fetch_here_now(params, callback);
            }
            Msg::HereNowFetched(Ok(response)) => {
                info!("HereNowFetched: {:?}", response);
                response.channels.unwrap().get("chatengine-demo-chat").map(|channel| {
                    channel.occupants.iter().for_each(|occupant| {
                        info!("Occupant: {:?}", occupant);
                        for occ in occupant.iter() {
                            ctx.link().send_message(Msg::UserOnline(occ.uuid.clone()));
                        }
                    }); 
                });
            }
            Msg::HereNowFetched(Err(e)) => {
                error!("Error fetching here now: {:?}", e);
            }
            Msg::NoOp => {}
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="flex-container">
                <div class="chat-container">
                    <div class="chat-header">
                        <h1 class="chat-header-title">{"Chat Room"}</h1>
                        <button class="chat-header-button">
                            <i class="chat-header-icon fa-regular fa-user"></i>
                        </button>
                    </div>

                    <div class="chat-messages-container">
                        <div class="messages-space">
                            { for self.messages.iter().enumerate().map(|(idx, message)| {
                                view_message((idx, message, self.thisUser.as_str()))
                            }) }
                        </div>
                    </div>

                    <div class="input-container">
                        <div class="connect-form" aria-label="Connect to Chat">
                            <input
                                type="text"
                                placeholder="Enter your username"
                                class="connect-input"
                                value={ self.alias.to_string() }
                                oninput={ ctx.link().callback(|e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    Msg::EnterName(input.value())
                                }) }
                                disabled={ !self.thisUser.is_empty() }
                            />

                            <button
                                class="connect-button"
                                disabled={ !self.thisUser.is_empty() }
                                onclick={ ctx.link().callback(|_| Msg::Connect) }
                            >
                                { if self.thisUser.is_empty() { "Connect" } else { "Connected" } }
                            </button>
                        </div>

                        <div class="connect-form" aria-label="Connect to Chat">
                            <input
                                type="text"
                                placeholder="Type a message..."
                                class="connect-input"
                                disabled={ self.thisUser.is_empty() }
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

                            <button
                                class="send-button"
                                disabled={ self.thisUser.is_empty() }
                                onclick={ ctx.link().callback(|_| Msg::SendChat) }
                            >
                                <i class="send-icon fa-regular fa-paper-plane"></i>
                                {"Send"}
                            </button>
                        </div>
                    </div>
                </div>

                <div class="users-container">
                    <div class="users-header">
                        <h2 class="text-title">{"Users"}</h2>
                    </div>
                    <div class="users-list">
                        { for self.users.iter().enumerate().map(view_user) }
                        <div class="separator"></div>
                    </div>
                </div>
            </div>
        }
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        println!("Destroying");
        self.pubnub.disconnect();
    }
}

fn view_message((_idx, message, this_user): (usize, &Message, &str)) -> Html {
    if this_user == message.from {
        html! {
            <div class="message sent">
                <div class="avatar">
                    <img src="https://via.placeholder.com/40" alt="You" class="avatar-image"/>
                    <div class="avatar-fallback">{"Y"}</div>
                </div>
                <div class="message-content">
                    <p class="message-user">{&message.from}</p>
                    <p class="message-text">{&message.text}</p>
                </div>
            </div>
        }
    } else {
        html! {
            <div class="message received">
                <div class="avatar">
                    <img src="https://via.placeholder.com/40" alt="User" class="avatar-image" />
                    <div class="avatar-fallback">{"U"}</div>
                </div>
                <div class="message-content">
                    <p class="message-user">{&message.from}</p>
                    <p class="message-text">{&message.text}</p>
                </div>
            </div>
        }
    }
}

fn view_user((_idx, user): (usize, &String)) -> Html {
    html! {
        <div class="user">
            <div class="avatar">
                <img src="https://via.placeholder.com/40" class="avatar-image" />
            </div>
            <div>{user}</div>
        </div>
    }
}

pub mod services;
