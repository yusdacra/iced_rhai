use flume::Receiver;
use iced::futures::stream::BoxStream;
use iced::{
    container, executor, Application, Background, Color, Column, Command, Container, Element,
    Length, Row, Settings, Space, Subscription, Text,
};
use iced_native::subscription::Recipe;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use rhai::{Array, Dynamic, Engine};
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::rc::Rc;

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

fn to_u8(num: Dynamic) -> u8 {
    if num.is::<u8>() {
        num.cast::<u8>()
    } else if num.is::<u16>() {
        num.cast::<u16>() as u8
    } else if num.is::<u32>() {
        num.cast::<u32>() as u8
    } else if num.is::<u64>() {
        num.cast::<u64>() as u8
    } else if num.is::<i64>() {
        num.cast::<i64>() as u8
    } else if num.is::<i32>() {
        num.cast::<i32>() as u8
    } else {
        panic!("number cannot be converted: {:?}", num);
    }
}

fn to_u16(num: Dynamic) -> u16 {
    if num.is::<u16>() {
        num.cast::<u16>()
    } else if num.is::<u8>() {
        num.cast::<u8>() as u16
    } else if num.is::<u32>() {
        num.cast::<u32>() as u16
    } else if num.is::<u64>() {
        num.cast::<u64>() as u16
    } else if num.is::<i64>() {
        num.cast::<i64>() as u16
    } else if num.is::<i32>() {
        num.cast::<i32>() as u16
    } else {
        panic!("number cannot be converted: {:?}", num);
    }
}

fn to_f32(num: Dynamic) -> f32 {
    if num.is::<f32>() {
        num.cast::<f32>()
    } else if num.is::<u8>() {
        num.cast::<u8>() as f32
    } else if num.is::<u32>() {
        num.cast::<u32>() as f32
    } else if num.is::<u64>() {
        num.cast::<u64>() as f32
    } else if num.is::<i64>() {
        num.cast::<i64>() as f32
    } else if num.is::<i32>() {
        num.cast::<i32>() as f32
    } else {
        panic!("number cannot be converted: {:?}", num);
    }
}

fn new_row(children: Array) -> RcRow {
    let owned = children.into_iter().map(dynamic_to_element);
    Rc::new(Row::with_children(owned.collect()))
}

fn new_col(children: Array) -> RcColumn {
    let owned = children.into_iter().map(dynamic_to_element);
    Rc::new(Column::with_children(owned.collect()))
}

fn new_space(width: Length, height: Length) -> RcSpace {
    Rc::new(Space::new(width, height))
}

fn new_space_w(width: Length) -> RcSpace {
    Rc::new(Space::new(width, Length::Shrink))
}

fn new_space_h(height: Length) -> RcSpace {
    Rc::new(Space::new(Length::Shrink, height))
}

const fn length_fill() -> Length {
    Length::Fill
}

const fn length_shrink() -> Length {
    Length::Shrink
}

fn length_portion(units: Dynamic) -> Length {
    Length::FillPortion(to_u16(units))
}

fn length_absolute(units: Dynamic) -> Length {
    Length::Units(to_u16(units))
}

fn rgb(r: Dynamic, g: Dynamic, b: Dynamic) -> Color {
    Color::from_rgb(to_f32(r), to_f32(g), to_f32(b))
}

fn rgb8(r: Dynamic, g: Dynamic, b: Dynamic) -> Color {
    Color::from_rgb8(to_u8(r), to_u8(g), to_u8(b))
}

fn rgba(r: Dynamic, g: Dynamic, b: Dynamic, a: Dynamic) -> Color {
    Color::from_rgba(to_f32(r), to_f32(g), to_f32(b), to_f32(a))
}

fn rgba8(r: Dynamic, g: Dynamic, b: Dynamic, a: Dynamic) -> Color {
    Color::from_rgba8(to_u8(r), to_u8(g), to_u8(b), to_f32(a))
}

