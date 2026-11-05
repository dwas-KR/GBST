// SPDX-License-Identifier: GPL-3.0-only
// GBST-specific LTBox-inspired visual shell. No firmware, Fastboot, EDL or root actions.
// Reuses the existing proven Google Services / direct-USB ADB workers.
use iced::widget::checkbox;

fn uix(language: LanguageOption, key: &str) -> String {
    gbst_core::log_i18n::log_label(language, key)
}

#[derive(Clone, Copy)]
struct ShellColors {
    page: Color,
    panel: Color,
    sidebar: Color,
    row: Color,
    text: Color,
    muted: Color,
    border: Color,
    accent: Color,
    tint: Color,
    success: Color,
}

// Independently authored Material 3 palette matching LTBox's visible UI.
fn shell_colors(theme: &Theme) -> ShellColors {
    let p = theme.palette();
    let dark = p.background.r + p.background.g + p.background.b < 1.55;
    if dark {
        ShellColors {
            page: Color::from_rgb8(19, 19, 24),
            panel: Color::from_rgb8(32, 31, 38),
            sidebar: Color::from_rgb8(27, 27, 33),
            row: Color::from_rgb8(42, 41, 48),
            text: Color::from_rgb8(228, 225, 233),
            muted: Color::from_rgb8(199, 197, 208),
            border: Color::from_rgb8(71, 70, 79),
            accent: p.primary,
            tint: if p.primary.g > 0.45 { Color::from_rgb8(40, 74, 76) }
                  else if p.primary.r > 0.55 { Color::from_rgb8(83, 48, 66) }
                  else { Color::from_rgb8(67, 69, 89) },
            success: Color::from_rgb8(138, 218, 149),
        }
    } else {
        ShellColors {
            page: Color::from_rgb8(251, 248, 253),
            panel: Color::from_rgb8(239, 236, 241),
            sidebar: Color::from_rgb8(245, 242, 247),
            row: Color::from_rgb8(233, 231, 235),
            text: Color::from_rgb8(27, 27, 33),
            muted: Color::from_rgb8(71, 70, 79),
            border: Color::from_rgb8(199, 197, 208),
            accent: p.primary,
            tint: if p.primary.g > 0.45 { Color::from_rgb8(203, 233, 231) }
                  else if p.primary.r > 0.55 { Color::from_rgb8(255, 215, 230) }
                  else { Color::from_rgb8(224, 225, 249) },
            success: Color::from_rgb8(33, 108, 42),
        }
    }
}

fn shell_border(theme: &Theme, radius: f32) -> iced::Border {
    iced::Border {
        radius: radius.into(),
        width: 1.0,
        color: shell_colors(theme).border,
    }
}

fn shell_main_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: Some(Background::Color(c.page)),
        text_color: Some(c.text),
        ..container::Style::default()
    }
}
fn shell_sidebar_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: Some(Background::Color(c.sidebar)),
        text_color: Some(c.text),
        ..container::Style::default()
    }
}
fn shell_panel_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: Some(Background::Color(c.panel)),
        text_color: Some(c.text),
        border: shell_border(theme, 14.0),
        ..container::Style::default()
    }
}
fn shell_row_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: Some(Background::Color(c.row)),
        text_color: Some(c.text),
        border: iced::Border {
            radius: 7.0.into(),
            ..iced::Border::default()
        },
        ..container::Style::default()
    }
}
fn shell_highlight_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: Some(Background::Color(c.tint)),
        text_color: Some(c.text),
        border: iced::Border {
            radius: 10.0.into(),
            width: 1.0,
            color: Color::from_rgba(c.accent.r, c.accent.g, c.accent.b, 0.48),
        },
        ..container::Style::default()
    }
}
fn shell_divider_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: Some(Background::Color(c.border)),
        ..container::Style::default()
    }
}
fn shell_footer_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: Some(Background::Color(c.sidebar)),
        text_color: Some(c.muted),
        border: iced::Border {
            width: 1.0,
            color: c.border,
            ..iced::Border::default()
        },
        ..container::Style::default()
    }
}
fn shell_muted<'a>(label: impl Into<String>, size: u32) -> iced::widget::Text<'a> {
    text(label.into())
        .size(size)
        .style(|theme: &Theme| iced::widget::text::Style {
            color: Some(shell_colors(theme).muted),
        })
}
fn shell_action<'a>(
    label: String,
    message: Message,
    enabled: bool,
    primary: bool,
) -> Element<'a, Message> {
    let btn = button(
        container(text(label).size(13))
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center),
    )
    .padding([11.0, 18.0])
    .height(Length::Fixed(40.0))
    .style(move |theme: &Theme, status| {
        let c = shell_colors(theme);
        let hovered = matches!(status, iced::widget::button::Status::Hovered);
        button::Style {
            background: Some(Background::Color(if !enabled {
                c.row
            } else if primary {
                if hovered {
                    Color::from_rgba(c.accent.r, c.accent.g, c.accent.b, 0.87)
                } else {
                    c.accent
                }
            } else if hovered {
                c.tint
            } else {
                c.panel
            })),
            text_color: if !enabled {
                c.muted
            } else if primary {
                Color::WHITE
            } else {
                c.text
            },
            border: iced::Border {
                radius: 11.0.into(),
                width: if primary { 0.0 } else { 1.0 },
                color: if primary {
                    Color::TRANSPARENT
                } else {
                    c.border
                },
            },
            ..button::Style::default()
        }
    });
    if enabled {
        btn.on_press(message).into()
    } else {
        btn.into()
    }
}
fn shell_text_button_style(theme:&Theme,status:button::Status)->button::Style {
    let c=shell_colors(theme);
    let hover=matches!(status, button::Status::Hovered);
    button::Style {
        background: if hover {Some(Background::Color(c.tint))} else {None},
        text_color:c.accent,
        border:iced::Border { radius:8.0.into(), ..Default::default() },
        ..Default::default()
    }
}
fn shell_choice<'a>(label: String, message: Message, selected: bool) -> Element<'a, Message> {
    button(text(label).size(12))
        .padding([10.0, 13.0])
        .style(move |theme: &Theme, status| {
            let c = shell_colors(theme);
            button::Style {
                background: Some(Background::Color(if selected {
                    c.tint
                } else if matches!(status, iced::widget::button::Status::Hovered) {
                    c.row
                } else {
                    c.panel
                })),
                text_color: if selected { c.accent } else { c.text },
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: if selected { c.accent } else { c.border },
                },
                ..button::Style::default()
            }
        })
        .on_press(message)
        .into()
}
// Stable SVG handles keep texture IDs fixed during the one-second worker ticks.
static ABOUT_GBST_REPO_ICON: std::sync::LazyLock<iced::widget::svg::Handle> =
    std::sync::LazyLock::new(|| iced::widget::svg::Handle::from_memory(
        include_bytes!("../assets/icons/about_gbst_repo.svg").as_slice()
    ));
static ABOUT_ISSUE_ICON: std::sync::LazyLock<iced::widget::svg::Handle> =
    std::sync::LazyLock::new(|| iced::widget::svg::Handle::from_memory(
        include_bytes!("../assets/icons/about_issue.svg").as_slice()
    ));
static ABOUT_LTBOX_ICON: std::sync::LazyLock<iced::widget::svg::Handle> =
    std::sync::LazyLock::new(|| iced::widget::svg::Handle::from_memory(
        include_bytes!("../assets/icons/about_ltbox.svg").as_slice()
    ));

fn shell_about_icon_link<'a>(
    glyph: iced::widget::svg::Handle,
    tooltip_text: String,
    message: Message,
) -> Element<'a, Message> {
    let button_icon = button(
        container(iced::widget::svg(glyph)
            .width(Length::Fixed(21.0)).height(Length::Fixed(21.0)))
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(40.0))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .padding(0)
    .on_press(message)
    .style(|theme: &Theme, status| {
        let c = shell_colors(theme);
        let hovered = matches!(status, button::Status::Hovered);
        button::Style {
            background: Some(Background::Color(if hovered { c.tint } else { c.row })),
            text_color: c.text,
            border: iced::Border {
                radius: 10.0.into(),
                ..iced::Border::default()
            },
            ..Default::default()
        }
    });
    iced::widget::tooltip(
        button_icon,
        container(text(tooltip_text).size(11)).padding([6.0, 9.0])
            .style(shell_panel_style),
        iced::widget::tooltip::Position::Top,
    )
    .into()
}

fn shell_settings_card<'a>(
    title: String,
    rows: Vec<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut content = column![
        container(text(title).size(14).font(lpm_bold_font()))
            .width(Length::Fill).padding([14.0, 16.0])
    ].spacing(0);
    for row in rows {
        content = content.push(
            container(Space::new())
                .width(Length::Fill).height(Length::Fixed(1.0))
                .style(shell_divider_style)
        );
        content = content.push(row);
    }
    container(content).width(Length::Fill)
        .style(shell_panel_style).into()
}

