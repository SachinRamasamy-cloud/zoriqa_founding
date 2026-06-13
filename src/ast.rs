#[derive(Debug, Clone)]
pub struct Program {
    pub page_name: String,
    pub title: Option<String>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: String,
    pub value: Option<String>,
    pub flags: Vec<String>,
    pub props: Vec<Property>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub value: String,
}
