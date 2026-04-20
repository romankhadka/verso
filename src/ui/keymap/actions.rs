use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    // Movement
    MoveDown, MoveUp, PageDown, PageUp, HalfPageDown, HalfPageUp,
    GotoTop, GotoBottom, NextChapter, PrevChapter,
    // Counts / commands
    BeginCount(u8), BeginCmd, BeginSearchFwd, BeginSearchBack,
    SearchNext, SearchPrev,
    // Marks
    MarkSetPrompt, MarkJumpPrompt,
    // Highlights
    VisualSelect, YankHighlight, ListHighlights,
    // View
    ToggleTheme, CycleWidth, Help,
    // Quit
    QuitToLibrary,
}

impl FromStr for Action {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "move_down" => Action::MoveDown,
            "move_up" => Action::MoveUp,
            "page_down" => Action::PageDown,
            "page_up" => Action::PageUp,
            "half_page_down" => Action::HalfPageDown,
            "half_page_up" => Action::HalfPageUp,
            "goto_top" => Action::GotoTop,
            "goto_bottom" => Action::GotoBottom,
            "next_chapter" => Action::NextChapter,
            "prev_chapter" => Action::PrevChapter,
            "cmd" => Action::BeginCmd,
            "search_forward" => Action::BeginSearchFwd,
            "search_backward" => Action::BeginSearchBack,
            "search_next" => Action::SearchNext,
            "search_prev" => Action::SearchPrev,
            "mark_set" => Action::MarkSetPrompt,
            "mark_jump" => Action::MarkJumpPrompt,
            "visual_select" => Action::VisualSelect,
            "yank_highlight" => Action::YankHighlight,
            "list_highlights" => Action::ListHighlights,
            "toggle_theme" => Action::ToggleTheme,
            "cycle_width" => Action::CycleWidth,
            "help" => Action::Help,
            "quit_to_library" => Action::QuitToLibrary,
            other => return Err(format!("unknown action: {other}")),
        })
    }
}