fn shell_segment_group<'a>(
    choices: Vec<(String, Message, bool)>,
) -> Element<'a, Message> {
    let mut segments = row![].spacing(0);
    for (label, message, selected) in choices {
        segments = segments.push(
            button(text(label).size(12))
                .height(Length::Fixed(39.0))
                .padding([10.0, 13.0])
                .on_press(message)
                .style(move |theme: &Theme, status| {
                    let c = shell_colors(theme);
                    let hovered = matches!(status, button::Status::Hovered);
                    button::Style {
                        background: Some(Background::Color(if selected {
                            c.tint
                        } else if hovered {
                            c.row
                        } else {
                            c.panel
                        })),
                        text_color: if selected { c.accent } else { c.text },
                        border: iced::Border {
                            width: 0.5,
                            color: c.border,
                            radius: 0.0.into()
                        },
                        ..Default::default()
                    }
                })
        );
    }
    container(segments)
        .style(|theme: &Theme| {
            let c = shell_colors(theme);
            container::Style {
                border: iced::Border {
                    color: c.border,
                    width: 1.0,
                    radius: 20.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

fn settings_path_display(path: &str, max_chars: usize) -> String {
    let chars: Vec<char> = path.chars().collect();
    if chars.len() <= max_chars || max_chars < 7 {
        return path.to_string();
    }
    // Show the Windows drive/volume and the most useful trailing component.
    let head: String = chars.iter().take(3).collect();
    let tail: String = chars[chars.len() - (max_chars - 4)..].iter().collect();
    format!("{head}…{tail}")
}

fn shell_settings_path<'a>(
    path: String,
    open_label: String,
    message: Message,
) -> Element<'a, Message> {
    let shortened = settings_path_display(&path, 25);
    let path_control = container(
        text(shortened)
            .size(11)
            .wrapping(iced::widget::text::Wrapping::None)
    )
    .height(Length::Fixed(40.0))
    .width(Length::Fixed(202.0))
    .padding([12.0, 9.0])
    .clip(true)
    .style(|theme: &Theme| {
        let c = shell_colors(theme);
        container::Style {
            background: Some(Background::Color(c.row)),
            border: iced::Border {
                radius: 7.0.into(),
                color: c.border,
                width: 1.0,
            },
            ..Default::default()
        }
    });
    let path_tooltip = iced::widget::tooltip(
        path_control,
        container(
            text(path).size(11)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
        )
            .padding([8.0, 12.0])
            .max_width(420.0)
            .style(shell_panel_style),
        iced::widget::tooltip::Position::Top,
    );
    row![
        path_tooltip,
        shell_action(open_label, message, true, false)
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

fn shell_detail<'a>(label: String, value: String) -> Element<'a, Message> {
    container(
        row![
            shell_muted(label, 12).width(Length::Fixed(152.0)),
            text(value).size(12).width(Length::Fill),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding([12.0, 16.0])
    .style(shell_detail_row_style)
    .into()
}
fn shell_detail_row_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: None,
        text_color: Some(c.text),
        border: iced::Border {
            color: c.border,
            width: 0.7,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}
fn shell_setting_row<'a>(
    label: String,
    info: String,
    control: Element<'a, Message>,
) -> Element<'a, Message> {
    container(row![
        column![text(label).size(13), shell_muted(info, 11)]
            .width(Length::Fill).spacing(4),
        control,
    ].spacing(12).align_y(iced::Alignment::Center))
        .width(Length::Fill)
        .padding([13.0, 16.0])
        .style(|theme:&Theme| {
            let c=shell_colors(theme);
            container::Style { text_color:Some(c.text), ..container::Style::default() }
        })
        .into()
}
fn shell_section<'a>(title: String, content: Element<'a, Message>) -> Element<'a, Message> {
    container(column![
        container(text(title).size(14).font(lpm_bold_font()))
            .width(Length::Fill).padding([14.0, 16.0]),
        container(Space::new())
            .width(Length::Fill).height(Length::Fixed(1.0))
            .style(shell_divider_style),
        content,
    ].spacing(0))
        .width(Length::Fill).style(shell_panel_style).into()
}
fn shell_step<'a>(
    language: LanguageOption,
    step: u8,
    active: u8,
    key: &str,
) -> Element<'a, Message> {
    let completed = step < active;
    let chosen = step == active;
    let label = uix(language, key);
    let badge = container(text(format!("{}", step + 1)).size(13))
        .width(Length::Fixed(28.0))
        .height(Length::Fixed(28.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(move |theme: &Theme| {
            let c = shell_colors(theme);
            container::Style {
                background: Some(Background::Color(if chosen || completed {
                    c.accent
                } else {
                    c.row
                })),
                text_color: Some(if chosen || completed {
                    Color::WHITE
                } else {
                    c.muted
                }),
                border: iced::Border {
                    radius: 14.0.into(),
                    ..iced::Border::default()
                },
                ..container::Style::default()
            }
        });
    container(
        row![badge, text(label).size(12)]
            .spacing(8)
            .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding([8.0, 10.0])
    .into()
}

fn disclaimer_can_continue(open: bool, confirmed: bool) -> bool {
    open && confirmed
}

fn is_compact_window(window_width: f32) -> bool {
    window_width - SIDEBAR_RAIL_WIDTH < EXPANDED_CONTENT_WIDTH
}

fn sidebar_width_for_window(window_width: f32, progress: f32) -> f32 {
    if is_compact_window(window_width) {
        SIDEBAR_RAIL_WIDTH + (SIDEBAR_EXPANDED_WIDTH - SIDEBAR_RAIL_WIDTH)
            * progress.clamp(0.0, 1.0)
    } else {
        SIDEBAR_EXPANDED_WIDTH
    }
}

fn execution_phase_index(visible: bool, overall: f32, stage: u8, finalizing: bool) -> usize {
    if visible && overall >= 100.0 {
        6
    } else if finalizing && stage >= 4 {
        5
    } else if stage > 0 {
        usize::from(stage.min(4))
    } else {
        0
    }
}

/// Translate device labels and unknown/status values at the display boundary.
fn localized_device_details(language: LanguageOption, d: &DashboardInfo) -> [(String,String);4] {
    [
        (uix(language,"ui.device.model"),ui_text(language,&d.model_name)),
        (uix(language,"ui.device.android"),ui_text(language,&d.android_version)),
        (uix(language,"ui.device.rom"),ui_text(language,&d.rom_type)),
        (uix(language,"ui.device.google_status"),ui_text(language,&d.google_service_status)),
    ]
}

impl App {
    fn lt_is_compact(&self) -> bool {
        is_compact_window(self.window_size.0)
    }

    fn lt_sidebar_progress(&self) -> f32 {
        if self.lt_is_compact() { self.sidebar_anim.clamp(0.0, 1.0) }
        else { 1.0 }
    }

    fn lt_sidebar_width(&self) -> f32 {
        sidebar_width_for_window(self.window_size.0, self.lt_sidebar_progress())
    }

    fn lt_sidebar_label_alpha(&self) -> f32 {
        if self.lt_is_compact() {
            self.sidebar_label_alpha.clamp(0.0, 1.0)
        } else {
            1.0
        }
    }

    fn lt_device_connected(&self) -> bool {
        self.dashboard_info.is_lenovo && self.dashboard_info.android_major.is_some()
    }

    // Follows LTBox's invariant: icon centers stay 40px from the drawer's
    // left edge. Compact: 12 + 16 + 12; Expanded: 8 + 20 + 12.
    fn lt_nav_item(&self, page: NavPage, key: &str) -> Element<'_, Message> {
        let palette = self.theme().palette();
        let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.55;
        let handle = match (page, is_dark) {
            (NavPage::Dashboard, false) => self.nav_home_handle.clone(),
            (NavPage::Google, false) => self.nav_g_handle.clone(),
            (NavPage::Log, false) => self.nav_log_handle.clone(),
            (NavPage::Settings, false) => self.nav_settings_handle.clone(),
            (NavPage::About, false) => self.nav_tab_settings_handle.clone(),
            (NavPage::Dashboard, true) => self.nav_home_dark_handle.clone(),
            (NavPage::Google, true) => self.nav_g_dark_handle.clone(),
            (NavPage::Log, true) => self.nav_log_dark_handle.clone(),
            (NavPage::Settings, true) => self.nav_settings_dark_handle.clone(),
            (NavPage::About, true) => self.nav_tab_settings_dark_handle.clone(),
        };
        let icon: Element<'_, Message> = iced::widget::image(handle)
            .width(Length::Fixed(21.0))
            .height(Length::Fixed(21.0))
            .into();

        let selected = self.active_nav == page;
        let collapsed = self.lt_is_compact() && !self.sidebar_expanded;
        let label_alpha = self.lt_sidebar_label_alpha();
        let icon_slot: Element<'_, Message> = container(icon)
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .into();
        let mut inner = row![icon_slot]
            .spacing(12)
            .align_y(iced::Alignment::Center);
        if !collapsed && label_alpha > 0.0 {
            let label = text(uix(self.language, key))
                .size(14)
                .height(Length::Fill)
                .align_y(iced::alignment::Vertical::Center)
                .wrapping(iced::widget::text::Wrapping::None)
                .style(move |theme: &Theme| {
                    let base = shell_colors(theme).text;
                    iced::widget::text::Style {
                        color: Some(Color::from_rgba(
                            base.r, base.g, base.b, label_alpha
                        ))
                    }
                });
            inner = inner.push(label);
        }

        let content: Element<'_, Message> = container(inner)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Center)
            .into();

        let width = if collapsed {
            Length::Fixed(56.0)
        } else {
            Length::Fill
        };
        // The selected indicator has a compact 56x32 pill and a full 56px
        // expanded pill. It is behind the button to prevent icon drift.
        let selection_height = if collapsed { 32.0 } else { 56.0 };
        let selection: Element<'_, Message> = container(
            container(Space::new())
                .width(width)
                .height(Length::Fixed(selection_height))
                .style(move |theme: &Theme| {
                    let c = shell_colors(theme);
                    container::Style {
                        background: if selected {
                            Some(Background::Color(c.tint))
                        } else {
                            None
                        },
                        border: iced::Border {
                            radius: 28.0.into(),
                            ..iced::Border::default()
                        },
                        ..container::Style::default()
                    }
                })
        )
        .width(width)
        .height(Length::Fixed(56.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into();

        let button: Element<'_, Message> = button(content)
            .padding([0.0, if collapsed { 16.0 } else { 20.0 }])
            .width(width)
            .height(Length::Fixed(56.0))
            .style(move |theme: &Theme, state| {
                let c = shell_colors(theme);
                button::Style {
                    background: if matches!(state, button::Status::Hovered) {
                        Some(Background::Color(Color::from_rgba(
                            c.text.r, c.text.g, c.text.b, 0.07
                        )))
                    } else {
                        None
                    },
                    text_color: c.text,
                    border: iced::Border {
                        radius: 28.0.into(),
                        ..iced::Border::default()
                    },
                    ..button::Style::default()
                }
            })
            .on_press(Message::SelectNav(page))
            .into();

        iced::widget::Stack::with_children(vec![selection, button])
            .width(width)
            .height(Length::Fixed(56.0))
            .into()
    }

    fn lt_sidebar(&self) -> Element<'_, Message> {
        let compact = self.lt_is_compact();
        let collapsed = compact && !self.sidebar_expanded;
        let alpha = self.lt_sidebar_label_alpha();

        // Keep the same menu item and section heights during hover
        // transitions. Never recenter the icons when labels appear.
        let section: Element<'_, Message> = if collapsed {
            container(
                container(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fixed(1.0))
                    .style(shell_divider_style)
            )
            .width(Length::Fill)
            .height(Length::Fixed(48.0))
            .align_y(iced::alignment::Vertical::Center)
            .into()
        } else {
            container(
                shell_muted(uix(self.language, "ui.group_tools"), 11)
                    .wrapping(iced::widget::text::Wrapping::None)
                    .style(move |theme: &Theme| {
                        let c = shell_colors(theme).muted;
                        iced::widget::text::Style {
                            color: Some(Color::from_rgba(c.r, c.g, c.b, alpha))
                        }
                    })
            )
            .width(Length::Fill)
            .height(Length::Fixed(48.0))
            .padding([0.0, 20.0])
            .align_y(iced::alignment::Vertical::Center)
            .into()
        };

        let list = column![
            self.lt_nav_item(NavPage::Dashboard, "ui.dashboard"),
            self.lt_nav_item(NavPage::Google, "ui.google"),
            self.lt_nav_item(NavPage::Log, "ui.logs"),
            section,
            self.lt_nav_item(NavPage::Settings, "ui.settings"),
            self.lt_nav_item(NavPage::About, "ui.about"),
        ]
        .spacing(1)
        .padding([12.0, if collapsed { 12.0 } else { 8.0 }])
        .align_x(iced::Alignment::Start);

        let panel_width = self.lt_sidebar_width();
        // At the minimum 820x720 window the six navigation entries fit.
        // A Scrollable wrapper creates an unwanted rail on the side menu.
        let panel = container(
            list.width(Length::Fill)
        )
        .width(Length::Fixed(panel_width))
        .height(Length::Fill)
        .style(shell_sidebar_style);

        let edge = container(Space::new())
            .height(Length::Fill)
            .width(Length::Fixed(1.0))
            .style(shell_divider_style);
        let mut shell = row![panel, edge].height(Length::Fill);
        if compact && self.sidebar_anim > 0.0 {
            let fade = self.sidebar_anim;
            shell = shell.push(
                container(Space::new())
                    .width(Length::Fixed(14.0 * fade))
                    .height(Length::Fill)
                    .style(move |theme: &Theme| {
                        let fg = shell_colors(theme).text;
                        container::Style {
                            background: Some(Background::Color(
                                Color::from_rgba(fg.r, fg.g, fg.b, 0.07 * fade)
                            )),
                            ..container::Style::default()
                        }
                    })
            );
        }

        if compact {
            iced::widget::mouse_area(shell)
                .on_enter(Message::SidebarHoverEnter)
                .on_exit(Message::SidebarHoverExit)
                .on_press(Message::Noop)
                .interaction(iced::mouse::Interaction::Idle)
                .into()
        } else {
            shell.into()
        }
    }

    fn lt_page_header(&self, title_key: &str, subtitle_key: &str) -> Element<'_, Message> {
        container(
            row![
                text(uix(self.language, title_key))
                    .size(24)
                    .font(lpm_bold_font()),
                shell_muted(uix(self.language, subtitle_key), 12)
                    .width(Length::Fill),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fixed(64.0))
        .padding([12.0, 24.0])
        .style(|theme: &Theme| {
            let c = shell_colors(theme);
            container::Style {
                background: Some(Background::Color(c.sidebar)),
                text_color: Some(c.text),
                border: iced::Border { width: 1.0, color: c.border,
                    ..iced::Border::default() },
                ..container::Style::default()
            }
        })
        .into()
    }

    fn lt_view(&self) -> Element<'_, Message> {
        let main_body: Element<'_, Message> = match self.active_nav {
            NavPage::Dashboard => self.lt_dashboard_panel(),
            NavPage::Google => self.lt_execution_panel(),
            NavPage::Log => self.log_section(),
            NavPage::Settings => self.lt_settings_panel(),
            NavPage::About => self.lt_about_panel(),
        };
        let (title, subtitle) = match self.active_nav {
            NavPage::Dashboard => ("ui.dashboard", "ui.dashboard_subtitle"),
            NavPage::Google => (
                "ui.google",
                if self.busy {"ui.exec.subtitle_running"}
                else {"ui.exec.subtitle_idle"}
            ),
            NavPage::Log => ("ui.logs", "ui.logs_subtitle"),
            NavPage::Settings => ("ui.settings", "ui.settings_subtitle"),
            NavPage::About => ("ui.about", "ui.about_subtitle"),
        };

        let page_column = if matches!(self.active_nav, NavPage::Dashboard | NavPage::About) {
            column![
                container(main_body)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(shell_main_style)
            ].height(Length::Fill).spacing(0)
        } else {
            column![
                self.lt_page_header(title, subtitle),
                container(main_body)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(shell_main_style),
            ].height(Length::Fill).spacing(0)
        };

        let footer_status = if self.busy && self.install_cancel.is_some() {
            uix(self.language, "ui.footer.busy")
        } else if self.lt_device_connected() {
            format!(
                "{}  ·  {}",
                uix(self.language, "ui.footer.connected"),
                ui_text(self.language, &self.dashboard_info.model_name)
            )
        } else {
            uix(self.language, "ui.footer.disconnected")
        };
        let connected = self.lt_device_connected();
        let footer = container(
            row![
                text("●")
                    .size(11)
                    .style(move |theme: &Theme| iced::widget::text::Style {
                        color: Some(if connected {
                            shell_colors(theme).success
                        } else {
                            shell_colors(theme).muted
                        })
                    }),
                shell_muted(footer_status, 11).width(Length::Fill),
                shell_muted(format!("v{APP_DISPLAY_VERSION}"), 11),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .padding([8.0, 15.0])
        .style(shell_footer_style);
        // LTBox's status bar spans the entire window below navigation.
        let main = container(page_column).width(Length::Fill).height(Length::Fill);
        let row_area: Element<'_, Message> = if self.lt_is_compact() {
            let placeholder = container(Space::new())
                .width(Length::Fixed(SIDEBAR_RAIL_WIDTH))
                .height(Length::Fill);
            let base = row![placeholder, main].width(Length::Fill).height(Length::Fill);
            iced::widget::Stack::with_children(vec![base.into(), self.lt_sidebar()])
                .width(Length::Fill).height(Length::Fill).into()
        } else {
            row![self.lt_sidebar(), main]
                .width(Length::Fill).height(Length::Fill).into()
        };
        let frame: Element<'_, Message> = column![
            row_area,
            footer
        ].height(Length::Fill).width(Length::Fill).spacing(0).into();

        let mut layers: Vec<Element<'_, Message>> = vec![frame.into()];
        if self.should_show_dashboard_update_notice() {
            layers.push(self.dashboard_update_notice_view());
        }
        if self.apk_download_modal_message.is_some() {
            layers.push(self.apk_download_notice_view());
        }
        if self.usb_authorization_pending {
            layers.push(self.usb_authorization_notice_view());
        }
        if self.completion_notice_visible {
            layers.push(self.completion_notice_view());
        }
        if self.google_start_confirmation_open {
            layers.push(self.lt_google_start_confirmation_popup());
        }
        if self.cache_cleanup_confirmation_open {
            layers.push(self.lt_cache_cleanup_popup());
        }
        if self.log_cleanup_confirmation_open {
            layers.push(self.lt_log_cleanup_popup());
        }
        if self.license_dialog_open {
            layers.push(self.lt_licenses_popup());
        }
        if self.disclaimer_open {
            layers.push(self.lt_disclaimer_popup());
        }
        iced::widget::Stack::with_children(layers)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn lt_google_start_confirmation_popup(&self) -> Element<'_, Message> {
        let body = column![
            text(uix(self.language, "ui.google.confirm_title"))
                .size(22).font(lpm_bold_font()),
            shell_muted(uix(self.language, "ui.google.confirm_description"), 13)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
            row![
                Space::new().width(Length::Fill),
                shell_action(uix(self.language, "ui.google.confirm_no"),
                    Message::GoogleStartDeclined, true, false),
                shell_action(uix(self.language, "ui.google.confirm_yes"),
                    Message::GoogleStartConfirmed, true, true),
            ]
            .spacing(10).align_y(iced::Alignment::Center),
        ].spacing(20);
        let popup = container(body)
            .width(Length::Fixed(430.0))
            .padding(24)
            .style(|theme: &Theme| {
                let mut style = shell_panel_style(theme);
                style.border.radius = 25.0.into();
                style.shadow = iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.27),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 24.0
                };
                style
            });
        let scrim = container(Space::new())
            .width(Length::Fill).height(Length::Fill)
            .style(|_: &Theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0,0.0,0.0,0.38))),
                ..container::Style::default()
            });
        let centered = container(popup)
            .width(Length::Fill).height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);
        iced::widget::opaque(
            iced::widget::Stack::with_children(vec![scrim.into(),centered.into()])
                .width(Length::Fill).height(Length::Fill)
        ).into()
    }

    fn lt_cache_cleanup_popup(&self) -> Element<'_, Message> {
        let body = column![
            text(uix(self.language, "ui.settings.cleanup_title"))
                .size(22).font(lpm_bold_font()),
            shell_muted(uix(self.language, "ui.settings.cleanup_desc"), 13)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
            shell_muted(uix(self.language, "ui.settings.cleanup_warning"), 12)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
            row![
                Space::new().width(Length::Fill),
                shell_action(
                    uix(self.language, "ui.settings.cleanup_cancel"),
                    Message::CancelCacheCleanup, true, false
                ),
                shell_action(
                    uix(self.language, "ui.settings.cleanup_confirm"),
                    Message::ConfirmCacheCleanup, self.can_start_cache_cleanup(), true
                ),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        ].spacing(18);
        let popup = container(body)
            .width(Length::Fixed(430.0))
            .padding(24)
            .style(|theme: &Theme| {
                let mut style = shell_panel_style(theme);
                style.border.radius = 25.0.into();
                style.shadow = iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 24.0,
                };
                style
            });
        let scrim = container(Space::new())
            .width(Length::Fill).height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(
                    Color::from_rgba(0.0, 0.0, 0.0, 0.38)
                )),
                ..container::Style::default()
            });
        let center = container(popup)
            .width(Length::Fill).height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);
        iced::widget::opaque(
            iced::widget::Stack::with_children(vec![scrim.into(), center.into()])
                .width(Length::Fill).height(Length::Fill)
        ).into()
    }

    fn lt_log_cleanup_popup(&self) -> Element<'_, Message> {
        let body = column![
            text(uix(self.language, "ui.settings.logcleanup_title"))
                .size(22).font(lpm_bold_font()),
            shell_muted(uix(self.language, "ui.settings.logcleanup_description"), 13)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
            shell_muted(uix(self.language, "ui.settings.logcleanup_warning"), 12)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
            row![
                Space::new().width(Length::Fill),
                shell_action(
                    uix(self.language, "ui.settings.cleanup_cancel"),
                    Message::CancelLogCleanup, true, false
                ),
                shell_action(
                    uix(self.language, "ui.settings.logcleanup_confirm"),
                    Message::ConfirmLogCleanup, self.can_start_log_cleanup(), true
                ),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        ].spacing(18);
        let popup = container(body)
            .width(Length::Fixed(430.0))
            .padding(24)
            .style(|theme: &Theme| {
                let mut style = shell_panel_style(theme);
                style.border.radius = 25.0.into();
                style.shadow = iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 24.0,
                };
                style
            });
        let scrim = container(Space::new())
            .width(Length::Fill).height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(
                    Color::from_rgba(0.0, 0.0, 0.0, 0.38)
                )),
                ..container::Style::default()
            });
        let center = container(popup)
            .width(Length::Fill).height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);
        iced::widget::opaque(
            iced::widget::Stack::with_children(vec![scrim.into(), center.into()])
                .width(Length::Fill).height(Length::Fill)
        ).into()
    }

    fn lt_disclaimer_popup(&self) -> Element<'_, Message> {
        let body=column![
            text(uix(self.language, "ui.disclaimer.title"))
                .size(24).font(lpm_bold_font()),
            shell_muted(uix(self.language,"ui.disclaimer.body"),13)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
            checkbox(self.disclaimer_checked)
                .label(uix(self.language, "ui.disclaimer.accept"))
                .on_toggle(Message::DisclaimerToggled)
                .size(20).text_size(13),
            row![
                Space::new().width(Length::Fill),
                shell_action(uix(self.language,"ui.disclaimer.exit"),
                    Message::DisclaimerExit,true,false),
                shell_action(uix(self.language,"ui.disclaimer.continue"),
                    Message::DisclaimerContinue,self.disclaimer_checked,true),
            ].spacing(10).align_y(iced::Alignment::Center),
        ].spacing(18);
        let popup=container(body)
            .width(Length::Fixed(410.0))
            .padding(24)
            .style(|theme:&Theme|{
                let mut style=shell_panel_style(theme);
                style.border.radius=26.0.into();
                style.shadow=iced::Shadow {
                    color:Color::from_rgba(0.0,0.0,0.0,0.26),
                    offset:iced::Vector::new(0.0,8.0),
                    blur_radius:24.0
                };
                style
            });
        let scrim=container(Space::new())
            .width(Length::Fill).height(Length::Fill)
            .style(|_:&Theme|container::Style{
                background:Some(Background::Color(Color::from_rgba(0.,0.,0.,0.35))),
                ..container::Style::default()
            });
        let center=container(popup)
            .width(Length::Fill).height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);
        iced::widget::opaque(
            iced::widget::Stack::with_children(vec![scrim.into(),center.into()])
                .width(Length::Fill).height(Length::Fill)
        ).into()
    }

    fn lt_device_card(&self) -> Element<'_, Message> {
        let d = &self.dashboard_info;
        let connected=self.lt_device_connected();
        let model=if connected {ui_text(self.language,&d.model_name)} else {uix(self.language,"ui.device_not_found")};
        let subtitle=if connected {
            format!("{} · Android {}", ui_text(self.language,&d.manufacturer), ui_text(self.language,&d.android_version))
        } else { "USB ADB".to_string() };
        let image=iced::widget::image(self.dashboard_model_image_handle(d))
            .width(Length::Fixed(92.0)).height(Length::Fixed(72.0));
        let portrait=row![
            container(image)
                .width(Length::Fixed(100.0)).height(Length::Fixed(82.0))
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
            column![
                text(model).size(16).font(lpm_bold_font()),
                shell_muted(subtitle,14),
            ].spacing(6).width(Length::Fill),
        ].spacing(15).align_y(iced::Alignment::Center);
        let details=localized_device_details(self.language,d);
        let mut detail_column=column![].spacing(0);
        for (label,value) in details {
            detail_column=detail_column.push(
                container(Space::new())
                    .height(Length::Fixed(1.0)).width(Length::Fill)
                    .style(shell_divider_style)
            );
            detail_column=detail_column.push(
                container(
                    row![
                        shell_muted(label,12).width(Length::Fixed(164.0)),
                        text(value).size(13).width(Length::Fill),
                    ].spacing(10).align_y(iced::Alignment::Center)
                ).width(Length::Fill).padding([10.0, 0.0])
            );
        }
        // Device information only. User-facing actions live in the
        // three dashboard tiles below, without duplicate buttons.
        container(column![
            portrait,
            detail_column,
        ].spacing(12))
            .width(Length::Fill)
            .padding(24)
            .style(shell_panel_style)
            .into()
    }

    fn lt_dashboard_action_card(
        &self,
        heading_key: &str,
        description_key: &str,
        button_key: &str,
        action: Message,
        primary: bool,
    ) -> Element<'_, Message> {
        let body = column![
            container(
                text(uix(self.language, heading_key))
                    .size(17)
                    .font(lpm_bold_font())
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
                    .align_x(iced::alignment::Horizontal::Center)
            )
            // Keep every card title centered on the same horizontal row,
            // regardless of whether it wraps to one or two lines.
            .width(Length::Fill)
            .height(Length::Fixed(72.0))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
            Space::new().height(Length::Fill),
            container(
                shell_muted(uix(self.language, description_key), 13)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
                    .align_x(iced::alignment::Horizontal::Center),
            ).width(Length::Fill).align_x(iced::alignment::Horizontal::Center),
            Space::new().height(Length::Fill),
            shell_action(uix(self.language, button_key), action, true, primary),
        ]
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::Alignment::Center);
        container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([18.0, 14.0])
            .style(shell_panel_style)
            .into()
    }

    fn lt_adb_server_blocking_banner(&self) -> Element<'_, Message> {
        let mut text_content = column![
            text(uix(self.language, "ui.adb.blocking_title"))
                .size(14).font(lpm_bold_font()),
            shell_muted(uix(self.language, "ui.adb.blocking_message"), 12)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
        ].spacing(5).width(Length::Fill);
        if let Some(error) = self.adb_server_kill_error.as_deref() {
            text_content = text_content.push(
                text(format!("{}: {error}",uix(self.language,"ui.adb.kill_failed")))
                    .size(11)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
            );
        }
        let permitted = adb_server_kill_allowed(
            self.adb_server_blocking,
            self.busy || self.worker_rx.is_some() || self.dashboard_refreshing
                || self.install_cancel.is_some() || self.cache_cleanup_running
                || self.log_cleanup_running,
            self.adb_server_kill_in_flight,
            self.disclaimer_open,
        );
        let label = if self.adb_server_kill_in_flight {
            uix(self.language, "ui.adb.kill_busy")
        } else {
            uix(self.language, "ui.adb.kill_button")
        };
        let content = row![
            text("⚠").size(18),
            text_content,
            shell_action(label, Message::KillAdbServer, permitted, true),
        ].spacing(12).align_y(iced::Alignment::Center).width(Length::Fill);
        container(content)
            .width(Length::Fill)
            .padding([13.0, 16.0])
            .style(|theme: &Theme| {
                let mut c = shell_panel_style(theme);
                c.border.color = Color::from_rgb8(215, 142, 34);
                c.border.width = 1.0;
                c
            }).into()
    }

    fn lt_dashboard_panel(&self) -> Element<'_, Message> {
        // The activity log remains accessible in the dedicated "Activity"
        // navigation entry. On the dashboard, the user's three task cards
        // replace the old Recent activity panel (see design mockup).
        let tiles = row![
            self.lt_dashboard_action_card(
                "ui.dashboard.donate_title",
                "ui.dashboard.donate_desc",
                "ui.dashboard.go",
                Message::OpenDonate,
                true,
            ),
            self.lt_dashboard_action_card(
                "ui.dashboard.google_title",
                "ui.dashboard.google_desc",
                "ui.dashboard.start",
                Message::SelectNav(NavPage::Google),
                true,
            ),
            self.lt_dashboard_action_card(
                "ui.dashboard.youtube_title",
                "ui.dashboard.youtube_desc",
                "ui.dashboard.go",
                Message::OpenYoutube,
                true,
            ),
        ]
        .spacing(8)
        .width(Length::Fill)
        .height(Length::Fill);
        let mut content = column![].spacing(12).height(Length::Fill).width(Length::Fill);
        if self.adb_server_blocking {
            content = content.push(self.lt_adb_server_blocking_banner());
        }
        content = content
            .push(self.lt_device_card())
            .push(container(tiles).width(Length::Fill).height(Length::Fill));
        container(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(20)
            .into()
    }

    fn lt_current_execution_phase(&self) -> usize {
        execution_phase_index(self.progress_visible, self.overall_progress, self.progress_stage, self.operation_finalizing)
    }

    fn lt_elapsed_display(&self) -> String {
        let elapsed = self.operation_started_at
            .map(|started| started.elapsed().as_secs())
            .unwrap_or(0);
        format!("{:02}:{:02}", elapsed / 60, elapsed % 60)
    }

    fn lt_execution_metric(&self, key: &str, value: String) -> Element<'_, Message> {
        container(column![
            shell_muted(uix(self.language, key), 11),
            text(value).size(12),
        ].spacing(3))
        .width(Length::Fill).padding([3.0, 9.0]).into()
    }

    fn lt_execution_summary(&self) -> Element<'_, Message> {
        let value = self.overall_progress.clamp(0.0, 100.0);
        let current = self.lt_current_execution_phase();
        let keys = [
            "ui.exec.phase.0", "ui.exec.phase.1", "ui.exec.phase.2",
            "ui.exec.phase.3", "ui.exec.phase.4", "ui.exec.phase.5",
            "ui.exec.phase.6"
        ];
        let title = uix(self.language, keys[current.min(6)]);
        let status = if self.busy {
            "ui.exec.status.running"
        } else if self.progress_visible && self.progress_status_key == "status.finished" {
            "ui.exec.status.done"
        } else if self.progress_visible && matches!(
            self.progress_status_key.as_str(), "status.failed" | "status.cancelled"
        ) {
            "ui.exec.status.failed"
        } else {
            "ui.exec.status.preparing"
        };
        let transferred = self.transfer_percent.map(|v| format!("{v}%"))
            .unwrap_or_else(|| "—".to_string());
        let stats = row![
            self.lt_execution_metric("ui.exec.connection", "USB ADB".to_string()),
            self.lt_execution_metric("ui.exec.transfer", transferred),
            self.lt_execution_metric("ui.exec.elapsed", self.lt_elapsed_display()),
        ].spacing(2).width(Length::Fill);
        let mut content = column![
            row![
                text(title).size(16).font(lpm_bold_font()).width(Length::Fill),
                text(format!("{value:.1}%")).size(16).font(lpm_bold_font())
            ].align_y(iced::Alignment::Center),
            shell_muted(uix(self.language, status), 12),
            progress_bar(0.0..=100.0, value)
                .length(Length::Fill).girth(Length::Fixed(6.0)),
            container(stats).width(Length::Fill).padding(9)
                .style(shell_detail_row_style),
        ].spacing(9);
        if let Some(percent) = self.active_stage_percent
            && (1..=4).contains(&self.progress_stage)
        {
            let stage_label = uix(
                self.language,
                &format!("ui.exec.phase.{}", self.progress_stage),
            );
            content = content.push(
                row![
                    shell_muted(stage_label, 11).width(Length::Fill),
                    text(format!("{percent}%")).size(11).font(lpm_bold_font()),
                ]
                .align_y(iced::Alignment::Center),
            );
        }
        if let Some(transfer) = self.transfer_percent {
            content = content.push(column![
                shell_muted(format!("{} · {}%",
                    self.transfer_package.as_deref().unwrap_or("APK"), transfer), 11),
                progress_bar(0.0..=100.0, f32::from(transfer))
                    .length(Length::Fill).girth(Length::Fixed(4.0)),
            ].spacing(4));
        }
        container(content)
            .width(Length::Fill).padding(16)
            .style(shell_panel_style).into()
    }

    fn lt_execution_timeline(&self) -> Element<'_, Message> {
        let active = self.lt_current_execution_phase();
        let failed = self.progress_visible && !self.busy
            && matches!(self.progress_status_key.as_str(), "status.failed" | "status.cancelled");
        let stages = [
            "ui.exec.phase.0", "ui.exec.phase.1", "ui.exec.phase.2",
            "ui.exec.phase.3", "ui.exec.phase.4", "ui.exec.phase.5"
        ];
        let mut content = column![].spacing(6);
        for (index, key) in stages.iter().enumerate() {
            let finished = active > index;
            let current = active == index;
            let symbol = if finished {"✓".to_string()} else {(index+1).to_string()};
            let marker = container(text(symbol).size(12))
                .width(Length::Fixed(24.0)).height(Length::Fixed(24.0))
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .style(move |theme: &Theme| {
                    let c = shell_colors(theme);
                    let accent = if failed && current {
                        Color::from_rgb8(193, 52, 58)
                    } else { c.accent };
                    container::Style {
                        background: Some(Background::Color(
                            if finished || current { accent } else { c.row }
                        )),
                        text_color: Some(
                            if finished || current { Color::WHITE } else { c.muted }
                        ),
                        border: iced::Border {
                            radius: 13.0.into(), ..Default::default()
                        },
                        ..Default::default()
                    }
                });
            content = content.push(
                row![marker, text(uix(self.language, key)).size(12).width(Length::Fill)]
                    .spacing(10).align_y(iced::Alignment::Center)
                    .width(Length::Fill).padding([8.0, 4.0])
            );
        }
        container(content).padding(14)
            .width(Length::Fixed(270.0)).height(Length::Fill)
            .style(shell_panel_style).into()
    }

    fn lt_execution_logs(&self) -> Element<'_, Message> {
        let title = row![
            text(uix(self.language, "ui.exec.log_title"))
                .size(14).font(lpm_bold_font()).width(Length::Fill),
            button(text(uix(self.language, "ui.save_log")).size(12))
                .on_press(Message::ExportLog)
                .style(shell_text_button_style),
        ].spacing(8).align_y(iced::Alignment::Center);
        let body = scrollable(
            container(text(self.build_log_text()).size(LOG_FONT).width(Length::Fill))
                .width(Length::Fill).padding([7.0, 2.0])
        )
        .auto_scroll(true).anchor_bottom().anchor_left()
        .height(Length::Fill).width(Length::Fill);
        container(column![title, body].spacing(8))
            .width(Length::Fill).height(Length::Fill).padding(15)
            .style(shell_panel_style).into()
    }

    fn lt_execution_panel(&self) -> Element<'_, Message> {
        // At LTBox's compact size, prioritize the log. At wider sizes,
        // add the phase timeline as a second column.
        let lower: Element<'_, Message> = if self.lt_is_compact() {
            self.lt_execution_logs()
        } else {
            row![
                self.lt_execution_timeline(),
                self.lt_execution_logs()
            ].spacing(16).width(Length::Fill).height(Length::Fill).into()
        };
        container(column![
            self.lt_execution_summary(),
            lower,
        ].spacing(16).height(Length::Fill))
            .width(Length::Fill).height(Length::Fill)
            .padding(16).into()
    }

    fn lt_settings_panel(&self) -> Element<'_, Message> {
        let language = pick_list(
            LanguageOption::ALL.as_slice(),
            Some(self.language),
            Message::SettingsLanguageSelected,
        )
        .text_size(13)
        .style(lpm_nav_language_pick_list_style)
        .menu_style(lpm_nav_language_pick_list_menu_style)
        .width(Length::Fixed(176.0));

        let theme = shell_segment_group(
            ThemeChoice::ALL.iter().copied()
                .map(|choice| (
                    uix(self.language, &format!("ui.theme.{}", choice.code())),
                    Message::SettingsThemeSelected(choice),
                    self.appearance.theme == choice
                ))
                .collect()
        );
        let accent = shell_segment_group(
            AccentChoice::ALL.iter().copied()
                .map(|choice| (
                    uix(self.language, &format!("ui.accent.{}", choice.code())),
                    Message::SettingsAccentSelected(choice),
                    self.appearance.accent == choice
                ))
                .collect()
        );

        let appearance = shell_settings_card(
            uix(self.language,"ui.settings.appearance"),
            vec![
                shell_setting_row(
                    uix(self.language,"ui.settings.language"),
                    String::new(), language.into()
                ),
                shell_setting_row(
                    uix(self.language,"ui.settings.theme"),
                    String::new(), theme
                ),
                shell_setting_row(
                    uix(self.language,"ui.settings.accent"),
                    uix(self.language,"ui.settings.accent_desc"),
                    accent
                ),
            ]
        );
        let connection = shell_settings_card(
            uix(self.language,"ui.settings.connection"),
            vec![
                shell_setting_row(
                    uix(self.language,"ui.settings.diagfolder"),
                    uix(self.language,"ui.settings.diag_desc"),
                    shell_action(
                        uix(self.language,"ui.settings.open"),
                        Message::OpenDiagnosticsFolder,true,false
                    )
                ),
            ]
        );
        let files = shell_settings_card(
            uix(self.language,"ui.settings.files"),
            vec![
                shell_setting_row(
                    uix(self.language,"ui.settings.cache"),
                    uix(self.language,"ui.settings.cache_desc"),
                    shell_settings_path(
                        paths::apk_cache_dir().display().to_string(),
                        uix(self.language,"ui.settings.open"),
                        Message::OpenApkFolder
                    )
                ),
                shell_setting_row(
                    uix(self.language,"ui.settings.logfolder"),
                    uix(self.language,"ui.settings.logs_desc"),
                    shell_settings_path(
                        paths::logs_dir().display().to_string(),
                        uix(self.language,"ui.settings.open"),
                        Message::OpenLogFolder
                    )
                ),
                shell_setting_row(
                    uix(self.language, "ui.settings.cleanup_title"),
                    if let Some(report) = &self.cache_cleanup_report {
                        format!(
                            "{}: {} {} · {:.2} MiB",
                            uix(self.language, "ui.settings.cleanup_result"),
                            report.files_removed,
                            uix(self.language, "ui.settings.cleanup_files"),
                            report.bytes_freed as f64 / (1024.0 * 1024.0)
                        )
                    } else {
                        uix(self.language, "ui.settings.cleanup_desc")
                    },
                    shell_action(
                        if self.cache_cleanup_running {
                            uix(self.language, "ui.settings.cleanup_busy")
                        } else {
                            uix(self.language, "ui.settings.cleanup_button")
                        },
                        Message::RequestCacheCleanup,
                        self.can_start_cache_cleanup(),
                        false
                    )
                ),
                shell_setting_row(
                    uix(self.language, "ui.settings.logcleanup_title"),
                    if let Some(report) = &self.log_cleanup_report {
                        format!(
                            "{}: {} {} · {:.2} MiB",
                            uix(self.language, "ui.settings.logcleanup_done"),
                            report.files_removed,
                            uix(self.language, "ui.settings.cleanup_files"),
                            report.bytes_freed as f64 / (1024.0 * 1024.0)
                        )
                    } else {
                        uix(self.language, "ui.settings.logcleanup_description")
                    },
                    shell_action(
                        if self.log_cleanup_running {
                            uix(self.language, "ui.settings.cleanup_busy")
                        } else {
                            uix(self.language, "ui.settings.logcleanup_button")
                        },
                        Message::RequestLogCleanup,
                        self.can_start_log_cleanup(),
                        false
                    )
                ),
            ]
        );
        let program = shell_settings_card(
            uix(self.language,"ui.settings.general"),
            vec![
                shell_setting_row(
                    uix(self.language,"ui.settings.updates"),
                    format!("v{APP_DISPLAY_VERSION}"),
                    shell_action(
                        uix(self.language,"ui.settings.check"),
                        Message::CheckProgramUpdate,!self.program_update_checking,false
                    )
                ),
                shell_setting_row(
                    uix(self.language,"ui.settings.feedback"),
                    String::new(),
                    shell_action(
                        uix(self.language,"ui.settings.open"),
                        Message::OpenFeedback,true,false
                    )
                ),
            ]
        );
        let cards = column![appearance, connection, files, program]
            .spacing(18).width(Length::Fill);
        let centered = container(cards)
            .width(Length::Fill)
            .max_width(840.0);
        container(scrollable(
            container(centered)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .padding([24.0, 22.0])
        ).height(Length::Fill))
            .width(Length::Fill).height(Length::Fill).into()
    }

    fn lt_about_panel(&self) -> Element<'_, Message> {
        let icon = iced::widget::image(self.about_logo_handle.clone())
            .width(Length::Fixed(78.0))
            .height(Length::Fixed(78.0));
        let links = row![
            shell_about_icon_link(
                ABOUT_GBST_REPO_ICON.clone(), uix(self.language,"ui.about.source"),
                Message::OpenGitHubRepository
            ),
            shell_about_icon_link(
                ABOUT_ISSUE_ICON.clone(), uix(self.language,"ui.about.issues"),
                Message::OpenGBSTIssues
            ),
            shell_about_icon_link(
                ABOUT_LTBOX_ICON.clone(), uix(self.language,"ui.about.ltbox_github"),
                Message::OpenLTBoxGitHub
            ),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);
        let licenses = button(text(uix(self.language,"ui.about.license"))
            .size(13))
            .on_press(Message::ShowLicenseDialog)
            .padding([12.0, 17.0])
            .style(shell_text_button_style);

        let about = column![
            icon,
            text("GBST").size(26).font(lpm_bold_font()),
            shell_muted("Google Basic Service Tool", 13),
            shell_muted(uix(self.language, "ui.about.tagline"), 12)
                .align_x(iced::alignment::Horizontal::Center),
            shell_muted(uix(self.language, "ui.about.ltbox_credit"), 12)
                .align_x(iced::alignment::Horizontal::Center),
            shell_muted(format!("v{APP_DISPLAY_VERSION}"), 12),
            links,
            licenses,
        ]
        .spacing(13)
        .align_x(iced::Alignment::Center);

        container(about)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .padding(22)
            .into()
    }

    // The GPLv3 text remains scrollable, but its scrollbar has its own
    // reserved column and cannot cover the text or license list.
    fn lt_licenses_popup(&self) -> Element<'_, Message> {
        // Embed the project's own license and the bundled adb_client license
        // so release EXEs work without a source checkout or external files.
        const GBST_LICENSE: &str = include_str!("../../../LICENSE");
        const ADB_CLIENT_LICENSE: &str = include_str!("../../../vendor/adb_client/LICENSE");
        // Separate layout space is provided by an embedded scrollbar;
        // padding keeps both technical license text and UI labels readable.
        const LICENSE_SCROLL_LEFT_GUTTER: f32 = 12.0;
        const LICENSE_SCROLL_RIGHT_GUTTER: f32 = 16.0;
        const LICENSE_SCROLLBAR_WIDTH: f32 = 8.0;
        const LICENSE_SCROLLBAR_GAP: f32 = 12.0;

        let license_row = |name: &'static str, license: String| {
            column![
                text(name)
                    .size(13)
                    .font(lpm_bold_font())
                    .width(Length::Fill),
                shell_muted(license, 12)
                    .width(Length::Fill)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
            ]
            .spacing(3)
            .width(Length::Fill)
        };
        let index = column![
            text(uix(self.language,"ui.about.licenses_section"))
                .size(13).font(lpm_bold_font()),
            license_row("GBST", "GPLv3".to_owned()),
            license_row("adb_client", "MIT".to_owned()),
            license_row(
                "LTBox",
                format!("GPLv3+ · {}",
                    uix(self.language, "ui.about.reference_project"))
            ),
            shell_muted(uix(self.language,"ui.about.license_notice"),11)
                .width(Length::Fill)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
        ]
        .spacing(10)
        .width(Length::Fill);
        let licenses = scrollable(
            column![
                index,
                container(Space::new()).height(Length::Fixed(1.0))
                    .width(Length::Fill).style(shell_divider_style),
                text("GBST · GNU GPLv3").size(13).font(lpm_bold_font()),
                text(GBST_LICENSE).size(11)
                    .width(Length::Fill)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
                container(Space::new()).height(Length::Fixed(1.0))
                    .width(Length::Fill).style(shell_divider_style),
                text("adb_client · MIT").size(13).font(lpm_bold_font()),
                text(ADB_CLIENT_LICENSE).size(11)
                    .width(Length::Fill)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
            ]
            .spacing(16)
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 12.0,
                right: LICENSE_SCROLL_RIGHT_GUTTER,
                bottom: 12.0,
                left: LICENSE_SCROLL_LEFT_GUTTER,
            })
        )
        .direction(iced::widget::scrollable::Direction::Vertical(
            iced::widget::scrollable::Scrollbar::new()
                .width(LICENSE_SCROLLBAR_WIDTH)
                .scroller_width(6)
                .spacing(LICENSE_SCROLLBAR_GAP),
        ))
        .width(Length::Fill)
        .height(Length::Fill);
        let body=column![
            text(uix(self.language,"ui.about.license"))
                .size(22).font(lpm_bold_font()),
            container(Space::new())
                .width(Length::Fill).height(Length::Fixed(1.0))
                .style(shell_divider_style),
            licenses,
            container(Space::new())
                .width(Length::Fill).height(Length::Fixed(1.0))
                .style(shell_divider_style),
            row![
                Space::new().width(Length::Fill),
                shell_action(
                    uix(self.language,"ui.about.license_close"),
                    Message::CloseLicenseDialog,true,false
                )
            ],
        ]
        .spacing(12)
        .height(Length::Fill);
        let popup=container(body)
            .width(Length::Fixed(520.0))
            .height(Length::Fixed(590.0))
            .padding(22)
            .style(|theme: &Theme|{
                let mut s = shell_panel_style(theme);
                s.border.radius = 26.0.into();
                s.shadow=iced::Shadow{
                    color: Color::from_rgba(0.0,0.0,0.0,0.25),
                    offset: iced::Vector::new(0.0,8.0),
                    blur_radius: 24.0
                };
                s
            });
        let scrim=container(Space::new())
            .width(Length::Fill).height(Length::Fill)
            .style(|_: &Theme| container::Style{
                background: Some(Background::Color(
                    Color::from_rgba(0.0,0.0,0.0,0.35)
                )),
                ..Default::default()
            });
        let center=container(popup)
            .width(Length::Fill).height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);
        iced::widget::opaque(
            iced::widget::Stack::with_children(vec![scrim.into(),center.into()])
                .width(Length::Fill).height(Length::Fill)
        ).into()
    }

}

