use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use iced::Font;
use iced::font::Weight;
use iced::widget::{button, column, row, text};
use zbus::SignalContext;

use common::dbus::DbusEventViewEvent;
use common::model::PluginId;

use crate::dbus::DbusClient;
use crate::model::{NativeUiPropertyValue, NativeUiWidget, NativeUiWidgetId};
use crate::ui::theme::{ButtonStyle, Element};

#[derive(Clone, Debug)]
pub struct ComponentWidgetWrapper {
    id: NativeUiWidgetId,
    inner: Arc<RwLock<ComponentWidget>>,
}

include!(concat!(env!("OUT_DIR"), "/components.rs"));

#[derive(Clone, Copy)]
pub enum ComponentRenderContext {
    None,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl ComponentWidgetWrapper {
    pub fn widget(id: NativeUiWidgetId, widget_type: &str, properties: HashMap<String, NativeUiPropertyValue>) -> anyhow::Result<Self> {
        Ok(ComponentWidgetWrapper::new(id, create_component_widget(widget_type, properties)?))
    }

    pub fn root(id: NativeUiWidgetId) -> Self {
        ComponentWidgetWrapper::new(id, ComponentWidget::Root { children: vec![] })
    }

    pub fn text_part(id: NativeUiWidgetId, text: &str) -> anyhow::Result<Self> {
        Ok(ComponentWidgetWrapper::new(id, ComponentWidget::TextPart(text.to_owned())))
    }

    fn new(id: NativeUiWidgetId, widget: ComponentWidget) -> Self {
        Self {
            id,
            inner: Arc::new(RwLock::new(widget)),
        }
    }

    fn get(&self) -> RwLockReadGuard<'_, ComponentWidget> {
        self.inner.read().expect("lock is poisoned")
    }

    fn get_mut(&self) -> RwLockWriteGuard<'_, ComponentWidget> {
        self.inner.write().expect("lock is poisoned")
    }

    pub fn render_widget<'a>(&self, context: ComponentRenderContext) -> Element<'a, ComponentWidgetEvent> {
        let widget = self.get();
        match &*widget {
            ComponentWidget::TextPart(text_content) => {
                let size = match context {
                    ComponentRenderContext::None => None,
                    ComponentRenderContext::H1 => Some(34),
                    ComponentRenderContext::H2 => Some(30),
                    ComponentRenderContext::H3 => Some(24),
                    ComponentRenderContext::H4 => Some(20),
                    ComponentRenderContext::H5 => Some(18),
                    ComponentRenderContext::H6 => Some(16),
                };

                let mut text = text(text_content);

                if let Some(size) = size {
                    text = text
                        .size(size)
                        .font(Font {
                            weight: Weight::Bold,
                            ..Font::DEFAULT
                        })
                }

                text.into()
            }
            ComponentWidget::Text { children } => {
                row(render_children(children, context))
                    .into()
            }
            ComponentWidget::Link { children, href } => {
                let content: Element<_> = row(render_children(children, ComponentRenderContext::None))
                    .into();

                button(content)
                    .style(ButtonStyle::Link)
                    .on_press(ComponentWidgetEvent::LinkClick { href: href.to_owned() })
                    .into()
            }
            ComponentWidget::Tag { children, onClick: _, color: _ } => {
                let content: Element<_> = row(render_children(children, ComponentRenderContext::None))
                    .into();

                button(content)
                    .on_press(ComponentWidgetEvent::TagClick { widget: self.as_native_widget() })
                    .into()
            }
            ComponentWidget::MetadataItem { children } => {
                row(render_children(children, ComponentRenderContext::None))
                    .into()
            }
            ComponentWidget::Separator => {
                text("Separator").into()
            }
            ComponentWidget::Metadata { children } => {
                column(render_children(children, ComponentRenderContext::None))
                    .into()
            }
            ComponentWidget::Image => {
                text("Image").into()
            }
            ComponentWidget::H1 { children } => {
                row(render_children(children, ComponentRenderContext::H1))
                    .into()
            }
            ComponentWidget::H2 { children } => {
                row(render_children(children, ComponentRenderContext::H2))
                    .into()
            }
            ComponentWidget::H3 { children } => {
                row(render_children(children, ComponentRenderContext::H3))
                    .into()
            }
            ComponentWidget::H4 { children } => {
                row(render_children(children, ComponentRenderContext::H4))
                    .into()
            }
            ComponentWidget::H5 { children } => {
                row(render_children(children, ComponentRenderContext::H5))
                    .into()
            }
            ComponentWidget::H6 { children } => {
                row(render_children(children, ComponentRenderContext::H6))
                    .into()
            }
            ComponentWidget::HorizontalBreak => {
                text("HorizontalBreak").into()
            }
            ComponentWidget::CodeBlock { children } => {
                text("CodeBlock").into()
            }
            ComponentWidget::Code { children } => {
                text("Code").into()
            }
            ComponentWidget::Content { children } => {
                column(render_children(children, ComponentRenderContext::None))
                    .into()
            }
            ComponentWidget::Detail { children } => {
                let metadata_element = render_child_by_type(children, |widget| matches!(widget, ComponentWidget::Metadata { .. }), ComponentRenderContext::None)
                    .unwrap();
                let content_element = render_child_by_type(children, |widget| matches!(widget, ComponentWidget::Content { .. }), ComponentRenderContext::None)
                    .unwrap();

                row(vec![content_element, metadata_element])
                    .into()
            }
            ComponentWidget::Root { children } => {
                row(render_children(children, ComponentRenderContext::None))
                    .into()
            }
        }
    }

    pub fn append_child(&self, child: ComponentWidgetWrapper) -> anyhow::Result<()> {
        append_component_widget_child(&self, child)
    }

    pub fn get_children(&self) -> anyhow::Result<Vec<ComponentWidgetWrapper>> {
        get_component_widget_children(&self)
    }

    pub fn set_children(&self, new_children: Vec<ComponentWidgetWrapper>) -> anyhow::Result<()> {
        set_component_widget_children(&self, new_children)
    }

    pub fn as_native_widget(&self) -> NativeUiWidget {
        let (internal_name, _) = get_component_widget_type(&self);
        NativeUiWidget {
            widget_id: self.id,
            widget_type: internal_name.to_owned()
        }
    }
}

