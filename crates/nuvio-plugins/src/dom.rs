use scraper::{ElementRef, Html, Selector};
use std::collections::{HashMap, VecDeque};

const MAX_CACHED_DOCUMENTS: usize = 8;

/// Pure-Rust HTML DOM bridge replacing `ksoup` (`DomBridge.kt`), powered by `scraper`.
#[derive(Default)]
pub struct DomBridge {
    next_doc_seq: u64,
    next_el_seq: u64,
    doc_order: VecDeque<String>,
    documents: HashMap<String, Html>,
    elements: HashMap<String, StoredElement>,
}

struct StoredElement {
    doc_id: String,
    text: String,
    inner_html: String,
    attrs: HashMap<String, String>,
    outer_html: String,
}

impl DomBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_document(&mut self, html: &str) -> String {
        self.next_doc_seq += 1;
        let doc_id = format!("doc_{}", self.next_doc_seq);
        let parsed = Html::parse_document(html);
        self.documents.insert(doc_id.clone(), parsed);
        self.doc_order.push_back(doc_id.clone());
        self.evict_oldest_if_needed();
        doc_id
    }

    pub fn select(&mut self, doc_id: &str, selector_css: &str) -> Vec<String> {
        let Ok(selector) = Selector::parse(selector_css) else {
            return Vec::new();
        };
        let captured: Vec<StoredElement> = {
            let Some(doc) = self.documents.get(doc_id) else {
                return Vec::new();
            };
            doc.select(&selector)
                .map(|el| capture_element(doc_id, el))
                .collect()
        };
        captured.into_iter().map(|el| self.insert_element(el)).collect()
    }

    pub fn text(&self, element_id: &str) -> String {
        self.elements
            .get(element_id)
            .map(|e| e.text.clone())
            .unwrap_or_default()
    }

    pub fn html(&self, element_id: &str) -> String {
        self.elements
            .get(element_id)
            .map(|e| e.inner_html.clone())
            .unwrap_or_default()
    }

    pub fn outer_html(&self, element_id: &str) -> String {
        self.elements
            .get(element_id)
            .map(|e| e.outer_html.clone())
            .unwrap_or_default()
    }

    pub fn attr(&self, element_id: &str, name: &str) -> Option<String> {
        self.elements
            .get(element_id)
            .and_then(|e| e.attrs.get(name).cloned())
    }

    pub fn next_sibling(&mut self, doc_id: &str, element_id: &str) -> Option<String> {
        self.adjacent_sibling(doc_id, element_id, true)
    }

    pub fn prev_sibling(&mut self, doc_id: &str, element_id: &str) -> Option<String> {
        self.adjacent_sibling(doc_id, element_id, false)
    }

    fn adjacent_sibling(&mut self, doc_id: &str, element_id: &str, forward: bool) -> Option<String> {
        let target_outer = self.elements.get(element_id)?.outer_html.clone();
        let captured = {
            let doc = self.documents.get(doc_id)?;
            let mut found = None;
            for node in doc.tree.nodes() {
                if let Some(el_ref) = ElementRef::wrap(node)
                    && el_ref.html() == target_outer
                {
                    let mut cursor = if forward {
                        el_ref.next_sibling()
                    } else {
                        el_ref.prev_sibling()
                    };
                    while let Some(sib) = cursor {
                        if let Some(sib_el) = ElementRef::wrap(sib) {
                            found = Some(capture_element(doc_id, sib_el));
                            break;
                        }
                        cursor = if forward {
                            sib.next_sibling()
                        } else {
                            sib.prev_sibling()
                        };
                    }
                    break;
                }
            }
            found
        }?;
        Some(self.insert_element(captured))
    }

    pub fn clear(&mut self) {
        self.documents.clear();
        self.elements.clear();
        self.doc_order.clear();
    }

    fn insert_element(&mut self, element: StoredElement) -> String {
        self.next_el_seq += 1;
        let id = format!("{}:el:{}", element.doc_id, self.next_el_seq);
        self.elements.insert(id.clone(), element);
        id
    }

    fn evict_oldest_if_needed(&mut self) {
        while self.doc_order.len() > MAX_CACHED_DOCUMENTS {
            if let Some(evicted) = self.doc_order.pop_front() {
                self.documents.remove(&evicted);
                self.elements.retain(|_, v| v.doc_id != evicted);
            }
        }
    }
}

fn capture_element(doc_id: &str, el: ElementRef<'_>) -> StoredElement {
    let text = el.text().collect::<Vec<_>>().join("").trim().to_string();
    let attrs = el
        .value()
        .attrs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    StoredElement {
        doc_id: doc_id.to_string(),
        text,
        inner_html: el.inner_html(),
        attrs,
        outer_html: el.html(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_selects_and_reads_attrs() {
        let mut dom = DomBridge::new();
        let doc = dom.load_document(
            r#"<html><body><div class="list"><a href="https://stream.example/1.mp4">Watch HD</a><span>1080p</span></div></body></html>"#,
        );
        let links = dom.select(&doc, "div.list a");
        assert_eq!(links.len(), 1);
        assert_eq!(dom.text(&links[0]), "Watch HD");
        assert_eq!(
            dom.attr(&links[0], "href").as_deref(),
            Some("https://stream.example/1.mp4")
        );
        let next = dom.next_sibling(&doc, &links[0]).unwrap();
        assert_eq!(dom.text(&next), "1080p");
        let prev = dom.prev_sibling(&doc, &next).unwrap();
        assert_eq!(dom.text(&prev), "Watch HD");
    }
}
