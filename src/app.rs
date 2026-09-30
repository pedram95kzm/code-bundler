use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::text::{Alignment, Shaping};
use iced::widget::{
    Space, button, checkbox, column, container, row, scrollable, text, text_editor, text_input,
};
use iced::{Background, Border, Color, Element, Fill, Font, Padding, Task};

use crate::{embedder, workflow};

const BACKGROUND: Color = Color::from_rgb(0.047, 0.055, 0.071);
const CARD: Color = Color::from_rgb(0.086, 0.098, 0.122);
const FIELD: Color = Color::from_rgb(0.118, 0.133, 0.165);
const BORDER: Color = Color::from_rgb(0.212, 0.235, 0.275);
const MUTED: Color = Color::from_rgb(0.620, 0.651, 0.698);
const WHITE: Color = Color::WHITE;
const BLACK: Color = Color::BLACK;

const REGULAR: Font = Font::with_name("Vazirmatn");
const BOLD: Font = Font {
    weight: iced::font::Weight::Bold,
    ..Font::with_name("Vazirmatn")
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Embed,
    Extract,
}

#[derive(Debug, Clone)]
struct Notice {
    message: String,
    kind: NoticeKind,
    links: Vec<NoticeLink>,
}

#[derive(Debug, Clone)]
struct NoticeLink {
    label: String,
    path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct Completion {
    message: String,
    links: Vec<NoticeLink>,
}

#[derive(Debug, Clone, Copy)]
enum NoticeKind {
    Working,
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    SelectMode(Mode),
    EmbedPathChanged(String),
    RequestAction(text_editor::Action),
    CompressChanged(bool),
    BundlePathChanged(String),
    ModificationPathChanged(String),
    BrowseProject,
    BrowseBundle,
    BrowseModification,
    ProjectSelected(Option<PathBuf>),
    BundleSelected(Option<PathBuf>),
    ModificationSelected(Option<PathBuf>),
    Generate,
    Extract,
    OpenPath(PathBuf),
    Finished(Result<Completion, String>),
}

pub(crate) struct CodeBundlerApp {
    mode: Mode,
    embed_path: String,
    request: text_editor::Content,
    compress: bool,
    bundle_path: String,
    modification_path: String,
    notice: Option<Notice>,
    busy: bool,
}

impl Default for CodeBundlerApp {
    fn default() -> Self {
        Self {
            mode: Mode::Embed,
            embed_path: String::new(),
            request: text_editor::Content::new(),
            compress: false,
            bundle_path: String::new(),
            modification_path: String::new(),
            notice: None,
            busy: false,
        }
    }
}

pub(crate) fn update(state: &mut CodeBundlerApp, message: Message) -> Task<Message> {
    match message {
        Message::SelectMode(mode) if !state.busy => {
            state.mode = mode;
            state.notice = None;
        }
        Message::EmbedPathChanged(value) if !state.busy => state.embed_path = value,
        Message::RequestAction(action) if !state.busy => {
            state.request.perform(action);
        }
        Message::CompressChanged(value) if !state.busy => state.compress = value,
        Message::BundlePathChanged(value) if !state.busy => state.bundle_path = value,
        Message::ModificationPathChanged(value) if !state.busy => {
            state.modification_path = value;
        }
        Message::BrowseProject if !state.busy => {
            return Task::perform(
                async { rfd::FileDialog::new().pick_folder() },
                Message::ProjectSelected,
            );
        }
        Message::BrowseBundle if !state.busy => {
            return Task::perform(
                async {
                    rfd::FileDialog::new()
                        .add_filter("Text files", &["txt"])
                        .pick_file()
                },
                Message::BundleSelected,
            );
        }
        Message::BrowseModification if !state.busy => {
            return Task::perform(
                async {
                    rfd::FileDialog::new()
                        .add_filter("Text files", &["txt"])
                        .pick_file()
                },
                Message::ModificationSelected,
            );
        }
        Message::ProjectSelected(Some(path)) if !state.busy => {
            state.embed_path = path.display().to_string();
        }
        Message::BundleSelected(Some(path)) if !state.busy => {
            state.bundle_path = path.display().to_string();
        }
        Message::ModificationSelected(Some(path)) if !state.busy => {
            state.modification_path = path.display().to_string();
        }
        Message::Generate if !state.busy => return begin_generation(state),
        Message::Extract if !state.busy => return begin_extraction(state),
        Message::OpenPath(path) => {
            if let Err(error) = open_in_file_explorer(&path) {
                state.notice = Some(error_notice(error));
            }
        }
        Message::Finished(outcome) => {
            state.busy = false;
            state.notice = Some(match outcome {
                Ok(completion) => {
                    if state.mode == Mode::Embed {
                        state.request = text_editor::Content::new();
                    }
                    Notice {
                        message: completion.message,
                        kind: NoticeKind::Success,
                        links: completion.links,
                    }
                }
                Err(message) => error_notice(message),
            });
        }
        _ => {}
    }

    Task::none()
}

fn begin_generation(state: &mut CodeBundlerApp) -> Task<Message> {
    let Some(root) = entered_path(&state.embed_path) else {
        state.notice = Some(error_notice("Choose a project folder first."));
        return Task::none();
    };
    let request = state.request.text();
    let has_request = !request.trim().is_empty();
    let compress = state.compress;

    state.busy = true;
    state.notice = Some(Notice {
        message: if has_request {
            "Generating the bundle and prompt…".to_owned()
        } else {
            "Generating the bundle…".to_owned()
        },
        kind: NoticeKind::Working,
        links: Vec::new(),
    });

    Task::perform(
        async move {
            let report = workflow::generate_project(&root, &request, compress)?;
            let mut message = format!("Generated {} text file(s).", report.text_files);
            if report.binary_files > 0 {
                message.push_str(&format!(
                    "\n\nSkipped {} binary file(s).",
                    report.binary_files
                ));
            }
            append_warnings(&mut message, &report.warnings);
            let mut links = vec![NoticeLink {
                label: "Open bundle".to_owned(),
                path: report.bundle_path.clone(),
            }];
            if let Some(prompt_path) = report.prompt_path {
                links.push(NoticeLink {
                    label: "Open generated prompt".to_owned(),
                    path: prompt_path,
                });
            }
            if let Some(output_directory) = report.bundle_path.parent() {
                links.push(NoticeLink {
                    label: "Open output folder".to_owned(),
                    path: output_directory.to_path_buf(),
                });
            }
            Ok(Completion { message, links })
        },
        Message::Finished,
    )
}

fn begin_extraction(state: &mut CodeBundlerApp) -> Task<Message> {
    let Some(bundle_path) = entered_path(&state.bundle_path) else {
        state.notice = Some(error_notice("Choose a bundled text file first."));
        return Task::none();
    };
    let modification_path = entered_path(&state.modification_path);

    state.busy = true;
    state.notice = Some(Notice {
        message: "Extracting the project files…".to_owned(),
        kind: NoticeKind::Working,
        links: Vec::new(),
    });

    Task::perform(
        async move {
            if !bundle_path.is_file() {
                return Err(format!("'{}' is not a file", bundle_path.display()));
            }
            if let Some(path) = &modification_path
                && !path.is_file()
            {
                return Err(format!("'{}' is not a modification file", path.display()));
            }

            let report = embedder::embed_file(&bundle_path, modification_path.as_deref())?;
            let mut message = format!("Created {} file(s).", report.files_created);
            let changes_applied = report.modifications_applied
                + report.files_added
                + report.files_deleted
                + report.files_renamed;
            if changes_applied == 0 {
                message.push_str("\n\nNo modification file was applied.");
            } else {
                message.push_str("\n\nApplied changes:");
                if report.modifications_applied > 0 {
                    message.push_str(&format!(
                        "\n- Replaced {} line(s) across {} file(s).",
                        report.modifications_applied, report.files_modified
                    ));
                }
                if report.files_added > 0 {
                    message.push_str(&format!("\n- Added {} file(s).", report.files_added));
                }
                if report.files_deleted > 0 {
                    message.push_str(&format!("\n- Deleted {} file(s).", report.files_deleted));
                }
                if report.files_renamed > 0 {
                    message.push_str(&format!("\n- Renamed {} file(s).", report.files_renamed));
                }
            }
            if report.binary_files_skipped > 0 {
                message.push_str(&format!(
                    "\nSkipped {} binary file marker(s).",
                    report.binary_files_skipped
                ));
            }
            Ok(Completion {
                message,
                links: vec![NoticeLink {
                    label: "Open output folder".to_owned(),
                    path: report.output_root,
                }],
            })
        },
        Message::Finished,
    )
}

pub(crate) fn view(state: &CodeBundlerApp) -> Element<'_, Message> {
    let body = column![
        header(),
        Space::new().height(26),
        mode_switcher(state),
        Space::new().height(16),
        form_card(state),
        notice_view(state.notice.as_ref()),
        Space::new().height(20),
    ]
    .width(Fill)
    .align_x(Horizontal::Center);

