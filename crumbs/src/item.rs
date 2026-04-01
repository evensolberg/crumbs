use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Open,
    InProgress,
    Blocked,
    Deferred,
    Closed,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Blocked => write!(f, "blocked"),
            Self::Deferred => write!(f, "deferred"),
            Self::Closed => write!(f, "closed"),
        }
    }
}

impl std::str::FromStr for Status {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(Self::Open),
            "in_progress" | "in-progress" => Ok(Self::InProgress),
            "blocked" => Ok(Self::Blocked),
            "deferred" => Ok(Self::Deferred),
            "closed" => Ok(Self::Closed),
            other => Err(format!("unknown status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    #[default]
    Task,
    Bug,
    Feature,
    Epic,
    Idea,
}

impl std::fmt::Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Task => write!(f, "task"),
            Self::Bug => write!(f, "bug"),
            Self::Feature => write!(f, "feature"),
            Self::Epic => write!(f, "epic"),
            Self::Idea => write!(f, "idea"),
        }
    }
}

impl std::str::FromStr for ItemType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "task" => Ok(Self::Task),
            "bug" => Ok(Self::Bug),
            "feature" => Ok(Self::Feature),
            "epic" => Ok(Self::Epic),
            "idea" => Ok(Self::Idea),
            other => Err(format!("unknown type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub status: Status,
    #[serde(rename = "type")]
    #[allow(clippy::struct_field_names)]
    pub item_type: ItemType,
    pub priority: u8,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created: NaiveDate,
    pub updated: NaiveDate,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub closed_reason: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocks: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_by: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due: Option<NaiveDate>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub story_points: Option<u8>,
}

pub const FIBONACCI: &[u8] = &[1, 2, 3, 5, 8, 13, 21];

#[must_use]
pub fn is_fibonacci(n: u8) -> bool {
    FIBONACCI.contains(&n)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Status ---

    #[test]
    fn status_display() {
        assert_eq!(Status::Open.to_string(), "open");
        assert_eq!(Status::InProgress.to_string(), "in_progress");
        assert_eq!(Status::Blocked.to_string(), "blocked");
        assert_eq!(Status::Deferred.to_string(), "deferred");
        assert_eq!(Status::Closed.to_string(), "closed");
    }

    #[test]
    fn status_from_str_valid() {
        assert_eq!("open".parse::<Status>().unwrap(), Status::Open);
        assert_eq!("in_progress".parse::<Status>().unwrap(), Status::InProgress);
        assert_eq!("in-progress".parse::<Status>().unwrap(), Status::InProgress);
        assert_eq!("blocked".parse::<Status>().unwrap(), Status::Blocked);
        assert_eq!("deferred".parse::<Status>().unwrap(), Status::Deferred);
        assert_eq!("closed".parse::<Status>().unwrap(), Status::Closed);
    }

    #[test]
    fn status_from_str_invalid() {
        assert!("done".parse::<Status>().is_err());
        assert!("".parse::<Status>().is_err());
        assert!("OPEN".parse::<Status>().is_err());
    }

    #[test]
    fn status_round_trip() {
        for s in [
            Status::Open,
            Status::InProgress,
            Status::Blocked,
            Status::Deferred,
            Status::Closed,
        ] {
            assert_eq!(s.to_string().parse::<Status>().unwrap(), s);
        }
    }

    // --- ItemType ---

    #[test]
    fn item_type_display() {
        assert_eq!(ItemType::Task.to_string(), "task");
        assert_eq!(ItemType::Bug.to_string(), "bug");
        assert_eq!(ItemType::Feature.to_string(), "feature");
        assert_eq!(ItemType::Epic.to_string(), "epic");
        assert_eq!(ItemType::Idea.to_string(), "idea");
    }

    #[test]
    fn item_type_from_str_valid() {
        assert_eq!("task".parse::<ItemType>().unwrap(), ItemType::Task);
        assert_eq!("bug".parse::<ItemType>().unwrap(), ItemType::Bug);
        assert_eq!("feature".parse::<ItemType>().unwrap(), ItemType::Feature);
        assert_eq!("epic".parse::<ItemType>().unwrap(), ItemType::Epic);
        assert_eq!("idea".parse::<ItemType>().unwrap(), ItemType::Idea);
    }

    #[test]
    fn item_type_from_str_invalid() {
        assert!("story".parse::<ItemType>().is_err());
        assert!("Task".parse::<ItemType>().is_err());
        assert!("".parse::<ItemType>().is_err());
    }

    #[test]
    fn item_type_round_trip() {
        for t in [
            ItemType::Task,
            ItemType::Bug,
            ItemType::Feature,
            ItemType::Epic,
            ItemType::Idea,
        ] {
            assert_eq!(t.to_string().parse::<ItemType>().unwrap(), t);
        }
    }
}