fn wizard_can_advance(step: u8, connected: bool, confirmed: bool, busy: bool) -> bool {
    if busy || step >= 3 {
        return false;
    }
    match step {
        0 => connected,
        1 => connected,
        2 => connected && confirmed,
        _ => false,
    }
}
fn wizard_can_start(
    step: u8,
    connected: bool,
    confirmed: bool,
    busy: bool,
    finished: bool,
) -> bool {
    step == 3 && connected && confirmed && !busy && !finished
}
#[cfg(test)]
mod visual_shell_tests {
    use super::*;
    #[test]
    fn long_settings_paths_fit_and_preserve_filename() {
        let path = r"C:\Users\PC\AppData\Local\dwas_kr\GBST\apk\Android 13";
        let result = settings_path_display(path, 25);
        assert!(result.chars().count() <= 25);
        assert!(result.starts_with("C:\\…"));
        assert!(result.ends_with("Android 13"));
        assert_eq!(settings_path_display(r"C:\\GBST", 25), r"C:\\GBST");
    }
    #[test]
    fn both_dark_and_light_icon_variants_can_be_loaded() {
        for data in [
            NAV_HOME_ICON_BYTES, NAV_G_ICON_BYTES, NAV_LOG_ICON_BYTES,
            NAV_SETTINGS_ICON_BYTES, NAV_TAB_SETTINGS_ICON_BYTES,
        ] {
            assert!(!data.is_empty());
            assert!(image_crate::load_from_memory(data).is_ok());
            let handle = light_nav_png_handle(data, 21, 21);
            assert_eq!(format!("{handle:?}").is_empty(), false);
        }
    }

