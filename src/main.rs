use std::io;
use strum::VariantArray;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};

#[derive(Debug, Clone, Copy, PartialEq, VariantArray)]
enum ProjectType {
    New,
    Existing,
}

impl ProjectType {
    fn label(&self) -> &str {
        match self {
            ProjectType::New => "New Project",
            ProjectType::Existing => "Existing Project",
        }
    }
}

#[derive(Debug, Default)]
struct ProjectConfig {
    project_type: Option<ProjectType>,
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

#[derive(Debug, Default, VariantArray)]
enum WizardStep {
    #[default]
    ProjectType,
    Vcs,
    Languages,
    Database,
    Remotes,
    Extras,
    Summary,
}

impl WizardStep {
    fn title(&self) -> &str {
        match self {
            WizardStep::ProjectType => "Project Type",
            WizardStep::Vcs => "Version Control System",
            WizardStep::Languages => "Languages",
            WizardStep::Database => "Database",
            WizardStep::Remotes => "Remotes",
            WizardStep::Extras => "Extras",
            WizardStep::Summary => "Summary",
        }
    }
}

#[derive(Debug, Default)]
struct App {
    step_index: usize,
    cursor: usize,
    config: ProjectConfig,
    exit: bool,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let [wizard_area, config_area] =
            Layout::horizontal([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)])
                .areas(frame.area());

        // Wizard panel (left 2/3)
        let title = Line::from(format!(" cinderbox — {} ", self.current_step().title())).bold();
        let instructions = Line::from(vec![
            " Back ".into(),
            "<Left/H> ".blue().bold(),
            " Next ".into(),
            "<Right/L> ".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let wizard_block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered());

        let content = self.step_content();
        let wizard = Paragraph::new(content).block(wizard_block);
        frame.render_widget(wizard, wizard_area);

        // Config panel (right 1/3)
        let config_block = Block::bordered().title(Line::from(" Config ").bold().centered());

        let config_text = self.config_summary();
        let config = Paragraph::new(config_text).block(config_block);
        frame.render_widget(config, config_area);
    }

    fn step_content(&self) -> String {
        match self.current_step() {
            WizardStep::ProjectType => {
                let mut lines = Vec::new();
                for (i, variant) in ProjectType::VARIANTS.iter().enumerate() {
                    let marker = if i == self.cursor { "▸ " } else { "  " };
                    lines.push(format!("{}{}", marker, variant.label()));
                }
                lines.join("\n")
            }
            _ => format!("Step: {}", self.current_step().title()),
        }
    }

    fn config_summary(&self) -> String {
        let mut lines = Vec::new();

        match &self.config.project_type {
            Some(pt) => lines.push(format!("Type: {}", pt.label())),
            None => lines.push("Type: —".to_string()),
        }

        if lines.iter().all(|l| l.ends_with('—')) {
            return "No selections yet.".to_string();
        }

        lines.join("\n")
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => self.exit = true,
                KeyCode::Right | KeyCode::Char('l') => self.next(),
                KeyCode::Left | KeyCode::Char('h') => self.prev(),
                KeyCode::Down | KeyCode::Char('j') => self.cursor_down(),
                KeyCode::Up | KeyCode::Char('k') => self.cursor_up(),
                KeyCode::Enter => self.select(),
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn current_step(&self) -> &WizardStep {
        &WizardStep::VARIANTS[self.step_index]
    }

    fn cursor_max(&self) -> usize {
        match self.current_step() {
            WizardStep::ProjectType => ProjectType::VARIANTS.len().saturating_sub(1),
            _ => 0,
        }
    }

    fn cursor_down(&mut self) {
        if self.cursor < self.cursor_max() {
            self.cursor += 1;
        }
    }

    fn cursor_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn select(&mut self) {
        match self.current_step() {
            WizardStep::ProjectType => {
                self.config.project_type = Some(ProjectType::VARIANTS[self.cursor]);
                self.next();
            }
            _ => {}
        }
    }

    fn restore_cursor(&mut self) {
        self.cursor = match self.current_step() {
            WizardStep::ProjectType => self
                .config
                .project_type
                .and_then(|pt| ProjectType::VARIANTS.iter().position(|v| *v == pt))
                .unwrap_or(0),
            _ => 0,
        };
    }

    fn next(&mut self) {
        if self.step_index + 1 < WizardStep::VARIANTS.len() {
            self.step_index += 1;
            self.restore_cursor();
        }
    }

    fn prev(&mut self) {
        if self.step_index > 0 {
            self.step_index -= 1;
            self.restore_cursor();
        }
    }
}
