use crate::widgets::content::Content;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use iced::theme::Theme;
use iced::widget::{
    button, column, container, horizontal_rule,
    row, scrollable, text, text_input,
};
use iced::{Application, Element, Length, Color, Command};
use git2::{ObjectType, Repository, Oid};
use iced_native::widget::{horizontal_space, vertical_space};

#[derive(Default)]
pub struct ConseilApp {
    path_input: String,
    commit_id_input: String,
    repo: Option<Repository>,
    scroll_content: Vec<Content>,
    subheading_inputs: Vec<String>,
    paragraph_inputs: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    PathInputChanged(String),
    CommitInputChanged(String),
    SubheadingInputChanged(usize, String),
    ParagraphInputChanged(usize, String),
    SearchButtonPressed,
    ExportButtonPressed,
}

impl Application for ConseilApp {
    type Executor = iced::executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (ConseilApp::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Conseil v0.1")
    }

    fn update(&mut self, message: Message) -> Command<Message> {

        match message {
            Message::PathInputChanged(value) => self.path_input = value,
            Message::CommitInputChanged(value) => self.commit_id_input = value,
            Message::SubheadingInputChanged(index, value) => self.subheading_inputs[index] = value,
            Message::ParagraphInputChanged(index, value) => self.paragraph_inputs[index] = value,
            Message::SearchButtonPressed => {
                self.repo = match Repository::open(self.path_input.as_str()) {
                    Ok(repo) => Some(repo),
                    Err(_) => None,
                };

                self.write_content();
            }
            Message::ExportButtonPressed => {
                self.make_markdown_file();
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {

        let title = text("Conseil").size(48).style(Color::from([0.0, 0.5, 1.0]));

        let path_input = text_input("Type repository path...", &self.path_input)
            .on_input(Message::PathInputChanged)
            .padding(10)
            .size(20);
                
        let commit_input = text_input("Type commit ID...", &self.commit_id_input)
            .on_input(Message::CommitInputChanged)
            .padding(10)
            .size(20);

        let search_button = button("Search")
            .padding(10)
            .on_press(Message::SearchButtonPressed);

        let export_button = button("Export")
            .padding(10)
            .on_press(Message::ExportButtonPressed);

        let scrollable = scrollable(
            self.scroll_content.iter().fold(
                column![].width(Length::Fill),
                |column, content| {
                    column.push(match content {
                        Content::Heading(line) => column![
                            text(line).size(60).style(Color::from([1.0, 0.5, 0.0])),
                            vertical_space(20.0),
                        ],
                        Content::Subheading(index, _) => {
                            column![
                                text_input("Subheading", &self.subheading_inputs[*index])
                                    .on_input(|s| Message::SubheadingInputChanged(*index, s))
                                    .size(32)
                                    .padding(10)
                            ]
                        }
                        Content::Filename(line) => column![text(format!("File: {}", line))],
                        Content::Paragraph(index, _) => {
                            column![
                                text_input("Subheading", &self.paragraph_inputs[*index])
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
            path_input,
            row![commit_input, search_button].spacing(10),
            horizontal_rule(5),
            scrollable,
        ]
        .spacing(20)
        .padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

impl ConseilApp {
    fn write_content(&mut self) {
        match self.repo {
            Some(_) => {
                
                self.scroll_content.clear();
                self.subheading_inputs.clear();
                self.paragraph_inputs.clear();

                let repo = self.repo.as_ref().unwrap();
                
                let oid = Oid::from_str(&self.commit_id_input.as_str());
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

                self.scroll_content.push(Content::Heading(commit.message().unwrap().to_string()));

                let parent_tree = parent.tree().expect("Could not find parent tree");
                let commit_tree = commit.tree().expect("Could not find parent tree");

                let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)
                               .expect("Could not find diff");

                let mut current_delta: String = String::new();
                let mut current_hunk: Vec<(String, Color)> = vec![];
                diff.print(git2::DiffFormat::Patch, |delta, _, line| {
                    if line.origin() == 'H' || line.origin() == 'F' {
                        true
                    } else {
                        if current_delta != delta.old_file().path().unwrap().to_str().unwrap().to_string() {
                            if !current_hunk.is_empty() {
                                self.scroll_content.push(Content::Subheading(self.subheading_inputs.len(), format!("Subheading {}", self.subheading_inputs.len() + 1)));
                                self.subheading_inputs.push(format!("Subheading {}", self.subheading_inputs.len() + 1));

                                self.scroll_content.push(Content::Paragraph(self.paragraph_inputs.len(), format!("Placeholder paragraph {}", self.paragraph_inputs.len() + 1)));
                                self.paragraph_inputs.push(format!("Placeholder paragraph {}", self.paragraph_inputs.len() + 1));

                                self.scroll_content.push(Content::Filename(current_delta.clone()));
                                self.scroll_content.push(Content::Hunk(current_hunk.clone()));
                                current_hunk.clear();
                            }
                            current_delta = delta.old_file().path().unwrap().to_str().unwrap().to_string();
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
            }
            None => {}
        }
    }

    fn make_markdown_file(&self) {
        let path = Path::new("entry.md");
        let display = path.display();

        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => file,
        };

        self.scroll_content.iter().map(|content| {
            match content {
                Content::Subheading(idx, _) => Content::Subheading(*idx, self.subheading_inputs[*idx].clone()),
                Content::Paragraph(idx, _) => Content::Paragraph(*idx, self.paragraph_inputs[*idx].clone()),
                _ => content.clone(),
            }
        }).for_each(|content| {
            match file.write_all(content.to_md_string().as_bytes()) {
                Err(why) => panic!("couldn't write to {}: {}", display, why),
                Ok(_) => println!("successfully wrote to {}", display),
            }
        })
    }
}