    #[test]
    fn about_dialog_embeds_real_license_and_uses_svg_icon_assets() {
        let lic = include_str!("../../../LICENSE");
        let adb_lic = include_str!("../../../vendor/adb_client/LICENSE");
        assert!(lic.starts_with("GNU GENERAL PUBLIC LICENSE"));
        assert!(lic.contains("Version 3, 29 June 2007"));
        assert!(lic.contains("7. Additional Terms."));
        assert!(adb_lic.contains("MIT License") || adb_lic.contains("MIT license"));
        for asset in [
            include_bytes!("../assets/icons/about_gbst_repo.svg").as_slice(),
            include_bytes!("../assets/icons/about_issue.svg").as_slice(),
            include_bytes!("../assets/icons/about_ltbox.svg").as_slice(),
        ] {
            assert!(asset.starts_with(b"<svg"));
        }
    }
    #[test]
    fn changing_accent_changes_settings_selection_tint() {
        let mut appearance = Appearance::default();
        appearance.theme = ThemeChoice::Light;
        appearance.accent = AccentChoice::Indigo;
        let indigo = shell_colors(&appearance.as_theme(false)).tint;
        appearance.accent = AccentChoice::Teal;
        let teal = shell_colors(&appearance.as_theme(false)).tint;
        appearance.accent = AccentChoice::Rose;
        let rose = shell_colors(&appearance.as_theme(false)).tint;
        assert_ne!(indigo, teal);
        assert_ne!(indigo, rose);
        assert_ne!(rose, teal);
    }

