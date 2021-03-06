//! Code to lex and tokenize css files

use option::is_none;
use str::from_bytes;
use vec::push;

use pipes::{Port, Chan};

use html::lexer_util::*;

use std::net::url::Url;
use resource::resource_task::{ResourceTask, ProgressMsg, Load};

enum ParserState {
    CssElement,
    CssRelation,
    CssDescription,
    CssAttribute
}

type CssLexer = {
    input_state: InputState,
    mut parser_state: ParserState
};

pub enum Token {
    StartDescription,
    EndDescription,
    Descendant,
    Child,
    Sibling,
    Comma,
    Element(~str),
    Attr(css::values::Attr), 
    Description(~str, ~str),
    Eof
}

trait CssLexerMethods {
    fn parse_css() -> Token;
    fn parse_css_relation(c : u8) -> Token;
    fn parse_css_element(c : u8) -> Token;
    fn parse_css_attribute(c : u8) -> Token;
    fn parse_css_description(c: u8) -> Token;
}

impl CssLexer : CssLexerMethods {
    fn parse_css() -> Token {
        let mut ch: u8;
        match self.input_state.get() {
            CoeChar(c) => ch = c,
            CoeEof => { return Eof; }
        }

        let token = match self.parser_state {
          CssDescription => self.parse_css_description(ch),
          CssAttribute => self.parse_css_attribute(ch),
          CssElement => self.parse_css_element(ch),
          CssRelation => self.parse_css_relation(ch)
        };

        #debug["token=%?", token];
        return move token;
    }

    fn parse_css_relation(c : u8) -> Token {
        self.parser_state = CssElement;

        let token = match c {
          '{' as u8 => { self.parser_state = CssDescription; StartDescription }
          '>' as u8 => { Child }
          '+' as u8 => { Sibling }
          ',' as u8 => { Comma }
          _ => { self.input_state.unget(c); Descendant }
        };

        self.input_state.eat_whitespace();
        
        return move token;
    }

    fn parse_css_element(c : u8) -> Token {
        assert is_none(&self.input_state.lookahead);

        /* Check for special attributes with an implied element,
        or a wildcard which is not a alphabet character.*/
        if c == '.' as u8 || c == '#' as u8 {
            self.parser_state = CssAttribute;
            self.input_state.unget(c);
            return Element(~"*");
        } else if c == '*' as u8 {
            self.parser_state = CssAttribute;
            return Element(~"*");
        }

        self.input_state.unget(c);
        let element = self.input_state.parse_ident();

        self.parser_state = CssAttribute;
        
        return move Element(move element);
    }

