mod plugin_container;
mod search_list;

use std::sync::{Arc, RwLock};
use deno_core::error::AnyError;

use iced::{Application, Command, Element, Event, executor, futures, keyboard, Length, Padding, Renderer, Subscription, subscription, widget};
use iced::{Settings, window};
use iced::keyboard::KeyCode;
use iced::widget::{column, container, horizontal_rule, scrollable, text_input};
use iced::window::Position;
use zbus::{Connection, InterfaceRef};

use crate::client::dbus::{DbusClient, DbusServerProxyProxy};
use crate::client::model::{NativeUiRequestData, NativeUiResponseData, NativeUiSearchResult};
use crate::client::native_ui::plugin_container::{BuiltInWidgetEvent, ClientContext, plugin_container};
use crate::client::native_ui::search_list::search_list;
use crate::dbus::{DbusEventViewCreated, DbusEventViewEvent};
use crate::utils::channel::{channel, RequestReceiver};

pub struct AppModel {
    client_context: Arc<RwLock<ClientContext>>,
    dbus_connection: Connection,
    dbus_server: DbusServerProxyProxy<'static>,
    dbus_client: InterfaceRef<DbusClient>,
    state: AppState,
    prompt: Option<String>,
    search_results: Vec<NativeUiSearchResult>,
}

enum AppState {
    SearchView,
    PluginView {
        plugin_id: String,
        entrypoint_id: String,
    },
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    OpenView {
        plugin_uuid: String,
        entrypoint_id: String,
    },
    PromptChanged(String),
    SetSearchResults(Vec<NativeUiSearchResult>),
    IcedEvent(Event),
    WidgetEvent {
        plugin_uuid: String,
        widget_event: BuiltInWidgetEvent
    },
    Noop,
}

pub fn run() {
    AppModel::run(Settings {
        id: None,
        window: window::Settings {
            size: (650, 400),
            position: Position::Centered,
            resizable: false,
            decorations: false,
            ..Default::default()
        },
        ..Default::default()
    }).unwrap();
}


