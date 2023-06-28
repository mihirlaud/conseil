use crate::widgets::content::Content;

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;

use iced::theme::Theme;
use iced::widget::{button, column, horizontal_rule, row, scrollable, text, text_input};
use iced::{Application, Element, Length, Color, Command};
use git2::{ObjectType, Repository, Oid};
use iced_aw::{Split, split};
use iced_native::theme;
use iced_native::widget::{horizontal_space, vertical_space, pick_list};
use toml::Value;

use super::config::Config;

#[derive(Default)]
pub struct ConseilApp {
    config: Config,
    search_input: String,
    search_results: Vec<String>,
    commit_id: Option<String>,
    commit_id_arr: Vec<String>,
    commit_lookup_table: HashMap<String, String>,
    repo: Option<Repository>,
    repo_name: String,
    scroll_content: Vec<Content>,
    inputs: Vec<Vec<String>>,
    vert_divider_pos: Option<u16>,
}

#[derive(Debug, Clone)]
pub enum Message {
    OnVertResize(u16),
    SearchInputChanged(String),
    CommitIDSelected(String),
    HeadingInputChanged(usize, String),
    SubheadingInputChanged(usize, String),
    ParagraphInputChanged(usize, String),
    RepoButtonPressed(String),
    ExportButtonPressed,
}

impl Application for ConseilApp {
    type Executor = iced::executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let config = Config::from_path("configs/default.toml");
        let inputs = vec![vec![], vec![], vec![]];

        (Self {
            config,
            inputs,
            vert_divider_pos: Some(300),
            ..Default::default()
        }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Conseil v0.1")
    }