    fn parse_css_attribute(c : u8) -> Token {
        let mut ch = c;
        
        /* If we've reached the end of this list of attributes,
        look for the relation to the next element.*/
        if c.is_whitespace() {
            self.parser_state = CssRelation;
            self.input_state.eat_whitespace();

            match self.input_state.get() {
              CoeChar(c) => { ch = c }
              CoeEof => { fail ~"File ended before description of style" }
            }

            return self.parse_css_relation(ch);
        }
        
        match ch {
          '.' as u8 => return Attr(css::values::Includes(~"class", self.input_state.parse_ident())),
          '#' as u8 => return Attr(css::values::Includes(~"id", self.input_state.parse_ident())),
          '[' as u8 => {
            let attr_name = self.input_state.parse_ident();
            
            match self.input_state.get() {
              CoeChar(c) => { ch = c; }
              CoeEof => { fail ~"File ended before description finished"; }
            }

            if ch == ']' as u8 {
                return Attr(css::values::Exists(move attr_name));
            } else if ch == '=' as u8 {
                let attr_val = self.input_state.parse_ident();
                self.input_state.expect(']' as u8);
                return Attr(css::values::Exact(move attr_name, move attr_val));
            } else if ch == '~' as u8 {
                self.input_state.expect('=' as u8);
                let attr_val = self.input_state.parse_ident();
                self.input_state.expect(']' as u8);
                return Attr(css::values::Includes(move attr_name, move attr_val));
            } else if ch == '|' as u8 {
                self.input_state.expect('=' as u8);
                let attr_val = self.input_state.parse_ident();
                self.input_state.expect(']' as u8);
                return Attr(css::values::StartsWith(move attr_name, move attr_val));
            }
            
            fail #fmt("Unexpected symbol %c in attribute", ch as char);
          }
          _ => { fail #fmt("Unexpected symbol %c in attribute", ch as char); }
        }
    }

    fn parse_css_description(c: u8) -> Token {
        let mut ch = c;

        if ch == '}' as u8 {
            self.parser_state = CssElement;
            self.input_state.eat_whitespace();
            return EndDescription;
        } else if ch.is_whitespace() {
            self.input_state.eat_whitespace();

            match self.input_state.get() {
              CoeChar(c) => { ch = c }
              CoeEof => { fail ~"Reached end of file in CSS description" }
            }
        }
        
        let mut desc_name = ~[];
        
        // Get the name of the descriptor
        loop {
            if ch.is_whitespace() {
                self.input_state.eat_whitespace();
            } else if ch == ':' as u8 {
                if desc_name.len() == 0u {
                    fail ~"Expected descriptor name";
                } else {
                    break;
                }
            } else {
                push(&mut desc_name, ch);
            }

            match self.input_state.get() {
              CoeChar(c) => { ch = c }
              CoeEof => { fail ~"Reached end of file in CSS description" }
            }
        }

        self.input_state.eat_whitespace();
        let mut desc_val = ~[];

        // Get the value of the descriptor
        loop {
            match self.input_state.get() {
              CoeChar(c) => { ch = c }
              CoeEof => { fail ~"Reached end of file in CSS description" }
            }

            if ch.is_whitespace() {
                self.input_state.eat_whitespace();
            } else if ch == '}' as u8 {
                if desc_val.len() == 0u {
                    fail ~"Expected descriptor value";
                } else {
                    self.input_state.unget('}' as u8);
                    break;
                }
            } else if ch == ';' as u8 {
                if desc_val.len() == 0u {
                    fail ~"Expected descriptor value";
                } else {
                    break;
                }
            } else {
                push(&mut desc_val, ch);
            }
        }

        return Description(from_bytes(desc_name), from_bytes(desc_val));
    }
}

fn parser(input_port: comm::Port<ProgressMsg>, state : ParserState) -> CssLexer {
    return {
           input_state: {
               mut lookahead: None,
               mut buffer: ~[],
               input_port: input_port,
               mut eof: false
           },
           mut parser_state: state
    };
}

fn lex_css_from_bytes(input_port: comm::Port<ProgressMsg>, result_chan : &Chan<Token>) {
    let lexer = parser(input_port, CssElement);

    loop {
        let token = lexer.parse_css();
        let should_break = match token { Eof => true, _ => false };

        result_chan.send(move token);

        if should_break { 
            break;
        }
    }
}

fn spawn_css_lexer_from_string(content : ~str) -> pipes::Port<Token> {
    let (result_chan, result_port) = pipes::stream();

    do task::spawn |move result_chan, move content| {
        let input_port = comm::Port();
        input_port.send(Payload(str::to_bytes(content)));
        input_port.send(Done(Ok(())));

        lex_css_from_bytes(input_port, &result_chan);
    }

    return move result_port;
}

#[allow(non_implicitly_copyable_typarams)]
pub fn spawn_css_lexer_task(url: Url, resource_task: ResourceTask) -> pipes::Port<Token> {
    let (result_chan, result_port) = pipes::stream();

    do task::spawn |move result_chan, move url| {
        assert url.path.ends_with(".css");
        let input_port = Port();
        // TODO: change copy to move once the compiler permits it
        resource_task.send(Load(copy url, input_port.chan()));

        lex_css_from_bytes(input_port, &result_chan);
    };

    return move result_port;
}