    container(scrollable(
        container(body).width(Fill).max_width(680).center_x(Fill),
    ))
    .width(Fill)
    .height(Fill)
    .padding(Padding::new(24.0))
    .style(|_| container::Style {
        background: Some(Background::Color(BACKGROUND)),
        text_color: Some(WHITE),
        ..container::Style::default()
    })
    .into()
}

fn header<'a>() -> Element<'a, Message> {
    let logo_text = text("CB")
        .size(29)
        .font(BOLD)
        .align_x(Alignment::Center)
        .shaping(Shaping::Advanced);
    let logo = container(logo_text)
        .width(78)
        .height(78)
        .center_x(78)
        .center_y(78)
        .style(|_| container::Style {
            background: Some(Background::Color(BLACK)),
            border: Border {
                color: WHITE,
                width: 3.0,
                radius: 15.0.into(),
            },
            text_color: Some(WHITE),
            ..container::Style::default()
        });

    column![
        logo,
        Space::new().height(12),
        centered_text("Code Bundler").size(30).font(BOLD),
        Space::new().height(5),
        centered_muted(
            "Bundle a project for AI-assisted editing, or restore it safely.",
            14
        ),
    ]
    .width(Fill)
    .align_x(Horizontal::Center)
    .into()
}

fn mode_switcher(state: &CodeBundlerApp) -> Element<'_, Message> {
    row![
        mode_button("Embed", Mode::Embed, state),
        Space::new().width(10),
        mode_button("Extract", Mode::Extract, state),
    ]
    .width(Fill)
    .into()
}