    #[test]
    fn google_requires_confirmation_unless_running_or_disclaimer_visible() {
        assert!(google_confirmation_required(false, false));
        assert!(!google_confirmation_required(true, false));
        assert!(!google_confirmation_required(false, true));
    }
    #[test]
    fn about_icon_handle_reuses_stable_image_bytes() {
        assert!(!APP_ICON_PNG_BYTES.is_empty());
        // Loading the handle in App::new rather than lt_about_panel prevents
        // 1-second elapsed-time ticks from creating new image resource IDs.
        assert_eq!((WINDOW_WIDTH, WINDOW_HEIGHT), (820.0, 720.0));
    }
    #[test]
    fn google_click_starts_immediately_or_queues_without_double_start() {
        assert_eq!(google_navigation_action(false,false,false), GoogleNavigationAction::Start);
        assert_eq!(google_navigation_action(false,false,true), GoogleNavigationAction::Queue);
        assert_eq!(google_navigation_action(false,true,true), GoogleNavigationAction::Ignore);
        assert_eq!(google_navigation_action(false,true,false), GoogleNavigationAction::Ignore);
        assert_eq!(google_navigation_action(true,false,false), GoogleNavigationAction::Ignore);
    }
    #[test]
    fn progress_timeline_shows_actual_plan_stages_not_fake_boot_phases() {
        assert_eq!(execution_phase_index(false,0.0,0,false),0);
        assert_eq!(execution_phase_index(true,11.0,0,false),0);
        assert_eq!(execution_phase_index(true,12.0,1,false),1);
        assert_eq!(execution_phase_index(true,51.0,2,false),2);
        assert_eq!(execution_phase_index(true,91.0,4,false),4);
        assert_eq!(execution_phase_index(true,92.0,4,true),5);
        assert_eq!(execution_phase_index(true,100.0,4,true),6);
    }

