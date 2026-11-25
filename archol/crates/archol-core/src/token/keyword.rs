//! Reserved words of the Archol language.
//!
//! Each word has a test file in `tests/lexer.rs`, but the table lives in a
//! single place to make it easy to audit what is a keyword and what is a
//! regular identifier.

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Keyword {
    Application,
    Service,
    Document,
    Process,
    View,
    Form,
    Compute,
    Report,
    Message,
    Shape,
    Role,
    Organogram,
    Policy,
    Decision,
    Integration,
    Parameter,
    Secret,
    Icon,
    Manual,
    Error,
    Value,
    Transition,
    End,
    If,
    Else,
    Try,
    Catch,
    Return,
    Const,
    Let,
    On,
    By,
    Schedule,
    FanOut,
    Called,
    Start,
    Imports,
    As,
    Where,
    In,
    True,
    False,
    Null,
}

impl Keyword {
    pub fn from_word(text: &str) -> Option<Self> {
        use Keyword::*;
        Some(match text {
            "application" => Application,
            "service" => Service,
            "document" => Document,
            "process" => Process,
            "view" => View,
            "form" => Form,
            "compute" => Compute,
            "report" => Report,
            "message" => Message,
            "shape" => Shape,
            "role" => Role,
            "organogram" => Organogram,
            "policy" => Policy,
            "decision" => Decision,
            "integration" => Integration,
            "parameter" => Parameter,
            "secret" => Secret,
            "icon" => Icon,
            "manual" => Manual,
            "error" => Error,
            "value" => Value,
            "transition" => Transition,
            "end" => End,
            "if" => If,
            "else" => Else,
            "try" => Try,
            "catch" => Catch,
            "return" => Return,
            "const" => Const,
            "let" => Let,
            "on" => On,
            "by" => By,
            "schedule" => Schedule,
            "fanOut" => FanOut,
            "called" => Called,
            "start" => Start,
            "imports" => Imports,
            "as" => As,
            "where" => Where,
            "in" => In,
            "true" => True,
            "false" => False,
            "null" => Null,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        use Keyword::*;
        match self {
            Application => "application",
            Service => "service",
            Document => "document",
            Process => "process",
            View => "view",
            Form => "form",
            Compute => "compute",
            Report => "report",
            Message => "message",
            Shape => "shape",
            Role => "role",
            Organogram => "organogram",
            Policy => "policy",
            Decision => "decision",
            Integration => "integration",
            Parameter => "parameter",
            Secret => "secret",
            Icon => "icon",
            Manual => "manual",
            Error => "error",
            Value => "value",
            Transition => "transition",
            End => "end",
            If => "if",
            Else => "else",
            Try => "try",
            Catch => "catch",
            Return => "return",
            Const => "const",
            Let => "let",
            On => "on",
            By => "by",
            Schedule => "schedule",
            FanOut => "fanOut",
            Called => "called",
            Start => "start",
            Imports => "imports",
            As => "as",
            Where => "where",
            In => "in",
            True => "true",
            False => "false",
            Null => "null",
        }
    }

    /// Words that start a top-level artifact.
    pub fn is_artifact(self) -> bool {
        matches!(
            self,
            Keyword::Application
                | Keyword::Service
                | Keyword::Document
                | Keyword::Process
                | Keyword::View
                | Keyword::Form
                | Keyword::Compute
                | Keyword::Report
                | Keyword::Message
                | Keyword::Shape
                | Keyword::Role
                | Keyword::Organogram
                | Keyword::Policy
                | Keyword::Decision
                | Keyword::Integration
                | Keyword::Parameter
                | Keyword::Secret
                | Keyword::Icon
                | Keyword::Manual
                | Keyword::Error
        )
    }

    /// JSON name of the artifact (`Document`, `Process`, ...).
    pub fn artifact_name(self) -> &'static str {
        use Keyword::*;
        match self {
            Application => "Application",
            Service => "Service",
            Document => "Document",
            Process => "Process",
            View => "View",
            Form => "Form",
            Compute => "Step",
            Report => "Step",
            Message => "Message",
            Shape => "Shape",
            Role => "Role",
            Organogram => "Organogram",
            Policy => "Policy",
            Decision => "Decision",
            Integration => "Integration",
            Parameter => "Parameter",
            Secret => "Secret",
            Icon => "Icon",
            Manual => "Manual",
            Error => "Error",
            _ => "",
        }
    }
}