fn mode_button<'a>(label: &'a str, mode: Mode, state: &CodeBundlerApp) -> Element<'a, Message> {
    let selected = state.mode == mode;
    let content = button_label(label, if selected { BLACK } else { WHITE });
    let widget = button(content)
        .width(Fill)
        .height(44)
        .style(move |_, status| button_style(selected, status));
    if state.busy {
        widget.into()
    } else {
        widget.on_press(Message::SelectMode(mode)).into()
    }
}

fn form_card(state: &CodeBundlerApp) -> Element<'_, Message> {
    let form = match state.mode {
        Mode::Embed => embed_form(state),
        Mode::Extract => extract_form(state),
    };

    container(form)
        .width(Fill)
        .padding(Padding::new(24.0))
        .style(|_| container::Style {
            background: Some(Background::Color(CARD)),
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 16.0.into(),
            },
            text_color: Some(WHITE),
            ..container::Style::default()
        })
        .into()
}

fn embed_form(state: &CodeBundlerApp) -> Element<'_, Message> {
    let mut project = centered_input("Choose the root folder of your project", &state.embed_path);
    let mut request = text_editor(&state.request)
        .placeholder("Describe the change you want the AI to make…")
        .height(120)
        .padding(Padding::from([14, 12]))
        .size(14)
        .font(REGULAR)
        .key_binding(|key_press| {
            let submits = matches!(
                key_press.key,
                iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter)
            ) && !key_press.modifiers.shift();

            if submits {
                Some(text_editor::Binding::Custom(Message::Generate))
            } else {
                text_editor::Binding::from_key_press(key_press)
            }
        });
    if !state.busy {
        project = project.on_input(Message::EmbedPathChanged);
        request = request.on_action(Message::RequestAction);
    }

    let browse = action_button("Browse", !state.busy, Message::BrowseProject, false).width(104);
    let generate = action_button("Generate", !state.busy, Message::Generate, true);
    let mut compress = checkbox(state.compress)
        .label("Compress source code safely")
        .size(18)
        .spacing(10)
        .text_size(14)
        .font(REGULAR)
        .style(|theme, status| checkbox::Style {
            text_color: Some(WHITE),
            ..checkbox::primary(theme, status)
        });
    if !state.busy {
        compress = compress.on_toggle(Message::CompressChanged);
    }

    column![
        form_heading(
            "Create an AI-ready bundle",
            "The bundle and generated prompt are saved in Documents/code_bundler.",
        ),
        Space::new().height(22),
        field_label("Project folder", None),
        row![project, Space::new().width(10), browse]
            .width(Fill)
            .align_y(Vertical::Center),
        Space::new().height(18),
        field_label("Request", Some("Optional")),
        request,
        centered_muted("Enter generates; Shift+Enter starts a new line.", 12),
        Space::new().height(16),
        container(compress).width(Fill).center_x(Fill),
        centered_muted(
            "Compression is off by default and never changes the original files.",
            12,
        ),
        Space::new().height(24),
        generate,
    ]
    .width(Fill)
    .align_x(Horizontal::Center)
    .into()
}

