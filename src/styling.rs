use std::collections::HashMap;

use crate::ui;



struct Stylesheet {
    rules: Vec<Rule>
}

type PropertyMap = HashMap<String, Vec<Style>>;
struct Rule {
    selector: Selector,
    properties: PropertyMap,
}

struct Selector {
    classes: Vec<String>,
    states: Vec<String>
}

enum Style {
    Color(ui::Color),
    Enum(String),
    Number(f64)
}






