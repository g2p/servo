/**
   Performs CSS selector matching.
*/

use dom::node::{LayoutData, Node, Text};
use dom::element::ElementData;

use values::*;
use styles::{SpecifiedStyle};

/** 
   Check if a CSS attribute matches the attribute of an HTML element.
*/
fn attrs_match(attr: &Attr, elmt: &ElementData) -> bool {
    match *attr {
      Exists(name) => {
        match elmt.get_attr(name) {
          Some(_) => true,
          None => false
        }
      }
      Exact(name, val) => {
        match elmt.get_attr(name) {
          Some(value) => value == val,
          None => false
        }
      }
      Includes(name, val) => {
        // Comply with css spec, if the specified attribute is empty
        // it cannot match.
        if val == ~"" { return false; }

        match elmt.get_attr(name) {
          Some(value) => value.split_char(' ').contains(&val),
          None => false
        }
      }
      StartsWith(name, val) => {
        match elmt.get_attr(name) {
          Some(value) => { 
            //check that there is only one attribute value and it
            //starts with the perscribed value
            if !value.starts_with(val) || value.contains(~" ") { return false; }

            // We match on either the exact value or value-foo
            if value.len() == val.len() { true }
            else { value.starts_with(val + ~"-") }
          }
          None => {
            false
          }
        }
      }
    }
}

trait PrivMatchingMethods {
    fn matches_element(sel: &Selector) -> bool;
    fn matches_selector(sel: &Selector) -> bool;
}

impl Node : PrivMatchingMethods { 

    /** 
    Checks if the given CSS selector, which must describe a single
    element with no relational information, describes the given HTML
    element.
    */
    fn matches_element(sel: &Selector) -> bool {
        match *sel {
          Child(_, _) | Descendant(_, _) | Sibling(_, _) => { return false; }
          Element(tag, attrs) => {
            match self.read(|n| copy *n.kind) {
              dom::node::Element(elmt) => {
                if !(tag == ~"*" || tag == elmt.tag_name) {
                    return false;
                }
                
                let mut i = 0u;
                while i < attrs.len() {
                    if !attrs_match(&attrs[i], &elmt) { return false; }
                    i += 1u;
                }

                return true;
              }
              _ => { /*fall through, currently unsupported*/ }
            }
          }
        }

        return false; //If we got this far it was because something was
                   //unsupported.
    }

    /**
    Checks if a generic CSS selector matches a given HTML element
    */
    fn matches_selector(sel : &Selector) -> bool {
        match *sel {
          Element(*) => { return self.matches_element(sel); }
          Child(sel1, sel2) => {
            return match self.read(|n| n.tree.parent) {
              Some(parent) => self.matches_element(sel2) && parent.matches_selector(sel1),
              None => false
            }
          }
          Descendant(sel1, sel2) => {
            if !self.matches_element(sel2) { return false; }

            //loop over all ancestors to check if they are the person
            //we should be descended from.
            let mut cur_parent = match self.read(|n| n.tree.parent) {
              Some(parent) => parent,
              None => return false
            };

            loop {
                if cur_parent.matches_selector(sel1) { return true; }

                cur_parent = match cur_parent.read(|n| n.tree.parent) {
                  Some(parent) => parent,
                  None => return false
                };
            }
          }
          Sibling(sel1, sel2) => {
            if !self.matches_element(sel2) { return false; }

            // Loop over this node's previous siblings to see if they match.
            match self.read(|n| n.tree.prev_sibling) {
              Some(sib) => {
                let mut cur_sib = sib;
                loop {
                    if cur_sib.matches_selector(sel1) { return true; }
                    
                    cur_sib = match cur_sib.read(|n| n.tree.prev_sibling) {
                      Some(sib) => sib,
                      None => { break; }
                    };
                }
              }
              None => { }
            }

            // check the rest of the siblings
            match self.read(|n| n.tree.next_sibling) {
                Some(sib) => {
                    let mut cur_sib = sib;
                    loop {
                        if cur_sib.matches_selector(sel1) { return true; }
                
                        cur_sib = match cur_sib.read(|n| n.tree.next_sibling) {
                            Some(sib) => sib,
                            None => { break; }
                        };
                    }
                }
                None => { }
            }

            return false;
          }
        }
    }
}

trait PrivStyleMethods {
    fn update_style(decl : StyleDeclaration);
}

impl Node : PrivStyleMethods {
    /**
    Update the computed style of an HTML element with a style specified by CSS.
    */
    fn update_style(decl : StyleDeclaration) {
        self.aux(|layout| {
            match decl {
              BackgroundColor(col) => layout.style.background_color = col,
              Display(dis) => layout.style.display_type = dis,
              FontSize(size) => layout.style.font_size = size,
              Height(size) => layout.style.height = size,
              Color(col) => layout.style.text_color = col,
              Width(size) => layout.style.width = size,
              BorderColor(col) => layout.style.border_color = col,
              BorderWidth(size) => layout.style.border_width = size,
              Position(pos) => layout.style.position = pos,
              Top(pos) => layout.style.top = pos,
              Right(pos) => layout.style.right = pos,
              Bottom(pos) => layout.style.bottom = pos,
              Left(pos) => layout.style.left = pos,
            };
        })
    }
}

trait MatchingMethods {
    fn match_css_style(styles : &Stylesheet);
}

