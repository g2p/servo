use au = gfx::geometry;
use au::au;
use dvec::DVec;
use geom::size::Size2D;
use std::net::url::Url;

struct ElementData {
    tag_name: ~str,
    kind: ~ElementKind,
    attrs: DVec<~Attr>,
}

#[allow(non_implicitly_copyable_typarams)]
impl ElementData {
    fn get_attr(name: &str) -> Option<~str> {
        let found = do self.attrs.find |attr| { name == attr.name };
        match found {
            Some(attr) => Some(copy attr.value),
            None => None
        }
    }

    fn set_attr(name: &str, value: ~str) {
        let idx = do self.attrs.position |attr| { name == attr.name };
        match idx {
            Some(idx) => self.attrs.set_elt(idx, ~Attr(name.to_str(), move value)),
            None => {}
        }
    }
}

fn ElementData(tag_name: ~str, kind: ~ElementKind) -> ElementData {
    ElementData {
        tag_name : move tag_name,
        kind : move kind,
        attrs : DVec(),
    }
}

struct Attr {
    name: ~str,
    value: ~str,
}

fn Attr(name: ~str, value: ~str) -> Attr {
    Attr {
        name : move name,
        value : move value,
    }
}

fn HTMLImageData() -> HTMLImageData {
    HTMLImageData {
        image: None
    }
}

struct HTMLImageData {
    mut image: Option<Url>
}

enum HeadingLevel {
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
}

enum ElementKind {
    HTMLAnchorElement,
    HTMLAsideElement,
    HTMLBRElement,
    HTMLBodyElement,
    HTMLBoldElement,
    HTMLDivElement,
    HTMLFontElement,
    HTMLFormElement,
    HTMLHRElement,
    HTMLHeadElement,
    HTMLHeadingElement(HeadingLevel),
    HTMLHtmlElement,
    HTMLImageElement(HTMLImageData),
    HTMLInputElement,
    HTMLItalicElement,
    HTMLLinkElement,
    HTMLListItemElement,
    HTMLMetaElement,
    HTMLOListElement,
    HTMLOptionElement,
    HTMLParagraphElement,
    HTMLScriptElement,
    HTMLSectionElement,
    HTMLSelectElement,
    HTMLSmallElement,
    HTMLSpanElement,
    HTMLStyleElement,
    HTMLTableBodyElement,
    HTMLTableCellElement,
    HTMLTableElement,
    HTMLTableRowElement,
    HTMLTitleElement,
    HTMLUListElement,
    UnknownElement,
}
