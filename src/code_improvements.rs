use colored::Colorize;
use serde::{Deserialize, Deserializer};
use std::fmt;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    High,
    Medium,
    Low,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the corresponding string representation to the formatter
        match self {
            Severity::High => write!(f, "High"),
            Severity::Medium => write!(f, "Medium"),
            Severity::Low => write!(f, "Low"),
        }
    }
}

impl<'de> Deserialize<'de> for Severity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "high" => Ok(Severity::High),
            "medium" => Ok(Severity::Medium),
            "low" => Ok(Severity::Low),
            _ => Err(serde::de::Error::custom("Unexpected severity value")),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Improvement {
    code: String,
    reason: String,
    pub severity: Severity,
}

#[derive(Deserialize, Debug)]
pub struct Improvements {
    pub improvements: Vec<Improvement>,
}

impl Improvements {
    pub fn display(&self) {
        for improvement in &self.improvements {
            let color = match improvement.severity {
                Severity::High => "red",
                Severity::Medium => "yellow",
                Severity::Low => "green",
            };

            println!(
                "Severity: {}\n",
                improvement.severity.to_string().color(color)
            );
            println!("{}\n", improvement.code.color(color));
            println!("{}\n", improvement.reason);
            println!("------------------------------------");
        }
    }
}
