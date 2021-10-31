use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct TextAttr {
    pub alignment: TextAlignment,
    pub style: TextStyle,
}

pub trait AsText {
    fn as_text(&self, attr: &TextAttr) -> Text;
}

impl<T> AsText for T
where
    T: ToString,
{
    fn as_text(&self, attr: &TextAttr) -> Text {
        Text::with_section(self.to_string(), attr.style.clone(), attr.alignment.clone())
    }
}