fn extract_form(state: &CodeBundlerApp) -> Element<'_, Message> {
    let mut bundle = centered_input(
        "Choose extracted_content_project_name.txt",
        &state.bundle_path,
    );
    let mut modification = centered_input("Choose modification_file.txt", &state.modification_path);
    if !state.busy {
        bundle = bundle.on_input(Message::BundlePathChanged);
        modification = modification.on_input(Message::ModificationPathChanged);
    }

    column![
        form_heading(
            "Restore a bundled project",
            "Restored files are saved in Documents/code_bundler.",
        ),
        Space::new().height(22),
        field_label("Bundled TXT file", None),
        row![
            bundle,
            Space::new().width(10),
            action_button("Browse", !state.busy, Message::BrowseBundle, false).width(104),
        ]
        .width(Fill)
        .align_y(Vertical::Center),
        Space::new().height(18),
        field_label("Modification file", Some("Optional")),
        row![
            modification,
            Space::new().width(10),
            action_button("Browse", !state.busy, Message::BrowseModification, false,).width(104),
        ]
        .width(Fill)
        .align_y(Vertical::Center),
        Space::new().height(24),
        action_button("Extract files", !state.busy, Message::Extract, true),
    ]
    .width(Fill)
    .align_x(Horizontal::Center)
    .into()
}

fn notice_view(notice: Option<&Notice>) -> Element<'_, Message> {
    let Some(notice) = notice else {
        return Space::new().height(0).into();
    };
    let (fill, stroke, title) = match notice.kind {
        NoticeKind::Working => (
            Color::from_rgb8(24, 35, 49),
            Color::from_rgb8(58, 105, 153),
            "Working",
        ),
        NoticeKind::Success => (
            Color::from_rgb8(20, 42, 34),
            Color::from_rgb8(51, 137, 97),
            "Completed",
        ),
        NoticeKind::Error => (
            Color::from_rgb8(51, 25, 29),
            Color::from_rgb8(168, 64, 76),
            "Could not complete the operation",
        ),
    };

    let mut panel = column![
        centered_text(title).font(BOLD),
        Space::new().height(6),
        centered_text(&notice.message).size(13),
    ]
    .width(Fill)
    .align_x(Horizontal::Center);
    for link in &notice.links {
        panel = panel.push(Space::new().height(10)).push(output_link(link));
    }

    column![
        Space::new().height(18),
        container(panel)
            .width(Fill)
            .padding(Padding::new(16.0))
            .style(move |_| container::Style {
                background: Some(Background::Color(fill)),
                border: Border {
                    color: stroke,
                    width: 1.0,
                    radius: 12.0.into(),
                },
                text_color: Some(WHITE),
                ..container::Style::default()
            }),
    ]
    .width(Fill)
    .into()
}