    fn update(&mut self, message: Message) -> Command<Message> {

        match message {
            Message::OnVertResize(pos) => self.vert_divider_pos = Some(
                if pos < 300 {
                    300
                } else if pos > 500 {
                    500
                } else {
                    pos
                }
            ),
            Message::SearchInputChanged(value) => {
                self.search_input = value;
                self.search_results.clear();

                let dir = Path::new(&self.search_input);

                if dir.is_dir() {
                    match fs::read_dir(dir) {
                        Ok(entries) => {
                            for entry in entries {
                                match entry {
                                    Ok(e) => {
                                        let path = e.path();
                                        if path.is_dir() {
                                            self.search_results.push(path.to_str().unwrap().to_string());
                                        }
                                    }
                                    Err(_) => ()
                                }
                            }
                        }
                        Err(_) => ()
                    }
                    
                }
            }
            Message::CommitIDSelected(value) => {
                self.commit_id = Some(value);
                self.write_content();
            }
            Message::HeadingInputChanged(index, value) => self.inputs[0][index] = value,
            Message::SubheadingInputChanged(index, value) => self.inputs[1][index] = value,
            Message::ParagraphInputChanged(index, value) => self.inputs[2][index] = value,
            Message::RepoButtonPressed(path) => {
                self.repo = match Repository::open(path.as_str()) {
                    Ok(repo) => Some(repo),
                    Err(_) => None,
                };

                self.repo_name = path.as_str().to_string();

                self.scroll_content.clear();

                self.commit_id = None;
                self.commit_id_arr.clear();

                match self.repo {
                    None => {}
                    Some(_) => {
                        let repo = self.repo.as_ref().unwrap();

                        let head = repo.head().expect("Could not find head");
                        let obj = head.resolve().expect("Could not resolve").peel(ObjectType::Commit).expect("Could not peel to commit");
                        let mut commit = obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit")).expect("Could not find head");

                        let key = format!("{} : {}...", commit.message().unwrap().trim(), commit.id().to_string().get(0..6).unwrap());
                        let value = commit.id().to_string();
                        self.commit_lookup_table.insert(key.clone(), value);
                        self.commit_id_arr.push(key);

                        loop {
                            match commit.parent(0) {
                                Err(_) => break,
                                Ok(parent) => {
                                    let key = format!("{} : {}...", parent.message().unwrap().trim(), parent.id().to_string().get(0..6).unwrap());
                                    let value = parent.id().to_string();
                                    self.commit_lookup_table.insert(key.clone(), value);
                                    self.commit_id_arr.push(key);
                                    commit = parent;
                                }
                            }
                        }
                    }
                }

            }
            Message::ExportButtonPressed => {
                self.make_markdown_file();
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {

        let title = text("Conseil").size(48).style(self.theme().palette().primary);

        let repo_text = text(format!("Repository: {}", match self.repo_name.as_str() {
            "" => "None selected",
            _ => self.repo_name.as_str(),
        })).size(32);
                
        let commit_picker = pick_list(
            self.commit_id_arr.clone(), 
            self.commit_id.clone(), 
            Message::CommitIDSelected
        ).width(Length::Fill);

        let export_button = button("Export")
            .padding(10)
            .on_press(Message::ExportButtonPressed);

        let commit_info = scrollable(
            self.scroll_content.iter().fold(
                column![].width(Length::Fill),
                |column, content| {
                    column.push(match content {
                        Content::Heading(index, _) => column![
                            text_input("Heading", &self.inputs[0][*index])
                                .on_input(|s| Message::HeadingInputChanged(*index, s))
                                .size(60).padding(10),
                            vertical_space(20.0),
                        ],
                        Content::Subheading(index, _) => {
                            column![
                                text_input("Subheading", &self.inputs[1][*index])
                                    .on_input(|s| Message::SubheadingInputChanged(*index, s))
                                    .size(32)
                                    .padding(10)
                            ]
                        }
                        Content::Filename(line) => column![text(format!("File: {}", line))],
                        Content::Paragraph(index, _) => {
                            column![
                                text_input("Paragraph", &self.inputs[2][*index])
                                    .on_input(|s| Message::ParagraphInputChanged(*index, s))
                            ]
                        },
                        Content::Hunk(arr) => arr.iter().fold(
                            column![horizontal_rule(10.0)].width(Length::Fill),
                            |column, elem| {
                                let (line, color) = elem;
                                column.push(text(line).style(color.clone()))
                            }
                        ).push(horizontal_rule(10.0)).push(vertical_space(20.0))
                    })
                },
            )
        ).height(Length::Fill);

        let content = column![
            row![title, horizontal_space(Length::Fill), export_button],
            horizontal_rule(5),
            repo_text,
            commit_picker,
            horizontal_rule(5),
            commit_info,
        ]
        .spacing(20)
        .padding(20);

        let results = scrollable(
            self.search_results.iter().fold(
                column![],
                |column, result| {
                    column.push(
                        button(result.as_str())
                            .padding(10)
                            .width(Length::Fill)
                            .on_press(Message::RepoButtonPressed(result.clone()))
                    )
                }
            ));

        let sidebar = column![
            text_input("Search for repo...", &self.search_input)
                .on_input(Message::SearchInputChanged)
                .size(20)
                .padding(10),
            results
        ]
        .spacing(20)
        .padding(20);

        let split = Split::new(
            sidebar,
            content,
            self.vert_divider_pos,
            split::Axis::Vertical,
            Message::OnVertResize,
        ).into();

        split
    }

    fn theme(&self) -> Theme {
        Theme::custom(theme::Palette {
            background: Color::from_rgb(0.2148, 0.2266, 0.2109),
            primary: Color::from_rgb(0.8047, 0.7188, 0.5313),

            ..Theme::Dark.palette()
        })
    }
}

impl ConseilApp {

    fn write_content(&mut self) {
        let repo = match self.repo {
            Some(_) => self.repo.as_ref().unwrap(),
            None => return
        };
                
        self.scroll_content.clear();
        self.inputs = vec![vec![], vec![], vec![]];

        for piece in self.config.get_intro_content() {
            match piece {
                Value::String(s) => match s.as_str() {
                    "heading" => Self::add_text_edit(&mut self.inputs, &mut self.scroll_content, 0),
                    "subheading" => Self::add_text_edit(&mut self.inputs, &mut self.scroll_content, 1),
                    "paragraph" => Self::add_text_edit(&mut self.inputs, &mut self.scroll_content, 2),
                    _ => {}
                }
                _ => {}
            }
        }

        let cid = match self.commit_id.clone() {
            None => "".to_string(),
            Some(s) => self.commit_lookup_table[&s].clone(),
        };
                
        let oid = Oid::from_str(cid.as_str());
        let commit = match oid {
            Ok(o) => {
                repo.find_commit(o)
            }
            Err(_) => {
                println!("Not valid commit OID, defaulting to latest commit");

                let head = repo.head().expect("Could not find head");
                let obj = head.resolve().expect("Could not resolve").peel(ObjectType::Commit).expect("Could not peel to commit");
                obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))
            }
        }.expect("Could not find commit");
                
        let parent = commit.parent(0).expect("Could not find parent commit");

        let parent_tree = parent.tree().expect("Could not find parent tree");
        let commit_tree = commit.tree().expect("Could not find commit tree");

        let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None).expect("Could not find diff");

        let mut current_delta: String = String::new();
        let mut current_hunk: Vec<(String, Color)> = vec![];
        diff.print(git2::DiffFormat::Patch, |delta, _, line| {
            if line.origin() == 'H' || line.origin() == 'F' {
                true
            } else {
                if current_delta != delta.old_file().path().unwrap().to_string_lossy().to_string() {
                    if !current_hunk.is_empty() {
                        for piece in self.config.get_hunk_content() {
                            match piece {
                                Value::String(s) => match s.as_str() {
                                    "subheading" => Self::add_text_edit(&mut self.inputs, &mut self.scroll_content, 1),
                                    "paragraph" => Self::add_text_edit(&mut self.inputs, &mut self.scroll_content, 2),
                                    "filename" => self.scroll_content.push(Content::Filename(current_delta.clone())),
                                    "diff" => self.scroll_content.push(Content::Hunk(current_hunk.clone())),
                                    _ => {}
                                }
                                _ => {}
                            }
                        }
                        current_hunk.clear();
                    }
                    current_delta = delta.old_file().path().unwrap().to_string_lossy().to_string();
                }
                match std::str::from_utf8(line.content()) {
                    Ok(s) => current_hunk.push((
                            format!("{}{}", line.origin(), s),
                            match line.origin() {
                                '+' => Color::from([0.0, 1.0, 0.0]),
                                '-' => Color::from([1.0, 0.0, 0.0]),
                                _ => Color::WHITE,
                            }
                        )),
                    Err(e) => println!("{}", e)
                }
                    
                true
            }
        }).expect("Failed to iterate");

        for piece in self.config.get_outro_content() {
            match piece {
                Value::String(s) => match s.as_str() {
                    "heading" => Self::add_text_edit(&mut self.inputs, &mut self.scroll_content, 0),
                    "subheading" => Self::add_text_edit(&mut self.inputs, &mut self.scroll_content, 1),
                    "paragraph" => Self::add_text_edit(&mut self.inputs, &mut self.scroll_content, 2),
                    _ => {}
                }
                _ => {}
            }
        }
    }

