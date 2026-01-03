use crate::core::{DataElement, data_element::Vr, tag::Tag};

pub(crate) struct ElementInDataSet {
    pub(crate) element: DataElement,
    pub(crate) position: u64,
    pub(crate) parent_index: Option<usize>,
}

impl ElementInDataSet {
    pub fn tag(&self) -> Tag {
        self.element.tag()
    }

    pub fn vr(&self) -> Option<Vr> {
        self.element.vr()
    }

    pub fn value_length(&self) -> u32 {
        self.element.value_length()
    }

    pub fn value_field(&self) -> &[u8] {
        self.element.value_field()
    }

    pub fn size(&self) -> usize {
        self.element.size()
    }
}

impl From<ElementInDataSet> for Vec<u8> {
    fn from(v: ElementInDataSet) -> Vec<u8> {
        v.element.into()
    }
}
