use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct TextAttr {
    pub alignment: TextAlignment,
    pub style: TextStyle,
}

pub trait AsTextWithAttr {
    fn as_text_with_attr(&self, attr: TextAttr) -> Text;
}

impl<T> AsTextWithAttr for T
where
    T: ToString,
{
    fn as_text_with_attr(&self, attr: TextAttr) -> Text {
        Text::with_section(self.to_string(), attr.style, attr.alignment)
    }
}