    fn add_text_edit(
        inputs: &mut Vec<Vec<String>>, 
        scroll_content: &mut Vec<Content>, 
        selector: usize) {
        let idx = inputs[selector].len();
        let txt = format!("{} {}", match selector {
            0 => "Heading",
            1 => "Subheading",
            _ => "Paragraph",
        }, idx + 1);
        scroll_content.push(match selector {
            0 => Content::Heading(idx, txt.clone()),
            1 => Content::Subheading(idx, txt.clone()),
            _ => Content::Paragraph(idx, txt.clone()),
        });
        inputs[selector].push(txt);
    }

    fn make_markdown_file(&self) {
        let path = Path::new("markdown/entry.md");
        let display = path.display();

        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => file,
        };

        self.scroll_content.iter().map(|content| {
            match content {
                Content::Heading(idx, _) => Content::Heading(*idx, self.inputs[0][*idx].clone()),
                Content::Subheading(idx, _) => Content::Subheading(*idx, self.inputs[1][*idx].clone()),
                Content::Paragraph(idx, _) => Content::Paragraph(*idx, self.inputs[2][*idx].clone()),
                _ => content.clone(),
            }
        }).for_each(|content| {
            match file.write_all(content.to_md_string().as_bytes()) {
                Err(why) => panic!("couldn't write to {}: {}", display, why),
                Ok(_) => {},
            }
        })
    }
}