use std::{
    fs::File,
    io::{BufReader, Write},
    path::PathBuf,
};

use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use iced::{
    Element, Length,
    keyboard::{Key, Modifiers},
    widget::{
        Button, Image, TextEditor, column, container,
        image::Handle,
        markdown::{self, Style},
        row, text,
        text_editor::{Action, Content},
        text_input,
    },
};
use iced::{Subscription, keyboard};
use iced_aw::{
    TabBarPosition, Tabs, TimePicker,
    date_picker::Date,
    helpers::date_picker,
    time_picker::{Period, Time},
};
use serde::{Deserialize, Serialize};
struct Post {
    content: Content,
    parsed: Vec<markdown::Item>,
    name: String,
    description: String,
    tags: String,
    image_url: String,
    savepath: Option<PathBuf>,
    selected_tab: TabID,
    date: Date,
    show_picker: bool,
    time: Time,
    show_picker_time: bool,
    image: Option<Handle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlogPost {
    title: String,
    body: String,
    image_url: String,
    summary: String,
    timestamp: DateTime<Utc>,
    tags: Vec<String>,
}

impl From<BlogPost> for Post {
    fn from(post: BlogPost) -> Self {
        Self {
            content: Content::with_text(&post.body),
            parsed: vec![],
            name: post.title,
            description: post.summary,
            tags: post.tags.join(","),
            image_url: post.image_url,
            savepath: None,
            selected_tab: TabID::default(),
            date: Date::from_ymd(
                post.timestamp.year(),
                post.timestamp.month(),
                post.timestamp.day(),
            ),
            show_picker: false,
            time: Time::Hms {
                hour: post.timestamp.hour(),
                minute: post.timestamp.minute(),
                second: post.timestamp.second(),
                period: Period::H24 {},
            },
            show_picker_time: false,
            image: None,
        }
    }
}

impl From<Post> for BlogPost {
    fn from(post: Post) -> Self {
        let tstr = post.time.to_string();
        let mut time_str = tstr.split(":");
        Self {
            title: post.name,
            body: post.content.text(),
            image_url: post.image_url,
            summary: post.description,
            timestamp: Utc
                .with_ymd_and_hms(
                    post.date.year,
                    post.date.month,
                    post.date.day,
                    time_str
                        .next()
                        .unwrap_or("0")
                        .to_string()
                        .parse()
                        .unwrap_or_default(),
                    time_str
                        .next()
                        .unwrap_or("0")
                        .to_string()
                        .parse()
                        .unwrap_or_default(),
                    time_str
                        .next()
                        .unwrap_or("0")
                        .to_string()
                        .parse()
                        .unwrap_or_default(),
                )
                .unwrap(),
            tags: post.tags.split(",").map(|v| v.trim().to_string()).collect(),
        }
    }
}

impl From<&Post> for BlogPost {
    fn from(post: &Post) -> Self {
        let tstr = post.time.to_string();
        let mut time_str = tstr.split(":");
        Self {
            title: post.name.clone(),
            body: post.content.text(),
            image_url: post.image_url.clone(),
            summary: post.description.clone(),
            timestamp: Utc
                .with_ymd_and_hms(
                    post.date.year,
                    post.date.month,
                    post.date.day,
                    time_str
                        .next()
                        .unwrap_or("0")
                        .to_string()
                        .parse()
                        .unwrap_or_default(),
                    time_str
                        .next()
                        .unwrap_or("0")
                        .to_string()
                        .parse()
                        .unwrap_or_default(),
                    time_str
                        .next()
                        .unwrap_or("0")
                        .to_string()
                        .parse()
                        .unwrap_or_default(),
                )
                .unwrap(),
            tags: post.tags.split(",").map(|v| v.trim().to_string()).collect(),
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
enum TabID {
    #[default]
    Content,
    Meta,
}

#[derive(Debug, Clone)]
enum Message {
    LinkClicked(markdown::Url),
    EditContent(Action),
    EditTitle(String),
    EditSummary(String),
    EditTags(String),
    EditImageUrl(String),
    SubmitImageUrl(String),
    TabSelected(TabID),
    LoadFile,
    SaveFile,
    SaveToFile,
    ChooseDate,
    SubmitDate(Date),
    CancelDate,
    ChooseTime,
    SubmitTime(Time),
    CancelTime,
}

fn load_from_file(path: &PathBuf) -> Post {
    println!("{}", path.to_str().unwrap());
    let fileres = File::open(path);
    match fileres {
        Ok(file) => serde_json::from_reader::<_, BlogPost>(BufReader::new(file))
            .unwrap()
            .into(),
        Err(_) => Post::default(),
    }
}

fn select_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_directory("~/Documents/")
        .set_title("Select Post")
        .add_filter("json", &["json"])
        .pick_file()
}

fn save_file(post_name: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_directory("~/Documents/")
        .set_title("Select Post Save Location")
        .set_file_name(post_name.to_lowercase().replace(" ", "-"))
        .add_filter("json", &["json"])
        .set_can_create_directories(true)
        .save_file()
}

fn fetch_image(url: String) -> Result<Handle, String> {
    let resp = reqwest::blocking::get(url).map_err(|e| e.to_string())?;

    let bytes = resp.bytes().map_err(|e| e.to_string())?;
    Ok(Handle::from_bytes(bytes.to_vec()))
}

fn save_to_file(path: &PathBuf, state: &Post) {
    let fileres = File::create(path);
    if let Ok(mut file) = fileres {
        let _ = file.write_all(
            serde_json::to_string(&BlogPost::from(state))
                .unwrap_or_default()
                .as_bytes(),
        );
    }
}

fn content_view(state: &Post) -> Element<'_, Message> {
    let mkdwn: Element<'_, Message> = markdown::view(
        &state.parsed,
        markdown::Settings::default(),
        Style::from_palette(iced::Theme::default().palette()),
    )
    .map(Message::LinkClicked);
    let cfield: Element<'_, Message> = TextEditor::new(&state.content)
        .placeholder("something amazing...")
        .on_action(Message::EditContent)
        .height(Length::Fill)
        .into();
    let tfield: Element<'_, Message> = text_input("title", &state.name)
        .on_input(Message::EditTitle)
        .into();
    let interface = column![tfield, row![cfield, mkdwn]];
    interface.into()
}
fn meta_view(state: &Post) -> Element<'_, Message> {
    let but: Element<'_, Message> = Button::new(text(state.date.to_string()))
        .on_press(Message::ChooseDate)
        .into();

