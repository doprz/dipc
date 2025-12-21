use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    widgets::ListState,
};
use std::{collections::HashSet, path::PathBuf};

use crate::{
    cli::{ColorPalette, ColorPaletteStyles},
    tui::utils::{is_image_file, parse_color_value},
};

pub const PALETTES: &[(&str, ColorPalette)] = &[
    ("Catppuccin", ColorPalette::Catppuccin),
    ("Dracula", ColorPalette::Dracula),
    ("Edge", ColorPalette::Edge),
    ("Everforest", ColorPalette::Everforest),
    ("Gruvbox", ColorPalette::Gruvbox),
    ("Gruvbox Material", ColorPalette::GruvboxMaterial),
    ("Nord", ColorPalette::Nord),
    ("One Dark", ColorPalette::OneDark),
    ("Rose Pine", ColorPalette::RosePine),
    ("Solarized", ColorPalette::Solarized),
    ("Tokyo Night", ColorPalette::TokyoNight),
];

pub const HELP_TEXT: &str =
    " ↑↓/jk navigate │ Space select │ Tab panel │ Enter run │ / path │ a all │ ? help │ q quit ";

pub const HELP_POPUP: &str = r#"
  Keybindings
  ───────────────────────────────────────

  Navigation
    ↑/k         Move up
    ↓/j         Move down
    Tab         Next panel
    Shift+Tab   Previous panel

  Files Panel
    Space       Toggle file selection
    Enter       Enter directory / Run
    Backspace   Parent directory
    a           Select all images
    d           Deselect all
    /           Enter path manually

  Palette Panel
    Space       Toggle variation
    a           Select all variations
    d           Deselect all variations

  General
    Enter       Process selected files
    ?           Toggle this help
    q/Esc       Quit

  ───────────────────────────────────────
  Press any key to close
"#;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    #[default]
    Files,
    Palette,
}

impl Panel {
    fn next(self) -> Self {
        match self {
            Self::Files => Self::Palette,
            Self::Palette => Self::Files,
        }
    }
}

#[derive(Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_image: bool,
}

pub enum PaletteEntry {
    Palette { name: &'static str, idx: usize },
    Variation { name: String, idx: usize },
}

pub struct TuiConfig {
    pub palette: ColorPalette,
    pub styles: ColorPaletteStyles,
    pub files: Vec<PathBuf>,
}

pub struct App {
    // UI
    pub panel: Panel,

    // UI state
    pub show_help: bool,
    pub input_mode: bool,
    pub input_buf: String,

    // Files
    pub current_dir: PathBuf,
    pub file_entries: Vec<FileEntry>,
    pub file_state: ListState,
    pub selected_files: HashSet<PathBuf>,

    // Palette
    pub palette_entries: Vec<PaletteEntry>,
    pub palette_state: ListState,
    pub selected_palette_idx: usize,
    pub selected_variations: HashSet<String>,
    pub preview_colors: Vec<(String, [u8; 3])>,

    // App control
    pub should_run: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_default();
        let mut app = Self {
            panel: Panel::default(),

            show_help: false,
            input_mode: false,
            input_buf: String::new(),

            current_dir,
            file_entries: Vec::new(),
            file_state: ListState::default(),
            selected_files: HashSet::new(),

            palette_entries: Vec::new(),
            palette_state: ListState::default(),
            selected_palette_idx: 0,
            selected_variations: HashSet::new(),
            preview_colors: Vec::new(),

            should_run: false,
            should_quit: false,
        };

        app.refresh_files();
        app.refresh_palettes();
        app.update_preview();

        app
    }
}

impl App {
    fn refresh_files(&mut self) {
        self.file_entries.clear();

        // Parent directory
        if self.current_dir.parent().is_some() {
            self.file_entries.push(FileEntry {
                name: "..".into(),
                path: self.current_dir.parent().unwrap().to_path_buf(),
                is_dir: true,
                is_image: false,
            });
        }

        if let Ok(read_dir) = std::fs::read_dir(&self.current_dir) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();

            for entry in read_dir.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden files
                if name.starts_with('.') {
                    continue;
                }

                let is_dir = path.is_dir();
                let is_image = is_image_file(&path);

                if is_dir {
                    dirs.push(FileEntry {
                        name,
                        path,
                        is_dir,
                        is_image: false,
                    });
                } else if is_image {
                    files.push(FileEntry {
                        name,
                        path,
                        is_dir: false,
                        is_image,
                    });
                }
            }

            dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            self.file_entries.extend(dirs);
            self.file_entries.extend(files);
        }

        self.file_state.select(if self.file_entries.is_empty() {
            None
        } else {
            Some(0)
        });
    }

    fn refresh_palettes(&mut self) {
        self.palette_entries.clear();

        for (idx, (name, palette)) in PALETTES.iter().enumerate() {
            self.palette_entries
                .push(PaletteEntry::Palette { name, idx });

            if idx == self.selected_palette_idx {
                let json = palette.clone().get_json();
                let mut variations: Vec<_> = json.keys().cloned().collect();
                variations.sort();

                for var_name in variations {
                    self.palette_entries.push(PaletteEntry::Variation {
                        name: var_name,
                        idx,
                    });
                }
            }
        }

        // Select first item if nothing selected
        if self.palette_state.selected().is_none() {
            self.palette_state.select(Some(0));
        }
    }

    fn update_preview(&mut self) {
        self.preview_colors.clear();
        let palette = &PALETTES[self.selected_palette_idx].1;
        let json = palette.clone().get_json();

        // Collect unique colors (by RGB value) to avoid duplicates
        let mut seen: HashSet<[u8; 3]> = HashSet::new();

        let variations_to_show: Vec<&String> = if self.selected_variations.is_empty() {
            // No variations selected: show all variations
            json.keys().collect()
        } else {
            // Show only selected variations
            self.selected_variations.iter().collect()
        };

        for var_name in variations_to_show {
            if let Some(serde_json::Value::Object(map)) = json.get(var_name) {
                for (name, val) in map {
                    if let Some(color) = parse_color_value(val) {
                        if seen.insert(color) {
                            self.preview_colors.push((name.clone(), color));
                        }
                    }
                }
            }
        }

        // Sort by color name for consistent display
        self.preview_colors
            .sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    }

    fn selected_palette(&self) -> &ColorPalette {
        &PALETTES[self.selected_palette_idx].1
    }

    fn selected_styles(&self) -> ColorPaletteStyles {
        if self.selected_variations.is_empty() {
            ColorPaletteStyles::All
        } else {
            ColorPaletteStyles::Some {
                styles: self.selected_variations.iter().cloned().collect(),
            }
        }
    }

    pub fn output_preview(&self) -> Vec<String> {
        if self.selected_files.is_empty() {
            return vec!["No files selected".into()];
        }

        let palette_name = PALETTES[self.selected_palette_idx]
            .0
            .to_lowercase()
            .replace(' ', "-");
        let var_suffix: String = if self.selected_variations.is_empty() {
            String::new()
        } else {
            let mut vars: Vec<_> = self.selected_variations.iter().cloned().collect();
            vars.sort();
            format!("-{}", vars.join("-"))
        };

        self.selected_files
            .iter()
            .take(5)
            .map(|p| {
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
                format!("{}_{}{}.png", stem, palette_name, var_suffix)
            })
            .collect()
    }

    pub fn config(&self) -> Option<TuiConfig> {
        if self.selected_files.is_empty() {
            return None;
        }
        Some(TuiConfig {
            palette: self.selected_palette().clone(),
            styles: self.selected_styles(),
            files: self.selected_files.iter().cloned().collect(),
        })
    }

    pub fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        // Help popup
        if self.show_help {
            self.show_help = false;
            return;
        }

        // Input mode
        if self.input_mode {
            match code {
                KeyCode::Esc => {
                    self.input_mode = false;
                    self.input_buf.clear();
                }
                KeyCode::Enter => {
                    let path = PathBuf::from(&self.input_buf);
                    if path.is_dir() {
                        self.current_dir = path;
                        self.refresh_files();
                    } else if path.is_file() && is_image_file(&path) {
                        self.selected_files.insert(path);
                    }
                    self.input_mode = false;
                    self.input_buf.clear();
                }
                KeyCode::Char(c) => self.input_buf.push(c),
                KeyCode::Backspace => {
                    self.input_buf.pop();
                }
                _ => {}
            }
            return;
        }

        // Global keys
        match code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                return;
            }
            KeyCode::Char('?') => {
                self.show_help = true;
                return;
            }
            KeyCode::Tab if modifiers.contains(KeyModifiers::SHIFT) => {
                self.panel = self.panel.next();
                return;
            }
            KeyCode::Tab => {
                self.panel = self.panel.next();
                return;
            }
            KeyCode::Enter => {
                // In files panel, enter directory or run
                if self.panel == Panel::Files {
                    if let Some(idx) = self.file_state.selected() {
                        if let Some(entry) = self.file_entries.get(idx) {
                            if entry.is_dir {
                                self.current_dir = entry.path.clone();
                                self.refresh_files();
                                return;
                            }
                        }
                    }
                }
                // Otherwise, run if we have files
                if !self.selected_files.is_empty() {
                    self.should_run = true;
                }
                return;
            }
            _ => {}
        }

        // Panel-specific
        match self.panel {
            Panel::Files => self.handle_files_key(code),
            Panel::Palette => self.handle_palette_key(code),
        }
    }

    fn handle_files_key(&mut self, code: KeyCode) {
        let len = self.file_entries.len();
        if len == 0 && !matches!(code, KeyCode::Char('/') | KeyCode::Backspace) {
            return;
        }

        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.file_state.selected().unwrap_or(0);
                self.file_state.select(Some(i.saturating_sub(1)));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.file_state.selected().unwrap_or(0);
                self.file_state
                    .select(Some((i + 1).min(len.saturating_sub(1))));
            }
            KeyCode::Char(' ') => {
                if let Some(idx) = self.file_state.selected() {
                    if let Some(entry) = self.file_entries.get(idx) {
                        if entry.is_image {
                            if self.selected_files.contains(&entry.path) {
                                self.selected_files.remove(&entry.path);
                            } else {
                                self.selected_files.insert(entry.path.clone());
                            }
                        }
                    }
                }
            }
            KeyCode::Char('a') => {
                for entry in &self.file_entries {
                    if entry.is_image {
                        self.selected_files.insert(entry.path.clone());
                    }
                }
            }
            KeyCode::Char('d') => {
                self.selected_files.clear();
            }
            KeyCode::Char('/') => {
                self.input_mode = true;
                self.input_buf = self.current_dir.to_string_lossy().to_string();
            }
            KeyCode::Backspace => {
                if let Some(parent) = self.current_dir.parent() {
                    self.current_dir = parent.to_path_buf();
                    self.refresh_files();
                }
            }
            _ => {}
        }
    }

    fn handle_palette_key(&mut self, code: KeyCode) {
        let len = self.palette_entries.len();
        if len == 0 {
            return;
        }

        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.palette_state.selected().unwrap_or(0);
                self.palette_state.select(Some(i.saturating_sub(1)));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.palette_state.selected().unwrap_or(0);
                self.palette_state.select(Some((i + 1).min(len - 1)));
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                if let Some(idx) = self.palette_state.selected() {
                    match &self.palette_entries[idx] {
                        PaletteEntry::Palette { idx: pidx, .. } => {
                            if self.selected_palette_idx != *pidx {
                                self.selected_palette_idx = *pidx;
                                self.selected_variations.clear();
                                self.refresh_palettes();
                                self.update_preview();
                            }
                        }
                        PaletteEntry::Variation { name, .. } => {
                            if self.selected_variations.contains(name) {
                                self.selected_variations.remove(name);
                            } else {
                                self.selected_variations.insert(name.clone());
                            }
                            self.update_preview();
                        }
                    }
                }
            }
            KeyCode::Char('a') => {
                // Select all variations of current palette
                for entry in &self.palette_entries {
                    if let PaletteEntry::Variation { name, idx } = entry {
                        if *idx == self.selected_palette_idx {
                            self.selected_variations.insert(name.clone());
                        }
                    }
                }
                self.update_preview();
            }
            KeyCode::Char('d') => {
                self.selected_variations.clear();
                self.update_preview();
            }
            _ => {}
        }
    }
}
