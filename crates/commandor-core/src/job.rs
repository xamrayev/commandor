//! Job holat mashinasi ([arxitektura](../../docs/ARCHITECTURE.uz.md#ishlar-navbati-va-resurs-chegaralari)).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JobState {
    Queued,
    Running,
    Paused,
    WaitingForUser,
    Retrying,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("job holati {from:?} → {to:?} ruxsat etilmagan")]
pub struct TransitionError {
    pub from: JobState,
    pub to: JobState,
}

impl JobState {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    pub const fn can_transition_to(self, next: JobState) -> bool {
        use JobState::*;
        matches!(
            (self, next),
            (Queued, Running | Cancelled)
                | (
                    Running,
                    Paused | WaitingForUser | Retrying | Completed | Failed | Cancelled
                )
                | (Paused, Running | Cancelled)
                | (WaitingForUser, Running | Cancelled)
                | (Retrying, Running | Failed | Cancelled)
        )
    }

    pub fn transition(self, next: JobState) -> Result<JobState, TransitionError> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err(TransitionError {
                from: self,
                to: next,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::JobState::*;
    use super::*;

    const ALL: [JobState; 8] = [
        Queued,
        Running,
        Paused,
        WaitingForUser,
        Retrying,
        Completed,
        Failed,
        Cancelled,
    ];

    #[test]
    fn terminal_states_never_leave() {
        for from in ALL.iter().copied().filter(|s| s.is_terminal()) {
            for to in ALL {
                assert!(from.transition(to).is_err(), "{from:?} -> {to:?}");
            }
        }
    }

    #[test]
    fn happy_path_and_cancel() {
        let s = Queued
            .transition(Running)
            .and_then(|s| s.transition(Completed));
        assert_eq!(s, Ok(Completed));
        assert_eq!(Paused.transition(Cancelled), Ok(Cancelled));
        assert!(
            Queued.transition(Completed).is_err(),
            "bajarilmagan ish tugagan deb ko‘rsatilmaydi"
        );
    }

    #[test]
    fn every_non_terminal_state_can_be_cancelled() {
        for s in ALL.iter().copied().filter(|s| !s.is_terminal()) {
            assert!(s.can_transition_to(Cancelled), "{s:?}");
        }
    }
}
