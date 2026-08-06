#![deny(missing_docs)]
use super::Predicate;

/// Possible commands that can be used to communicate with the frontend.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Command {
    /// Informs about the screen size.
    ScreenSize((u64, u64)),
    /// Informs about the cursor position.
    Position((f64, f64)),
    /// Informs about the current input text.
    InputText(String),
    /// Informs about the current output text.
    OutputText(String),
    /// Informs about the max numbers of predicate by page.
    PageSize(usize),
    /// Whether the backend is in IDLE.
    State(bool),
    /// Information about a predicate.
    ///
    /// Use cases:
    /// - Send a new predicate to the frontend.
    /// - Send the selected predicate to the backend.
    Predicate(Predicate),
    /// Informs that there is no selected predicate.
    NoPredicate,
    /// Request to update the data.
    Update,
    /// Request to clear the data.
    Clear,
    /// Request to select the previous predicate.
    SelectPreviousPredicate,
    /// Request to select the next predicate.
    SelectNextPredicate,
    /// Request to get the selected predicate..
    SelectedPredicate,
    /// Informs about no operation available.
    NOP,
    /// Requests to end the communication.
    End,
}