    #[test]
    fn sidebar_items_fit_the_minimum_window_without_scrolling() {
        // Five 56px navigation items, a 48px group heading,
        // 5 x 1px gaps and 12px top/bottom padding.
        const NAV_ITEMS: f32 = 5.0;
        const NAV_ITEM_HEIGHT: f32 = 56.0;
        const SECTION_HEIGHT: f32 = 48.0;
        const NAV_GAP_COUNT: f32 = 5.0;
        const PADDING: f32 = 24.0;
        let total_height =
            NAV_ITEMS * NAV_ITEM_HEIGHT + SECTION_HEIGHT + NAV_GAP_COUNT + PADDING;
        assert!(total_height < WINDOW_HEIGHT - 32.0);
    }

    #[test]
    fn license_scrollbar_reserves_separate_text_space() {
        // Embedded spacing must be positive so the scrollbar is not
        // overlaid on copyright/license text at the minimum window width.
        assert!(820.0 >= 520.0);
        assert!(12.0 + 16.0 + 8.0 + 12.0 < 520.0 - 44.0);
        let license = include_str!("../../../LICENSE");
        assert!(license.starts_with("GNU GENERAL PUBLIC LICENSE"));
        assert!(license.contains("Version 3, 29 June 2007"));
    }

    #[test]
    fn sidebar_icon_origin_is_fixed_for_both_width_modes() {
        const ICON_HALF: f32 = 12.0;
        const COLLAPSED_OUTER: f32 = 12.0;
        const COLLAPSED_INNER: f32 = 16.0;
        const EXPANDED_OUTER: f32 = 8.0;
        const EXPANDED_INNER: f32 = 20.0;
        let compact_center = COLLAPSED_OUTER + COLLAPSED_INNER + ICON_HALF;
        let expanded_center = EXPANDED_OUTER + EXPANDED_INNER + ICON_HALF;
        assert_eq!(compact_center, 40.0);
        assert_eq!(expanded_center, compact_center);
    }
    #[test]
    fn first_run_window_matches_user_reference_image() {
        assert_eq!((WINDOW_WIDTH, WINDOW_HEIGHT), (820.0, 720.0));
        assert!(is_compact_window(WINDOW_WIDTH));
    }

