use crate::state::document::TabGroup;
use katana_core::document::Document;
use std::collections::HashSet;

pub(crate) enum DrawItem<'a> {
    GroupHeader(&'a TabGroup),
    Tab {
        idx: usize,
        group: Option<&'a TabGroup>,
    },
}

pub(crate) struct DrawItemCollector<'a> {
    pub open_documents: &'a [Document],
    pub tab_groups: &'a [TabGroup],
}

impl<'a> DrawItemCollector<'a> {
    pub fn collect(self) -> Vec<DrawItem<'a>> {
        let grouped = self.find_grouped_indices();
        let mut items = Vec::new();
        self.push_pinned(&mut items);
        self.push_groups(&mut items);
        self.push_ungrouped(&mut items, &grouped);
        items
    }

    fn find_grouped_indices(&self) -> HashSet<usize> {
        let mut grouped = HashSet::new();
        for g in self.tab_groups {
            for (idx, doc) in self.open_documents.iter().enumerate() {
                if g.members.contains(&doc.path.display().to_string()) {
                    grouped.insert(idx);
                }
            }
        }
        grouped
    }

    fn push_pinned(&self, items: &mut Vec<DrawItem<'a>>) {
        for (idx, doc) in self.open_documents.iter().enumerate() {
            if doc.is_pinned {
                items.push(DrawItem::Tab { idx, group: None });
            }
        }
    }

    fn push_groups(&self, items: &mut Vec<DrawItem<'a>>) {
        for g in self.tab_groups {
            items.push(DrawItem::GroupHeader(g));
            for (idx, doc) in self.open_documents.iter().enumerate() {
                if !doc.is_pinned && g.members.contains(&doc.path.display().to_string()) {
                    items.push(DrawItem::Tab { idx, group: Some(g) });
                }
            }
        }
    }

    fn push_ungrouped(&self, items: &mut Vec<DrawItem<'a>>, grouped: &HashSet<usize>) {
        for (idx, doc) in self.open_documents.iter().enumerate() {
            if !doc.is_pinned && !grouped.contains(&idx) {
                items.push(DrawItem::Tab { idx, group: None });
            }
        }
    }
}
