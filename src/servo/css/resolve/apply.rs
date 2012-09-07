#[doc="Applies the appropriate CSS style to boxes."]

import dom::base::{Element, HTMLImageElement, Node};
import gfx::geometry::au_to_px;
import layout::base::{Box, BTree, NTree, LayoutData, SpecifiedStyle, ImageHolder,
              BlockBox, InlineBox, IntrinsicBox, TextBox};
import layout::traverse::{top_down_traversal};
import std::net::url::Url;
import resource::image_cache_task::ImageCacheTask;

import css::values::{Percent, Mm, Pt, Px, Auto, PtToPx, MmToPx};

struct StyleApplicator {
    box: @Box;
    doc_url: &Url;
    image_cache_task: ImageCacheTask;
    reflow: fn~();
}

fn apply_style(box: @Box, doc_url: &Url, image_cache_task: ImageCacheTask, reflow: fn~()) {
    let applicator = StyleApplicator {
        box: box,
        doc_url: doc_url,
        image_cache_task: image_cache_task,
        reflow: reflow
    };

    applicator.apply_css_style();
}

#[doc="A wrapper around a set of functions that can be applied as a top-down traversal of layout
       boxes."]
fn inheritance_wrapper(box : @Box, doc_url: &Url, image_cache_task: ImageCacheTask, reflow: fn~()) {
    let applicator = StyleApplicator {
        box: box,
        doc_url: doc_url,
        image_cache_task: image_cache_task,
        reflow: reflow
    };
    applicator.apply_style();
    inhereit_height(box);
    inhereit_width(box);
}

#[doc="Compute the specified height of a layout box based on it's css specification and its
       parent's height."]
fn inhereit_height(box : @Box) {
    let style = box.node.get_specified_style();
    
    box.appearance.height = match style.height {
        None =>  Auto,
        Some(h) => match h {
            Auto | Px(*) => h,
            Pt(*) => PtToPx(h),
            Mm(*) => MmToPx(h),
            Percent(em) => {
                match box.tree.parent {
                    None => Auto,
                    Some(parent) => {
                        match parent.appearance.height {
                            //This is a poorly constrained case, so we ignore the percentage
                            Auto => Auto,
                            Px(f) => Px(em*f/100.0),
                            Percent(*) | Mm(*) | Pt(*) => {
                                fail ~"failed inheriting heights, parent should only be Px or Auto"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[doc="Compute the specified width of a layout box based on it's css specification and its
       parent's width."]
fn inhereit_width(box : @Box) {
    let style = box.node.get_specified_style();
    
    box.appearance.width = match style.width {
        None =>  Auto,
        Some(h) => match h {
            Auto | Px(*) => h,
            Pt(*) => PtToPx(h),
            Mm(*) => MmToPx(h),
            Percent(em) => {
                match box.tree.parent {
                    None => Auto,
                    Some(parent) => {
                        match parent.appearance.width {
                            //This is a poorly constrained case, so we ignore the percentage
                            Auto => Auto,
                            Px(f) => Px(em*f/100.0),
                            Percent(*) | Mm(*) | Pt(*) => {
                                fail ~"failed inheriting widths, parent should only be Px or Auto"
                            }
                        }
                    }
                }
            }
        }
    }
}

impl StyleApplicator {
    fn apply_css_style() {
        let doc_url = copy *self.doc_url;
        let image_cache_task = self.image_cache_task;
        let reflow = copy self.reflow;
        do top_down_traversal(self.box) |box, move doc_url| {
            inheritance_wrapper(box, &doc_url, image_cache_task, reflow);
        }
    }

    #[doc="Applies CSS style to a layout box.

      Get the specified style and apply the existing traits to a
      layout box.  If a trait does not exist, calculate the default
      value for the given type of element and use that instead.

     "]
    fn apply_style() {

        // Right now, we only handle images.
        do self.box.node.read |node| {
            match node.kind {
              ~Element(element) => {
                let style = self.box.node.get_specified_style();

                self.box.appearance.background_color = match style.background_color {
                  Some(col) => col,
                  None => node.kind.default_color()
                };

                match element.kind {
                  ~HTMLImageElement(*) => {
                    let url = element.get_attr(~"src");
                    
                    if url.is_some() {
                        // FIXME: Some sort of BASE HREF support!
                        // FIXME: Parse URLs!
                        let new_url = make_url(option::unwrap(url), Some(copy *self.doc_url));
                        self.box.appearance.background_image = Some(ImageHolder(new_url, self.image_cache_task, self.reflow))
                    };
                  }
                  _ => { /* Ignore. */ }
                }
              }
              _ => { /* Ignore. */ }
            }
        }
    }
}

#[cfg(test)]
mod test {
    import dom::base::{Attr, HTMLDivElement, HTMLHeadElement, HTMLImageElement, ElementData};
    import dom::base::{NodeScope, UnknownElement};
    import dvec::DVec;

    #[allow(non_implicitly_copyable_typarams)]
    fn new_node(scope: NodeScope, -name: ~str) -> Node {
        let elmt = ElementData(name, ~HTMLDivElement);
        return scope.new_node(dom::base::Element(elmt));
    }

    #[test]
    fn test_percent_height() {
        let scope = NodeScope();

        let parent = new_node(scope, ~"parent");
        let child = new_node(scope, ~"child");
        let child2 = new_node(scope, ~"child");
        let g1 = new_node(scope, ~"gchild");
        let g2 = new_node(scope, ~"gchild");

        scope.add_child(parent, child);
        scope.add_child(parent, child2);
        scope.add_child(child, g1);
        scope.add_child(child, g2);
        let _handles = parent.initialize_style_for_subtree();

        do parent.aux |aux| { aux.specified_style.height = Some(Px(100.0)); }
        do child.aux |aux| { aux.specified_style.height = Some(Auto); }
        do child2.aux |aux| { aux.specified_style.height = Some(Percent(50.0)); }
        do g1.aux |aux| { aux.specified_style.height = Some(Percent(50.0)); }
        do g2.aux |aux| { aux.specified_style.height = Some(Px(10.0)); }

        let parent_box = parent.construct_boxes();
        let child_box = parent_box.get().tree.first_child.get();
        let child2_box = parent_box.get().tree.last_child.get();
        let g1_box = child_box.tree.first_child.get();
        let g2_box = child_box.tree.last_child.get();
        
        top_down_traversal(parent_box.get(), inhereit_height);

        assert parent_box.get().appearance.height == Px(100.0);
        assert child_box.appearance.height == Auto;
        assert child2_box.appearance.height == Px(50.0);
        assert g1_box.appearance.height == Auto;
        assert g2_box.appearance.height == Px(10.0);
    }
}