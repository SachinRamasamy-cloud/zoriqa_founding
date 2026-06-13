use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Ident(String),
}

impl Value {
    pub fn as_str(&self) -> &str {
        match self {
            Value::String(s) => s,
            Value::Ident(s) => s,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StyleOverrides {
    pub bg: Option<String>,
    pub text: Option<String>,
    pub border: Option<String>,
    pub radius: Option<String>,
    pub shadow: Option<String>,
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tone {
    Primary,
    Success,
    Warning,
    Danger,
    Info,
    Neutral,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Variant {
    Soft,
    Solid,
    Outline,
    Dark,
    Light,
    Minimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StateFlag {
    Sticky,
    Popular,
    Disabled,
    Loading,
    Shadow,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResolvedFlags {
    pub tone: Option<Tone>,
    pub variant: Option<Variant>,
    pub state: Vec<StateFlag>,
    pub custom: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ElementNode {
    pub tag: String,
    pub args: Vec<Value>,
    pub props: HashMap<String, Value>,
    pub flags: Vec<String>,
    pub children: Vec<Node>,
    pub style: StyleOverrides,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct TextNode {
    pub content: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct ComponentNode {
    pub name: String,
    pub args: Vec<Value>,
    pub props: HashMap<String, Value>,
    pub flags: ResolvedFlags,
    pub children: Vec<Node>,
    pub style: StyleOverrides,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum Node {
    Element(ElementNode),
    Text(TextNode),
    Component(ComponentNode),
}

impl Node {
    #[allow(dead_code)]
    pub fn line(&self) -> usize {
        match self {
            Node::Element(e) => e.line,
            Node::Text(t) => t.line,
            Node::Component(c) => c.line,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThemeDecl {
    #[allow(dead_code)]
    pub name: Option<String>,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct LayoutDecl {
    pub name: String,
    pub children: Vec<Node>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct PageDecl {
    pub name: String,
    pub layout: Option<String>,
    pub title: Option<String>,
    pub children: Vec<Node>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct ComponentDecl {
    pub name: String,
    pub params: Vec<String>,
    pub children: Vec<Node>,
    #[allow(dead_code)]
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum TopLevel {
    Import(String),
    Theme(ThemeDecl),
    Layout(LayoutDecl),
    Page(PageDecl),
    Component(ComponentDecl),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub declarations: Vec<TopLevel>,
}
