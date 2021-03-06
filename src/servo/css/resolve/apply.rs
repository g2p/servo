/**
 * Applies the appropriate CSS style to nodes.
*/

use au = gfx::geometry;
use css::styles::SpecifiedStyle;
use dom::node::{Node, NodeTree};
use dom::element::*;
use layout::context::LayoutContext;
use image::ImageHolder;
use resource::image_cache_task::ImageCacheTask;
use std::net::url::Url;

use css::values::*;

trait ResolveMethods<T> {
    pure fn initial() -> T;
}

impl CSSValue<CSSBackgroundColor> : ResolveMethods<CSSBackgroundColor> {
    pure fn initial() -> CSSBackgroundColor { return BgColorTransparent; }
}

impl CSSValue<CSSDisplay> : ResolveMethods<CSSDisplay> {
    pure fn initial() -> CSSDisplay { return DisplayInline; }
}

impl CSSValue<BoxSizing> : ResolveMethods<BoxSizing> {
    pure fn initial() -> BoxSizing { return BoxAuto; }
}

impl CSSValue<CSSFontSize> : ResolveMethods<CSSFontSize> {
    pure fn initial() -> CSSFontSize { return AbsoluteSize(Medium); }
}


struct StyleApplicator {
    node: Node,
}

// TODO: normalize this into a normal preorder tree traversal function
fn apply_style(layout_ctx: &LayoutContext, node: Node) {
    let applicator = StyleApplicator {
        node: node,
    };

    applicator.apply_css_style(layout_ctx);
}

// TODO: this is misleadingly-named. It is actually trying to resolve CSS 'inherit' values.

/** A wrapper around a set of functions that can be applied as a
 * top-down traversal of layout boxes.
 */
fn inheritance_wrapper(layout_ctx: &LayoutContext, node : Node) {
    let applicator = StyleApplicator {
        node: node,
    };
    applicator.resolve_style(layout_ctx);
}

/*
fn resolve_fontsize(box : @RenderBox) {
    // TODO: complete this
    return
}

fn resolve_height(box : @RenderBox) -> au {
    let style = box.node.get_style();
    let inherit_val = match box.tree.parent {
        None => au(0),
        Some(parent) => parent.data.computed_size.height
    };

    box.appearance.height = match style.height {
        Initial => style.height.initial(),
        Inherit => inherit_val,
        Specified(val) => match val { // BoxSizing
            BoxPercent(*) | BoxAuto | BoxLength(Px(_)) => val,
            BoxLength(Em(n)) => BoxLength(Px(n * box.appearance.font_size.abs()))
        }
    }
}

fn resolve_width(box : @RenderBox) {
    let style = box.node.get_specified_style();
    let inherit_val = match box.tree.parent {
        None => style.height.initial(),
        Some(node) => node.appearance.width
    };

    box.appearance.width = match style.width {
        Initial => style.width.initial(),
        Inherit => inherit_val,
        Specified(val) => match val { // BoxSizing
            BoxPercent(*) | BoxAuto | BoxLength(Px(_)) => val,
            BoxLength(Em(n)) => BoxLength(Px(n * box.appearance.font_size.abs()))
        }
    }
}*/

impl StyleApplicator {
    fn apply_css_style(layout_ctx: &LayoutContext) {

        for NodeTree.each_child(&self.node) |child| {
            inheritance_wrapper(layout_ctx, *child)
        }
    }

    /** 
     * Convert the cascaded, specified style for this node into a resolved style:
     * one which additionally resolves the values of Initial, Inherit based on 
     * defaults and node parent style. It also converts Node attributes into 
     * equivalent inline style declarations (TODO: where is this defined??)
     */
    fn resolve_style(_layout_ctx: &LayoutContext) {
        // TODO: implement
    }
}

#[cfg(test)]
mod test {
    /* TODO: rewrite once cascade and resolve written. */
}
