use iced_core::{
    event::Status,
    layout::{Limits, Node},
    widget::Tree,
    Clipboard, Element, Event, Layout, Shell, Size, Widget, Rectangle, mouse::Cursor,
};

// pseoudo-widget used for capturing events

pub struct EventCapture<'a, Message>
where
    Message: 'a,
{
    handle: Box<
        dyn Fn(Event, Cursor, &mut dyn Clipboard) -> (Option<Message>, Status)
            + 'a,
    >,
}

impl<'a, Message> EventCapture<'a, Message> {
    pub fn new(
        handle: impl Fn(Event, Cursor, &mut dyn Clipboard) -> (Option<Message>, Status)
            + 'a,
    ) -> Self {
        EventCapture {
            handle: Box::new(handle),
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for EventCapture<'a, Message>
where
    Renderer: iced_core::Renderer,
{
    fn width(&self) -> iced_core::Length {
        0.into()
    }

    fn height(&self) -> iced_core::Length {
        0.into()
    }

    fn layout(&self, _renderer: &Renderer, _limits: &Limits) -> Node {
        Node::new(Size::ZERO)
    }

    fn on_event(
        &mut self,
        _state: &mut Tree,
        event: Event,
        _layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle
    ) -> Status {
        let (msg, status) =
            self.handle.as_ref()(event, cursor, clipboard);
        if let Some(msg) = msg {
            shell.publish(msg);
        }
        status
    }

    fn draw(
        &self,
        _state: &Tree,
        _renderer: &mut Renderer,
        _theme: &<Renderer as iced_core::Renderer>::Theme,
        _style: &iced_core::renderer::Style,
        _layout: Layout<'_>,
        _cursor_position: Cursor,
        _viewport: &Rectangle,
    ) {
        _ = ()
    }
}

impl<'a, Message, Renderer> From<EventCapture<'a, Message>>
    for Element<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn from(value: EventCapture<'a, Message>) -> Self {
        Self::new(value)
    }
}