fn output_link(link: &NoticeLink) -> Element<'_, Message> {
    button(
        column![
            centered_text(&link.label).font(BOLD),
            centered_text(link.path.display().to_string())
                .size(12)
                .color(MUTED),
        ]
        .width(Fill)
        .align_x(Horizontal::Center),
    )
    .width(Fill)
    .padding(Padding::from([10, 12]))
    .on_press(Message::OpenPath(link.path.clone()))
    .style(|_, status| {
        let background = match status {
            button::Status::Hovered => FIELD,
            button::Status::Pressed => lighten(FIELD, 0.08),
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(Background::Color(background)),
            text_color: WHITE,
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..button::Style::default()
        }
    })
    .into()
}

fn form_heading<'a>(title: &'a str, description: &'a str) -> Element<'a, Message> {
    column![
        centered_text(title).size(20).font(BOLD),
        Space::new().height(2),
        centered_muted(description, 13),
    ]
    .width(Fill)
    .align_x(Horizontal::Center)
    .into()
}

fn field_label<'a>(label: &'a str, hint: Option<&'a str>) -> Element<'a, Message> {
    let label = match hint {
        Some(hint) => format!("{label} · {hint}"),
        None => label.to_owned(),
    };
    column![
        centered_text(label).size(13).font(BOLD),
        Space::new().height(2),
    ]
    .width(Fill)
    .align_x(Horizontal::Center)
    .into()
}

fn centered_input<'a>(
    placeholder: &'a str,
    value: &'a str,
) -> iced::widget::TextInput<'a, Message> {
    text_input(placeholder, value)
        .width(Fill)
        .padding(Padding::from([14, 12]))
        .size(14)
        .font(REGULAR)
        .align_x(Horizontal::Center)
        .style(|_, status| text_input_style(status))
}

fn action_button<'a>(
    label: &'a str,
    enabled: bool,
    message: Message,
    primary: bool,
) -> iced::widget::Button<'a, Message> {
    let label_color = if primary { BLACK } else { WHITE };
    let widget = button(button_label(label, label_color))
        .width(Fill)
        .height(46)
        .style(move |_, status| action_button_style(primary, status));
    if enabled {
        widget.on_press(message)
    } else {
        widget
    }
}

/// Wraps a button label so it is centered both horizontally and vertically
/// inside a fixed-height button.
fn button_label<'a>(label: &'a str, color: Color) -> Element<'a, Message> {
    container(centered_text(label).font(BOLD).color(color))
        .width(Fill)
        .height(Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
}

fn centered_text<'a>(content: impl text::IntoFragment<'a>) -> iced::widget::Text<'a> {
    text(content)
        .width(Fill)
        .font(REGULAR)
        .align_x(Alignment::Center)
        .shaping(Shaping::Advanced)
}

fn centered_muted<'a>(content: &'a str, size: u32) -> iced::widget::Text<'a> {
    centered_text(content).size(size).color(MUTED)
}

fn button_style(selected: bool, status: button::Status) -> button::Style {
    let background = if selected { WHITE } else { FIELD };
    let background = match status {
        button::Status::Hovered => lighten(background, 0.08),
        button::Status::Pressed => lighten(background, 0.14),
        button::Status::Disabled => background.scale_alpha(0.55),
        button::Status::Active => background,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: if selected { BLACK } else { WHITE },
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 10.0.into(),
        },
        ..button::Style::default()
    }
}