fn new_container(widget: Dynamic) -> RcContainer {
    Rc::new(Container::new(dynamic_to_element(widget)))
}

fn container_style(container: RcContainer, style: container::Style) -> RcContainer {
    struct TempStyle(container::Style);

    impl container::StyleSheet for TempStyle {
        fn style(&self) -> container::Style {
            self.0
        }
    }

    Rc::new(rc_to_owned(container).style(TempStyle(style)))
}

fn into_bg(color: Color) -> Option<Background> {
    Some(color.into())
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

        rhai.register_fn("to_u8", to_u8)
            .register_fn("to_u16", to_u16)
            .register_fn("into_element", dynamic_to_element_rc)
            .register_type::<Color>()
            .register_fn("rgb", rgb)
            .register_fn("rgb8", rgb8)
            .register_fn("rgba", rgba)
            .register_fn("rgba8", rgba8)
            .register_fn("into_bg", into_bg)
            .register_type::<Length>()
            .register_fn("len_fill", length_fill)
            .register_fn("len_shrink", length_shrink)
            .register_fn("len_portion", length_portion)
            .register_fn("len_absolute", length_absolute)
            .register_type::<Text>()
            .register_fn("new_text", Text::new::<&str>)
            .register_fn("color", Text::color::<Color>)
            .register_fn("size", Text::size)
            .register_fn("width", Text::width)
            .register_fn("height", Text::height)
            .register_type_with_name::<RcElement>("Element")
            .register_type_with_name::<RcRow>("Row")
            .register_fn("new_row", new_row)
            .register_fn("new_col", new_col)
            .register_type_with_name::<RcColumn>("Column")
            .register_type_with_name::<RcSpace>("Space")
            .register_fn("new_space", new_space)
            .register_fn("new_space_w", new_space_w)
            .register_fn("new_space_h", new_space_h)
            .register_type_with_name::<RcContainer>("Container")
            .register_fn("new_container", new_container)
            .register_fn("style", container_style)
            .register_type_with_name::<container::Style>("ContainerStyle")
            .register_get_set(
                "background",
                |a: &mut container::Style| a.background,
                |a, b| a.background = b,
            )
            .register_fn("default_container_style", container::Style::default)
            .register_type::<Background>();

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
        self.rhai.eval::<Dynamic>(&self.rhai_code).map_or_else(
            |err| Text::new(format!("error evaluating: {}", err)).into(),
            dynamic_to_element,
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

// This is safe on the assumption that there are no other copies left of the passed rc
fn rc_to_owned<T>(rc: Rc<T>) -> T {
    let b = unsafe { std::mem::transmute_copy::<T, T>(rc.as_ref()) };
    std::mem::forget(rc);
    b
}

fn dynamic_to_element(dn: Dynamic) -> Element<'static, Message> {
    if dn.is::<Text>() {
        dn.cast::<Text>().into()
    } else if dn.is::<RcSpace>() {
        rc_to_owned(dn.cast::<RcSpace>()).into()
    } else if dn.is::<RcRow>() {
        rc_to_owned(dn.cast::<RcRow>()).into()
    } else if dn.is::<RcColumn>() {
        rc_to_owned(dn.cast::<RcColumn>()).into()
    } else if dn.is::<RcContainer>() {
        rc_to_owned(dn.cast::<RcContainer>()).into()
    } else if dn.is::<RcElement>() {
        rc_to_owned(dn.cast::<RcElement>())
    } else {
        Text::new(format!("not supported: {:?}", dn)).into()
    }
}

fn dynamic_to_element_rc(dn: Dynamic) -> RcElement {
    Rc::new(dynamic_to_element(dn))
}

type RcRow = Rc<Row<'static, Message>>;
type RcColumn = Rc<Column<'static, Message>>;
type RcElement = Rc<Element<'static, Message>>;
type RcSpace = Rc<Space>;
type RcContainer = Rc<Container<'static, Message>>;