    let datepicker = date_picker(
        state.show_picker,
        state.date,
        but,
        Message::CancelDate,
        Message::SubmitDate,
    );

    let but: Element<'_, Message> = Button::new(text(state.time.to_string()))
        .on_press(Message::ChooseTime)
        .into();

    let timepicker = TimePicker::new(
        state.show_picker_time,
        state.time,
        but,
        Message::CancelTime,
        Message::SubmitTime,
    )
    .use_24h()
    .show_seconds();

    let summary: Element<'_, Message> = text_input(
        "How to cook bread in just three easy steps...",
        &state.description,
    )
    .on_input(Message::EditSummary)
    .into();

    let image_url: Element<'_, Message> = text_input("/assets/bred.png", &state.image_url)
        .on_input(Message::EditImageUrl)
        .on_submit(Message::SubmitImageUrl(state.image_url.clone()))
        .into();

    let tags: Element<'_, Message> = text_input("cooking,bread,dough,easy,quick,...", &state.tags)
        .on_input(Message::EditTags)
        .into();

    let interface = column![
        text("Summary"),
        summary,
        text("Tags (seperated by ,'s)"),
        tags,
        text("Image Url (can be relative but won't show)"),
        image_url,
        row![text("premier date: "), datepicker],
        row![text("premier time: "), timepicker],
    ];
    if state.image.is_some() {
        let image = Image::new(state.image.clone().unwrap());

        interface.push(image).into()
    } else {
        interface.into()
    }
}

