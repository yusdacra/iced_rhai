use flume::Receiver;
use iced::futures::stream::BoxStream;
use iced::{executor, Application, Color, Command, Element, Settings, Subscription, Text};
use iced_native::subscription::Recipe;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use rhai::Engine;
use std::convert::TryInto;
use std::hash::{Hash, Hasher};
use std::path::Path;

fn main() {
    let (tx, rx) = flume::bounded(8);

    let path = Path::new("src/view.rhai");

    let mut watcher = RecommendedWatcher::new_immediate(move |res: notify::Result<Event>| {
        if res.is_ok() {
            tx.send(std::fs::read(path).unwrap()).unwrap();
        }
    })
    .unwrap();

    watcher.watch(path, RecursiveMode::NonRecursive).unwrap();

    State::run(Settings::with_flags((
        rx,
        std::fs::read_to_string(path).unwrap(),
    )))
    .unwrap();
}

fn to_u8<Num>(num: Num) -> u8
where
    Num: TryInto<u8>,
    Num::Error: std::fmt::Debug,
{
    num.try_into().expect("number cannot be converted to u8")
}

fn into_element<T: Into<Element<'static, Message>> + Clone>(widget: T) -> WrapperElement<Message> {
    WrapperElement(widget.into())
}

#[derive(Debug, Clone)]
enum Message {
    NewCode(String),
}

struct State {
    rhai: Engine,
    rhai_code: String,
    notify_rx: Receiver<Vec<u8>>,
}

impl Application for State {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = (Receiver<Vec<u8>>, String);

    fn new((rx, rhai_code): Self::Flags) -> (Self, Command<Message>) {
        let mut rhai = Engine::new();

        rhai.register_fn("to_u8", to_u8::<i64>)
            .register_fn("to_u8", to_u8::<i32>)
            .register_fn("into_element", into_element::<Text>)
            // .register_fn("into_element", into_element::<Container<Message>>) does not work, Container doesnt implement Clone
            .register_type::<Color>()
            .register_fn("new_color", Color::from_rgb8)
            .register_type::<Text>()
            .register_fn("new_text", Text::new::<&str>)
            .register_fn("color", Text::color::<Color>)
            .register_type_with_name::<WrapperElement<Message>>("Element");

        (
            Self {
                rhai,
                rhai_code,
                notify_rx: rx,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "iced rhai example".to_string()
    }

    fn update(&mut self, message: Self::Message, _: &mut iced::Clipboard) -> Command<Message> {
        match message {
            Message::NewCode(code) => {
                self.rhai_code = code;
            }
        }

        Command::none()
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        self.rhai
            .eval::<WrapperElement<Self::Message>>(&self.rhai_code)
            .map_or_else(
                |err| Text::new(format!("error evaluating: {}", err)).into(),
                |a| a.0,
            )
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::from_recipe(ReloadFile {
            notify_rx: self.notify_rx.clone(),
        })
        .map(|data| Message::NewCode(unsafe { String::from_utf8_unchecked(data) }))
    }
}

struct ReloadFile {
    notify_rx: Receiver<Vec<u8>>,
}

impl<H, I> Recipe<H, I> for ReloadFile
where
    H: Hasher,
{
    type Output = Vec<u8>;

    fn hash(&self, state: &mut H) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
    }

    fn stream(self: Box<Self>, _: BoxStream<I>) -> BoxStream<Self::Output> {
        Box::pin(self.notify_rx.into_stream())
    }
}

// This is needed because iced Element does not implement Clone...
struct WrapperElement<M>(Element<'static, M>);

impl<M> Clone for WrapperElement<M> {
    fn clone(&self) -> Self {
        panic!("why is clone needed anyways? it works fine without it!")
    }
}
