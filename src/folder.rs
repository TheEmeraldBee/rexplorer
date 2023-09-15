use std::{ffi::OsString, fs, path::PathBuf};

use ratatui::{
    style::Style,
    widgets::{List, ListItem},
};

use crate::chunks::Chunks;

const BORDER_SIZE: u16 = 3;

#[derive(Debug, Clone)]
pub struct Folder {
    selected: usize,
    path: PathBuf,
    path_name: String,
    entries: Vec<PathBuf>,
    scroll: usize,
    rows: usize,
}

impl From<PathBuf> for Folder {
    fn from(mut value: PathBuf) -> Self {
        value = value.canonicalize().expect("Path Should Exist");
        let dir = std::fs::read_dir(value.clone()).expect("PathBuf should be valid");

        let mut paths = vec![PathBuf::from("..")];
        for path in dir.into_iter().filter(|x| x.is_ok()).flatten() {
            paths.push(path.path())
        }

        let mut path_name = "N/A";
        let value_clone = value.clone();
        if let Some(file_name) = value_clone.file_name() {
            path_name = file_name.to_str().unwrap_or("N/A").clone();
        }

        Self {
            entries: paths,
            path: value,
            path_name: String::from(path_name),
            selected: 0,
            scroll: 0,
            rows: 0,
        }
    }
}

impl From<&str> for Folder {
    fn from(value: &str) -> Self {
        let path = PathBuf::from(value);

        Self::from(path)
    }
}

impl Folder {
    pub fn as_list(&mut self, chunks: &Chunks) -> List {
        let mut elements = vec![];

        let rows = chunks.main().height - BORDER_SIZE;
        self.rows = rows.into();

        for i in self.scroll..self.entries.len() {
            let entry = &self.entries[i];
            if let Some(name) = entry
                .file_name()
                .unwrap_or(&OsString::from(
                    entry.to_str().expect("Should be valid UTF-8"),
                ))
                .to_str()
            {
                let mut style = Style::new();
                if i == self.selected {
                    style = style.bg(ratatui::style::Color::DarkGray);
                }
                if entry.is_dir() {
                    elements.push(
                        ListItem::new(name.to_string())
                            .style(style.fg(ratatui::style::Color::Cyan)),
                    );
                } else {
                    elements.push(
                        ListItem::new(name.to_string())
                            .style(style.fg(ratatui::style::Color::LightGreen)),
                    );
                }
            }
        }

        List::new(elements)
    }

    pub fn scroll_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if (self.selected as i64 - self.scroll as i64) < 0 {
                self.scroll -= 1;
            }
        }
    }

    pub fn scroll_down(&mut self) {
        if self.selected < self.entries.len() - 1 {
            self.selected += 1;
            if self.selected - self.scroll > self.rows {
                self.scroll += 1;
            }
        }
    }

    pub fn make_folder(&mut self, name: String) -> anyhow::Result<()> {
        fs::create_dir(self.path.join(name))?;
        Ok(())
    }

    pub fn touch_file(&mut self, file: String) -> anyhow::Result<()> {
        // Check if file exists
        if fs::read(self.path.join(file.clone())).is_ok() {
            return Ok(());
        }

        fs::write(self.path.join(file), "")?;

        Ok(())
    }

    pub fn delete(&mut self) -> anyhow::Result<()> {
        if self.entries[self.selected].is_dir() {
            fs::remove_dir(self.entries[self.selected].clone())?;
        } else {
            fs::remove_file(self.entries[self.selected].clone())?;
        }
        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.path_name
    }

    pub fn file_name(&self) -> String {
        self.entries[self.selected]
            .file_name()
            .unwrap_or(&OsString::from("N/A"))
            .to_str()
            .expect("Should be valid")
            .to_string()
    }

    pub fn enter(&mut self) -> Option<Self> {
        if self.selected == 0 {
            return self.exit();
        }

        let path = self
            .path
            .join(self.entries[self.selected].file_name().unwrap().clone());

        if path.is_dir() {
            return Some(Self::from(path));
        }

        None
    }

    pub fn exit(&mut self) -> Option<Self> {
        let mut path = self.path.clone();

        if path.pop() {
            log::info!("Pop Successfull, path {:?}", path);
            return Some(Self::from(path));
        }
        None
    }

    pub fn reload(&self) -> Self {
        Self::from(self.path.clone())
    }
}
