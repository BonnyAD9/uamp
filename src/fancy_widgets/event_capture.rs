use iced_native::{
    event::Status,
    layout::{Limits, Node},
    widget::Tree,
    Clipboard, Element, Event, Layout, Point, Shell, Size, Widget,
};

// pseoudo-widget used for capturing events

pub struct EventCapture<'a, Message> where Message: 'a {
    handle: Box<
        dyn Fn(Event, Point, &mut dyn Clipboard) -> (Option<Message>, Status) + 'a,
    >,
}

impl<'a, Message> EventCapture<'a, Message> {
    pub fn new(
        handle: impl Fn(Event, Point, &mut dyn Clipboard) -> (Option<Message>, Status)
            + 'a,
    ) -> Self {
        EventCapture {
            handle: Box::new(handle),
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for EventCapture<'a, Message>
where
    Renderer: iced_native::Renderer,
{
    fn width(&self) -> iced_native::Length {
        0.into()
    }

    fn height(&self) -> iced_native::Length {
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
        cursor_position: Point,
        _renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> Status {
        let (msg, status) =
            self.handle.as_ref()(event, cursor_position, clipboard);
        if let Some(msg) = msg {
            shell.publish(msg);
        }
        status
    }

    fn draw(
        &self,
        _state: &iced_native::widget::Tree,
        _renderer: &mut Renderer,
        _theme: &<Renderer as iced_native::Renderer>::Theme,
        _style: &iced_native::renderer::Style,
        _layout: iced_native::Layout<'_>,
        _cursor_position: iced_native::Point,
        _viewport: &iced_native::Rectangle,
    ) {
        _ = ()
    }
}

impl<'a, Message, Renderer> From<EventCapture<'a, Message>>
    for Element<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn from(value: EventCapture<'a, Message>) -> Self {
        Self::new(value)
    }
}
