use std::fmt;

use crate::traits::{CommunicationStyle, MoodState};

#[derive(Debug, Clone)]
pub enum PersonalityEvent {
    MoodChanged {
        previous: MoodState,
        current: MoodState,
    },
    TraitUpdated {
        trait_name: String,
        old_value: f64,
        new_value: f64,
    },
    ProfileLoaded {
        profile_id: String,
    },
    ProfileSaved {
        profile_id: String,
    },
    StyleChanged {
        previous: CommunicationStyle,
        current: CommunicationStyle,
    },
}

impl fmt::Display for PersonalityEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MoodChanged { previous, current } => {
                write!(f, "Mood changed: {:?} -> {:?}", previous, current)
            }
            Self::TraitUpdated {
                trait_name,
                old_value,
                new_value,
            } => {
                write!(
                    f,
                    "Trait updated: {} from {} to {}",
                    trait_name, old_value, new_value
                )
            }
            Self::ProfileLoaded { profile_id } => {
                write!(f, "Profile loaded: {}", profile_id)
            }
            Self::ProfileSaved { profile_id } => {
                write!(f, "Profile saved: {}", profile_id)
            }
            Self::StyleChanged { previous, current } => {
                write!(f, "Style changed: {:?} -> {:?}", previous, current)
            }
        }
    }
}
