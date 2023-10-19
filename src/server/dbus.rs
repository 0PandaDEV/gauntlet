use std::fmt::Debug;

use crate::common::dbus::{DbusEventViewCreated, DbusEventViewEvent, DBusPlugin, DBusSearchResult, DBusUiPropertyContainer, DBusUiWidget};
use crate::common::model::{EntrypointId, PluginId};
use crate::server::plugins::PluginManager;
use crate::server::search::SearchIndex;

pub struct DbusServer {
    pub search_index: SearchIndex,
}

#[zbus::dbus_interface(name = "org.placeholdername.PlaceHolderName")]
impl DbusServer {
    fn search(&self, text: &str) -> Vec<DBusSearchResult> {
        self.search_index.create_handle()
            .search(text)
            .unwrap()
            .into_iter()
            .map(|item| {
                DBusSearchResult {
                    entrypoint_name: item.entrypoint_name,
                    entrypoint_id: item.entrypoint_id,
                    plugin_name: item.plugin_name,
                    plugin_id: item.plugin_id,
                }
            })
            .collect()
    }
}


pub struct DbusManagementServer {
    pub plugin_manager: PluginManager,
}

#[zbus::dbus_interface(name = "org.placeholdername.PlaceHolderName.Management")]
impl DbusManagementServer {
    fn plugins(&mut self) -> Vec<DBusPlugin> {
        self.plugin_manager.plugins()
    }

    fn set_plugin_state(&mut self, plugin_id: &str, enabled: bool) {
        println!("set_plugin_state {:?} {:?}", plugin_id, enabled);
        self.plugin_manager.set_plugin_state(PluginId::new(plugin_id), enabled)
    }

    fn set_entrypoint_state(&mut self, plugin_id: &str, entrypoint_id: &str, enabled: bool) {
        println!("set_entrypoint_state {:?} {:?}", plugin_id, enabled);
        self.plugin_manager.set_entrypoint_state(PluginId::new(plugin_id), EntrypointId::new(entrypoint_id), enabled)
    }
}


#[zbus::dbus_proxy(
default_service = "org.placeholdername.PlaceHolderName.Client",
default_path = "/org/placeholdername/PlaceHolderName",
interface = "org.placeholdername.PlaceHolderName.Client",
)]
trait DbusClientProxy {
    #[dbus_proxy(signal)]
    fn view_created_signal(&self, plugin_id: &str, event: DbusEventViewCreated) -> zbus::Result<()>;

    #[dbus_proxy(signal)]
    fn view_event_signal(&self, plugin_id: &str, event: DbusEventViewEvent) -> zbus::Result<()>;

    fn get_container(&self, plugin_id: &str) -> zbus::Result<DBusUiWidget>;

    fn create_instance(&self, plugin_id: &str, widget_type: &str, properties: DBusUiPropertyContainer) -> zbus::Result<DBusUiWidget>;

    fn create_text_instance(&self, plugin_id: &str, text: &str) -> zbus::Result<DBusUiWidget>;

    fn append_child(&self, plugin_id: &str, parent: DBusUiWidget, child: DBusUiWidget) -> zbus::Result<()>;

    fn remove_child(&self, plugin_id: &str, parent: DBusUiWidget, child: DBusUiWidget) -> zbus::Result<()>;

    fn insert_before(&self, plugin_id: &str, parent: DBusUiWidget, child: DBusUiWidget, before_child: DBusUiWidget) -> zbus::Result<()>;

    fn set_properties(&self, plugin_id: &str, widget: DBusUiWidget, properties: DBusUiPropertyContainer) -> zbus::Result<()>;

    fn set_text(&self, plugin_id: &str, widget: DBusUiWidget, text: &str) -> zbus::Result<()>;

    fn clone_instance(&self, plugin_id: &str, widget_type: &str, properties: DBusUiPropertyContainer) -> zbus::Result<DBusUiWidget>;

    fn replace_container_children(&self, plugin_id: &str, container: DBusUiWidget, new_children: Vec<DBusUiWidget>) -> zbus::Result<()>;
}