fn action_button_style(primary: bool, status: button::Status) -> button::Style {
    let background = if primary { WHITE } else { FIELD };
    let background = match status {
        button::Status::Hovered => lighten(background, if primary { -0.08 } else { 0.08 }),
        button::Status::Pressed => lighten(background, if primary { -0.15 } else { 0.14 }),
        button::Status::Disabled => background.scale_alpha(0.45),
        button::Status::Active => background,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: if primary { BLACK } else { WHITE },
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 10.0.into(),
        },
        ..button::Style::default()
    }
}

fn text_input_style(status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => Color::from_rgb8(104, 117, 139),
        text_input::Status::Hovered => Color::from_rgb8(86, 94, 108),
        _ => BORDER,
    };
    text_input::Style {
        background: Background::Color(FIELD),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 8.0.into(),
        },
        icon: MUTED,
        placeholder: MUTED,
        value: WHITE,
        selection: Color::from_rgb8(72, 82, 98),
    }
}

fn lighten(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r + amount).clamp(0.0, 1.0),
        g: (color.g + amount).clamp(0.0, 1.0),
        b: (color.b + amount).clamp(0.0, 1.0),
        a: color.a,
    }
}

fn entered_path(value: &str) -> Option<PathBuf> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let value = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value);
    Some(Path::new(value).to_path_buf())
}

fn open_in_file_explorer(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("'{}' no longer exists", path.display()));
    }

    #[cfg(target_os = "windows")]
    let result = {
        let mut command = ProcessCommand::new("explorer.exe");
        if path.is_dir() {
            command.arg(path);
        } else {
            command.arg(format!("/select,{}", path.display()));
        }
        command.spawn()
    };

    #[cfg(target_os = "macos")]
    let result = {
        let mut command = ProcessCommand::new("open");
        if path.is_file() {
            command.arg("-R");
        }
        command.arg(path).spawn()
    };

    #[cfg(all(unix, not(target_os = "macos")))]
    let result = {
        let target = if path.is_dir() {
            path
        } else {
            path.parent().unwrap_or(path)
        };
        ProcessCommand::new("xdg-open").arg(target).spawn()
    };

    #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
    let result: std::io::Result<std::process::Child> =
        Err(std::io::Error::other("file explorer is not supported"));

    result.map(|_| ()).map_err(|error| {
        format!(
            "cannot open '{}' in the file explorer: {error}",
            path.display()
        )
    })
}

fn error_notice(message: impl Into<String>) -> Notice {
    Notice {
        message: message.into(),
        kind: NoticeKind::Error,
        links: Vec::new(),
    }
}

fn append_warnings(message: &mut String, warnings: &[String]) {
    if warnings.is_empty() {
        return;
    }
    message.push_str(&format!("\n\nWarnings ({}):", warnings.len()));
    for warning in warnings {
        message.push_str("\n• ");
        message.push_str(warning);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entered_paths_may_be_quoted() {
        assert_eq!(
            entered_path("  \"C:\\some folder\"  "),
            Some(PathBuf::from("C:\\some folder"))
        );
        assert_eq!(entered_path("   "), None);
    }

    #[test]
    fn request_editor_initializes_empty() {
        let state = CodeBundlerApp::default();

        assert!(state.request.text().is_empty());
    }

    #[test]
    fn successful_generation_clears_the_request() {
        let mut state = CodeBundlerApp {
            request: text_editor::Content::with_text("Change the title"),
            busy: true,
            ..CodeBundlerApp::default()
        };

        let _ = update(
            &mut state,
            Message::Finished(Ok(Completion {
                message: "Generated".to_owned(),
                links: Vec::new(),
            })),
        );

        assert!(state.request.text().is_empty());
    }

    #[test]
    fn failed_generation_keeps_the_request() {
        let mut state = CodeBundlerApp {
            request: text_editor::Content::with_text("Change the title"),
            busy: true,
            ..CodeBundlerApp::default()
        };

        let _ = update(&mut state, Message::Finished(Err("Failed".to_owned())));

        assert_eq!(state.request.text(), "Change the title");
    }
}