impl Application for AppModel {
    type Executor = executor::Default;
    type Message = AppMsg;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {

        let (context_tx, request_rx) = channel::<(String, NativeUiRequestData), NativeUiResponseData>();

        let client_context = Arc::new(RwLock::new(
            ClientContext { containers: Default::default(), }
        ));

        let (dbus_connection, dbus_server, dbus_client) = futures::executor::block_on(async {
            let path = "/org/placeholdername/PlaceHolderName";

            let dbus_connection = zbus::ConnectionBuilder::session()?
                .name("org.placeholdername.PlaceHolderName.Client")?
                .serve_at(path, DbusClient { context_tx })?
                .build()
                .await?;

            let dbus_server = DbusServerProxyProxy::new(&dbus_connection).await?;

            let dbus_client = dbus_connection
                .object_server()
                .interface::<_, DbusClient>(path)
                .await?;

            Ok::<(Connection, DbusServerProxyProxy<'_>, InterfaceRef<DbusClient>), AnyError>((dbus_connection, dbus_server, dbus_client))
        }).unwrap();

        (
            AppModel {
                client_context: client_context.clone(),
                dbus_connection,
                dbus_server,
                dbus_client,
                state: AppState::SearchView,
                prompt: None,
                search_results: vec![]
            },
            Command::batch([
                request_loop(client_context, request_rx),
                Command::perform(async {  }, |_| AppMsg::PromptChanged("".to_owned()))
            ]),
        )
    }

    fn title(&self) -> String {
        "test".to_owned()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMsg::OpenView { plugin_uuid, entrypoint_id } => {
                self.state = AppState::PluginView {
                    plugin_id: plugin_uuid.clone(),
                    entrypoint_id: entrypoint_id.clone(),
                };

                let mut client_context = self.client_context.write().unwrap();
                client_context.create_view_container(&plugin_uuid);

                let dbus_client = self.dbus_client.clone();

                let plugin_uuid = plugin_uuid.clone();

                Command::perform(async move {
                    let event_view_created = DbusEventViewCreated {
                        reconciler_mode: "persistent".to_owned(),
                        view_name: entrypoint_id, // TODO what was view_name supposed to be?
                    };

                    let signal_context = dbus_client.signal_context();

                    DbusClient::view_created_signal(signal_context, &plugin_uuid, event_view_created)
                        .await
                        .unwrap();
                }, |_| AppMsg::Noop)
            }
            AppMsg::PromptChanged(prompt) => {
                self.prompt.replace(prompt.clone());

                let dbus_server = self.dbus_server.clone();

                Command::perform(async move {
                    dbus_server.search(&prompt)
                        .await
                        .unwrap()
                        .into_iter()
                        .map(|search_result| NativeUiSearchResult {
                            plugin_uuid: search_result.plugin_uuid,
                            plugin_name: search_result.plugin_name,
                            entrypoint_id: search_result.entrypoint_id,
                            entrypoint_name: search_result.entrypoint_name,
                        })
                        .collect()
                }, AppMsg::SetSearchResults)
            },
            AppMsg::SetSearchResults(search_results) => {
                self.search_results = search_results;
                Command::none()
            }
            AppMsg::IcedEvent(Event::Keyboard(event)) => {
                match event {
                    keyboard::Event::KeyPressed { key_code, .. } => {
                        match key_code {
                            KeyCode::Up => widget::focus_previous(),
                            KeyCode::Down => widget::focus_next(),
                            _ => Command::none()
                        }
                    }
                    _ => Command::none()
                }
            }
            AppMsg::IcedEvent(_) => Command::none(),
            AppMsg::WidgetEvent { widget_event, plugin_uuid } => {
                match widget_event {
                    BuiltInWidgetEvent::ButtonClick { widget_id } => {
                        let dbus_client = self.dbus_client.clone();

                        Command::perform(async move {
                            let signal_context = dbus_client.signal_context();

                            let event_view_event = DbusEventViewEvent {
                                event_name: "onClick".to_owned(),
                                widget_id,
                            };

                            DbusClient::view_event_signal(&signal_context, &plugin_uuid, event_view_event)
                                .await
                                .unwrap();
                        }, |_| AppMsg::Noop)
                    }
                }
            },
            AppMsg::Noop => Command::none(),
        }
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let client_context = self.client_context.clone();

        match &self.state {
            AppState::SearchView => {
                let input: Element<_> = text_input("", self.prompt.as_ref().unwrap_or(&"".to_owned()))
                    .on_input(AppMsg::PromptChanged)
                    .width(Length::Fill)
                    .into();

                let search_results = self.search_results.iter().cloned().collect();

                let search_list = search_list(search_results, |event| {
                    AppMsg::OpenView {
                        plugin_uuid: event.plugin_uuid,
                        entrypoint_id: event.entrypoint_id
                    }
                });

                let list: Element<_> = scrollable(search_list)
                    .width(Length::Fill)
                    .into();

                let column: Element<_> = column(vec![
                    container(input)
                        .width(Length::Fill)
                        .padding(Padding::new(10.0))
                        .into(),
                    horizontal_rule(1)
                        .into(),
                    container(list)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(Padding::new(5.0))
                        .into(),
                ])
                    .into();

                // column.explain(Color::from_rgb(1f32, 0f32, 0f32))
                column
            }
            AppState::PluginView { plugin_id: plugin_uuid, entrypoint_id } => {
                let container: Element<BuiltInWidgetEvent> = plugin_container(client_context, plugin_uuid.clone())
                    .into();

                container.map(|widget_event| AppMsg::WidgetEvent {
                    plugin_uuid: plugin_uuid.to_owned(),
                    widget_event,
                })
            }
        }
    }

    fn subscription(&self) -> Subscription<AppMsg> {
        subscription::events().map(AppMsg::IcedEvent)
    }
}


fn request_loop(client_context: Arc<RwLock<ClientContext>>, mut request_rx: RequestReceiver<(String, NativeUiRequestData), NativeUiResponseData>) -> Command<AppMsg> {
    Command::perform(async move {
        while let Ok(((plugin_uuid, request_data), responder)) = request_rx.recv().await {
            let mut client_context = client_context.write().unwrap();
            match request_data {
                NativeUiRequestData::GetContainer => {
                    let container = client_context.get_container(&plugin_uuid);

                    let response = NativeUiResponseData::GetContainer { container };

                    responder.respond(response).unwrap()
                }
                NativeUiRequestData::CreateInstance { widget_type, properties } => {
                    let widget = client_context.create_instance(&plugin_uuid, &widget_type, properties);

                    let response = NativeUiResponseData::CreateInstance { widget };

                    responder.respond(response).unwrap()
                }
                NativeUiRequestData::CreateTextInstance { text } => {
                    let widget = client_context.create_text_instance(&plugin_uuid, &text);

                    let response = NativeUiResponseData::CreateTextInstance { widget };

                    responder.respond(response).unwrap()
                }
                NativeUiRequestData::AppendChild { parent, child } => {
                    client_context.append_child(&plugin_uuid, parent, child);
                }
                NativeUiRequestData::CloneInstance { widget_type, properties } => {
                    let widget = client_context.clone_instance(&plugin_uuid, &widget_type, properties);

                    let response = NativeUiResponseData::CloneInstance { widget };

                    responder.respond(response).unwrap()
                }
                NativeUiRequestData::ReplaceContainerChildren { container, new_children } => {
                    client_context.replace_container_children(&plugin_uuid, container, new_children);
                }
            }
        }
    }, |_| AppMsg::Noop)
}