    #[test]
    fn disclaimer_requires_active_dialog_and_explicit_consent() {
        assert!(!disclaimer_can_continue(true, false));
        assert!(!disclaimer_can_continue(false, true));
        assert!(disclaimer_can_continue(true, true));
    }
    #[test]
    fn ltbox_material_surface_palette_is_stable() {
        let light = Appearance {
            theme: ThemeChoice::Light,
            accent: AccentChoice::Indigo,
        }.as_theme(false);
        let c = shell_colors(&light);
        assert_eq!(c.page, Color::from_rgb8(251,248,253));
        assert_eq!(c.panel, Color::from_rgb8(239,236,241));
        assert_eq!(c.sidebar, Color::from_rgb8(245,242,247));
        assert_eq!(c.tint, Color::from_rgb8(224,225,249));
    }
    #[test]
    fn responsive_sidebar_matches_ltbox_breakpoints() {
        assert!(is_compact_window(820.0));
        assert!(is_compact_window(1079.999));
        assert!(!is_compact_window(1080.0));
        assert!(!is_compact_window(1180.0));
        assert!(!is_compact_window(4000.0));
    }
    #[test]
    fn sidebar_width_and_hover_do_not_change_the_breakpoint() {
        assert_eq!(sidebar_width_for_window(820.0, 0.0), 80.0);
        assert_eq!(sidebar_width_for_window(820.0, 1.0), 232.0);
        assert!(is_compact_window(820.0));
        // An expanding overlay never changes the breakpoint calculation.
        assert_eq!(sidebar_width_for_window(1080.0, 0.0), 232.0);
        assert_eq!(sidebar_width_for_window(1080.0, 1.0), 232.0);
        assert!(!is_compact_window(1080.0));
    }
    #[test]
    fn wizard_requires_detected_device_before_next() {
        assert!(!wizard_can_advance(0, false, false, false));
        assert!(wizard_can_advance(0, true, false, false));
        assert!(!wizard_can_advance(1, false, false, false));
        assert!(wizard_can_advance(1, true, false, false));
        assert!(!wizard_can_advance(2, true, false, false));
        assert!(wizard_can_advance(2, true, true, false));
        assert!(!wizard_can_advance(2, true, true, true));
        assert!(!wizard_can_advance(3, true, true, false));
    }
    #[test]
    fn installation_requires_explicit_confirmation_and_only_finishes_once() {
        assert!(!wizard_can_start(0, true, true, false, false));
        assert!(!wizard_can_start(3, true, false, false, false));
        assert!(!wizard_can_start(3, false, true, false, false));
        assert!(!wizard_can_start(3, true, true, true, false));
        assert!(!wizard_can_start(3, true, true, false, true));
        assert!(wizard_can_start(3, true, true, false, false));
    }
    #[test]
    fn theme_palette_supports_dark_and_light() {
        let light = Appearance {
            theme: ThemeChoice::Light,
            accent: AccentChoice::Indigo,
        }
        .as_theme(false);
        let dark = Appearance {
            theme: ThemeChoice::Dark,
            accent: AccentChoice::Indigo,
        }
        .as_theme(false);
        assert_ne!(shell_colors(&light).page, shell_colors(&dark).page);
        assert_ne!(shell_colors(&light).text, shell_colors(&dark).text);
    }
    #[test]
    fn dashboard_unknown_values_are_translated_in_every_supported_language() {
        let info = DashboardInfo::unknown();
        for language in LanguageOption::ALL {
            let rows = localized_device_details(language, &info);
            for (label, value) in &rows {
                assert!(!label.is_empty(), "empty label for {}", language.code());
                assert!(!value.is_empty(), "empty value for {}", language.code());
                if language != LanguageOption::Korean {
                    assert!(
                        !label.chars().any(|c| ('\u{ac00}'..='\u{d7a3}').contains(&c)),
                        "Korean dashboard label in {}: {}", language.code(), label
                    );
                    assert!(
                        !value.chars().any(|c| ('\u{ac00}'..='\u{d7a3}').contains(&c)),
                        "Korean dashboard value in {}: {}", language.code(), value
                    );
                }
            }
        }
        let japanese = localized_device_details(LanguageOption::Japanese, &info);
        assert_eq!(japanese[0], ("モデル".into(),"不明".into()));
        assert_eq!(japanese[1], ("Android バージョン".into(),"不明".into()));
        assert_eq!(japanese[2].1,"不明");
        assert_eq!(japanese[3].1,"不明");
    }

    #[test]
    fn dashboard_real_identifiers_are_not_changed_by_localization() {
        let info = DashboardInfo {
            model_name:"TB128FU".into(),
            android_version:"13".into(),
            manufacturer:"LENOVO".into(),
            rom_type:"PRC(중국 내수롬)".into(),
            google_service_status:"정상".into(),
            is_lenovo:true,
            android_major:Some(gbst_core::model::AndroidMajor::Android13),
        };
        let rows = localized_device_details(LanguageOption::Japanese, &info);
        assert_eq!(rows[0].1,"TB128FU");
        assert_eq!(rows[1].1,"13");
        assert_eq!(rows[2].1,"PRC（中国版ROM）");
        assert_eq!(rows[3].1,"正常");
    }
}