impl Node : MatchingMethods {
    /**
    Compare an html element to a list of css rules and update its
    style according to the rules matching it.
    */
    fn match_css_style(styles : &Stylesheet) {
        // Loop over each rule, see if our node matches what is
        // described in the rule. If it matches, update its style. As
        // we don't currently have priorities of style information,
        // the latest rule takes precedence over the others. So we
        // just overwrite style information as we go.

        for styles.each |sty| {
            let (selectors, decls) = copy **sty;
            for selectors.each |sel| {
                if self.matches_selector(*sel) {
                    for decls.each |decl| {
                        self.update_style(*decl);
                    }
                }
            }
        }
        
        self.aux(|a| debug!("Changed the style to: %?", copy *a.style));
    }
}

#[cfg(test)]
mod test {
    use dom::element::{Attr, HTMLDivElement, HTMLHeadElement, HTMLImageElement, UnknownElement};
    use dom::node::NodeScope;
    use dvec::DVec;

    #[allow(non_implicitly_copyable_typarams)]
    fn new_node_from_attr(scope: &NodeScope, name: ~str, val: ~str) -> Node {
        let elmt = ElementData(~"div", ~HTMLDivElement);
        let attr = ~Attr(move name, move val);
        elmt.attrs.push(move attr);
        return scope.new_node(dom::node::Element(move elmt));
    }

    #[test]
    fn test_match_pipe1() {
        let scope = NodeScope();
        let node = new_node_from_attr(&scope, ~"lang", ~"en-us");

        let sel = Element(~"*", ~[StartsWith(~"lang", ~"en")]);

        assert node.matches_selector(~move sel);
    }

    #[test]
    fn test_match_pipe2() {
        let scope = NodeScope();
        let node = new_node_from_attr(&scope, ~"lang", ~"en");

        let sel = Element(~"*", ~[StartsWith(~"lang", ~"en")]);

        assert node.matches_selector(~move sel);
    }
    
    #[test] 
    fn test_not_match_pipe() {
        let scope = NodeScope();
        let node = new_node_from_attr(&scope, ~"lang", ~"english");

        let sel = Element(~"*", ~[StartsWith(~"lang", ~"en")]);

        assert !node.matches_selector(~move sel);
    }

    #[test]
    fn test_match_includes() {
        let scope = NodeScope();
        let node = new_node_from_attr(&scope, ~"mad", ~"hatter cobler cooper");

        let sel = Element(~"div", ~[Includes(~"mad", ~"hatter")]);

        assert node.matches_selector(~move sel);
    }

    #[test]
    fn test_match_exists() {
        let scope = NodeScope();
        let node = new_node_from_attr(&scope, ~"mad", ~"hatter cobler cooper");

        let sel1 = Element(~"div", ~[Exists(~"mad")]);
        let sel2 = Element(~"div", ~[Exists(~"hatter")]);

        assert node.matches_selector(~move sel1);
        assert !node.matches_selector(~move sel2);
    }

    #[test]
    fn test_match_exact() {
        let scope = NodeScope();
        let node1 = new_node_from_attr(&scope, ~"mad", ~"hatter cobler cooper");
        let node2 = new_node_from_attr(&scope, ~"mad", ~"hatter");

        let sel = Element(~"div", ~[Exact(~"mad", ~"hatter")]);

        assert !node1.matches_selector(~copy sel);
        assert node2.matches_selector(~move sel);
    }

    #[test]
    fn match_tree() {
        let scope = NodeScope();

        let root = new_node_from_attr(&scope, ~"class", ~"blue");
        let child1 = new_node_from_attr(&scope, ~"id", ~"green");
        let child2 = new_node_from_attr(&scope, ~"flag", ~"black");
        let gchild = new_node_from_attr(&scope, ~"flag", ~"grey");
        let ggchild = new_node_from_attr(&scope, ~"flag", ~"white");
        let gggchild = new_node_from_attr(&scope, ~"flag", ~"purple");

        scope.add_child(root, child1);
        scope.add_child(root, child2);
        scope.add_child(child2, gchild);
        scope.add_child(gchild, ggchild);
        scope.add_child(ggchild, gggchild);

        let sel1 = Descendant(~Element(~"*", ~[Exact(~"class", ~"blue")]), ~Element(~"*", ~[]));

        assert !root.matches_selector(~copy sel1);
        assert child1.matches_selector(~copy sel1);
        assert child2.matches_selector(~copy sel1);
        assert gchild.matches_selector(~copy sel1);
        assert ggchild.matches_selector(~copy sel1);
        assert gggchild.matches_selector(~move sel1);

        let sel2 = Descendant(~Child(~Element(~"*", ~[Exact(~"class", ~"blue")]),
                                     ~Element(~"*", ~[])),
                              ~Element(~"div", ~[Exists(~"flag")]));

        assert !root.matches_selector(~copy sel2);
        assert !child1.matches_selector(~copy sel2);
        assert !child2.matches_selector(~copy sel2);
        assert gchild.matches_selector(~copy sel2);
        assert ggchild.matches_selector(~copy sel2);
        assert gggchild.matches_selector(~move sel2);

        let sel3 = Sibling(~Element(~"*", ~[]), ~Element(~"*", ~[]));

        assert !root.matches_selector(~copy sel3);
        assert child1.matches_selector(~copy sel3);
        assert child2.matches_selector(~copy sel3);
        assert !gchild.matches_selector(~copy sel3);
        assert !ggchild.matches_selector(~copy sel3);
        assert !gggchild.matches_selector(~move sel3);

        let sel4 = Descendant(~Child(~Element(~"*", ~[Exists(~"class")]), ~Element(~"*", ~[])),
                              ~Element(~"*", ~[]));

        assert !root.matches_selector(~copy sel4);
        assert !child1.matches_selector(~copy sel4);
        assert !child2.matches_selector(~copy sel4);
        assert gchild.matches_selector(~copy sel4);
        assert ggchild.matches_selector(~copy sel4);
        assert gggchild.matches_selector(~move sel4);
    }
}
