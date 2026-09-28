use std::path::{Path, PathBuf};

use iced::alignment::{Horizontal, Vertical};
use iced::widget::text::{Alignment, Shaping};
use iced::widget::{Space, button, checkbox, column, container, row, scrollable, text, text_input};
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
    RequestChanged(String),
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
    Finished(Result<String, String>),
}

pub(crate) struct CodeBundlerApp {
    mode: Mode,
    embed_path: String,
    request: String,
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
            request: String::new(),
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
        Message::RequestChanged(value) if !state.busy => state.request = value,
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
        Message::Finished(outcome) => {
            state.busy = false;
            state.notice = Some(match outcome {
                Ok(message) => Notice {
                    message,
                    kind: NoticeKind::Success,
                },
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
    let request = state.request.clone();
    let compress = state.compress;

    state.busy = true;
    state.notice = Some(Notice {
        message: "Generating the bundle and prompt…".to_owned(),
        kind: NoticeKind::Working,
    });

    Task::perform(
        async move {
            let report = workflow::generate_project(&root, &request, compress)?;
            let mut message = format!(
                "Generated {} text file(s).\n\nBundle:\n{}\n\nPrompt:\n{}",
                report.text_files,
                report.bundle_path.display(),
                report.prompt_path.display()
            );
            if report.binary_files > 0 {
                message.push_str(&format!(
                    "\n\nSkipped {} binary file(s).",
                    report.binary_files
                ));
            }
            append_warnings(&mut message, &report.warnings);
            Ok(message)
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
            let mut message = format!(
                "Created {} file(s).\n\nOutput folder:\n{}",
                report.files_created,
                report.output_root.display()
            );
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
            Ok(message)
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
    let mut request = centered_input(
        "Describe the change you want the AI to make…",
        &state.request,
    );
    if !state.busy {
        project = project.on_input(Message::EmbedPathChanged);
        request = request.on_input(Message::RequestChanged);
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

    let panel = column![
        centered_text(title).font(BOLD),
        Space::new().height(6),
        centered_text(&notice.message).size(13),
    ]
    .width(Fill)
    .align_x(Horizontal::Center);

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

fn error_notice(message: impl Into<String>) -> Notice {
    Notice {
        message: message.into(),
        kind: NoticeKind::Error,
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
    fn request_input_accepts_persian_text() {
        let mut state = CodeBundlerApp::default();
        let request = "لطفاً این پروژه را بررسی و اصلاح کن";

        let _ = update(&mut state, Message::RequestChanged(request.to_owned()));

        assert_eq!(state.request, request);
    }
}