impl Post {
    fn update(&mut self, message: Message) {
        match message {
            Message::LinkClicked(url) => {
                let _ = webbrowser::open(url.as_str());
            }
            Message::EditContent(action) => {
                self.content.perform(action);
            }
            Message::TabSelected(id) => {
                self.selected_tab = id;
            }
            Message::EditTitle(title) => self.name = title,
            Message::EditSummary(summary) => self.description = summary,
            Message::EditTags(tags) => self.tags = tags,
            Message::EditImageUrl(url) => self.image_url = url,
            Message::SubmitImageUrl(url) => {
                let image = fetch_image(url);
                match image {
                    Ok(handle) => self.image = Some(handle),
                    Err(_) => self.image = None,
                }
            }
            Message::ChooseDate => {
                self.show_picker = true;
            }
            Message::SubmitDate(date) => {
                self.date = date;
                self.show_picker = false;
            }
            Message::CancelDate => {
                self.show_picker = false;
            }
            Message::ChooseTime => {
                self.show_picker_time = true;
            }
            Message::SubmitTime(time) => {
                self.time = time;
                self.show_picker_time = false;
            }
            Message::CancelTime => {
                self.show_picker_time = false;
            }
            Message::LoadFile => {
                let path = select_file();
                if path.is_some() {
                    let mut new_state = load_from_file(&path.clone().unwrap());
                    new_state.savepath = path;
                    new_state.selected_tab = self.selected_tab.clone();
                    *self = new_state;
                }
            }
            Message::SaveFile => {
                if self.savepath.is_none() {
                    let path = save_file(&self.name);
                    if path.is_some() {
                        self.savepath = path;
                    }
                }
                if self.savepath.is_some() {
                    save_to_file(&self.savepath.clone().unwrap(), self);
                }
            }
            Message::SaveToFile => {
                self.savepath = save_file(&self.name);
                if self.savepath.is_some() {
                    save_to_file(&self.savepath.clone().unwrap(), self);
                }
            } //_ => {}
        }
        self.parsed = markdown::parse(&self.content.text()).collect();
    }
    fn view(&self) -> Element<'_, Message> {
        Tabs::new(Message::TabSelected)
            .push(
                TabID::Content,
                iced_aw::TabLabel::Text("Content".to_string()),
                container(content_view(self)).height(Length::Fill),
            )
            .push(
                TabID::Meta,
                iced_aw::TabLabel::Text("Meta".to_string()),
                container(meta_view(self)).height(Length::Fill),
            )
            .set_active_tab(&self.selected_tab)
            .tab_bar_position(TabBarPosition::Bottom)
            .into()
    }
}

impl Default for Post {
    fn default() -> Self {
        Self {
            content: Content::default(),
            parsed: vec![],
            name: "".to_string(),
            description: "".to_string(),
            tags: "".to_string(),
            image_url: "".to_string(),
            savepath: None,
            selected_tab: TabID::default(),
            date: Date::today(),
            show_picker: false,
            time: Time::now_hms(true),
            show_picker_time: false,
            image: None,
        }
    }
}

fn subscription(_: &Post) -> Subscription<Message> {
    keyboard::on_key_press(
        |key: Key, modifiers: Modifiers| match (key.as_ref(), modifiers) {
            (Key::Character("s"), m) if m.command() && m.shift() => Some(Message::SaveToFile),
            (Key::Character("s"), m) if m.command() => Some(Message::SaveFile),
            (Key::Character("o"), m) if m.command() => Some(Message::LoadFile),
            _ => None,
        },
    )
}

fn main() -> iced::Result {
    iced::application("Blog Creator", Post::update, Post::view)
        .subscription(subscription)
        .centered()
        .font(iced_fonts::REQUIRED_FONT_BYTES)
        .run()
}