fn render_children<'a>(
    content: &[ComponentWidgetWrapper],
    context: ComponentRenderContext
) -> Vec<Element<'a, ComponentWidgetEvent>> {
    return content
        .into_iter()
        .map(|child| child.render_widget(context))
        .collect();
}

fn render_child_by_type<'a>(
    content: &[ComponentWidgetWrapper],
    predicate: impl Fn(&ComponentWidget) -> bool,
    context: ComponentRenderContext
) -> anyhow::Result<Element<'a, ComponentWidgetEvent>> {
    let vec: Vec<_> = content
        .into_iter()
        .filter(|child| predicate(&child.get()))
        .collect();

    match vec[..] {
        [] => Err(anyhow::anyhow!("no child matching predicate found")),
        [single] => Ok(single.render_widget(context)),
        [_, _, ..] => Err(anyhow::anyhow!("more than 1 child matching predicate found")),
    }
}

fn render_children_by_type<'a>(
    content: &[ComponentWidgetWrapper], predicate: impl Fn(&ComponentWidget) -> bool,
    context: ComponentRenderContext
) -> Vec<Element<'a, ComponentWidgetEvent>> {
    return content
        .into_iter()
        .filter(|child| predicate(&child.get()))
        .map(|child| child.render_widget(context))
        .collect();
}


#[derive(Clone, Debug)]
pub enum ComponentWidgetEvent {
    LinkClick {
        href: String
    },
    TagClick {
        widget: NativeUiWidget
    },
}

impl ComponentWidgetEvent {
    pub async fn handle(self, signal_context: &SignalContext<'_>, plugin_id: PluginId) {
        match self {
            ComponentWidgetEvent::LinkClick { href } => {
                todo!("href {:?}", href)
            }
            ComponentWidgetEvent::TagClick { widget } => {
                let event_view_event = DbusEventViewEvent {
                    event_name: "onClick".to_owned(),
                    widget: widget.into(),
                };

                DbusClient::view_event_signal(signal_context, &plugin_id.to_string(), event_view_event)
                    .await
                    .unwrap();
            }
        }
    }
}
