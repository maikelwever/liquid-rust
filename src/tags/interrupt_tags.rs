use std::io::Write;

use liquid_error::Result;

use compiler::Language;
use compiler::ParseTag;
use compiler::TagReflection;
use compiler::TagTokenIter;
use interpreter::Renderable;
use interpreter::{Context, Interrupt};

#[derive(Copy, Clone, Debug)]
struct Break;

impl Renderable for Break {
    fn render_to(&self, _writer: &mut dyn Write, context: &mut Context) -> Result<()> {
        context.interrupt_mut().set_interrupt(Interrupt::Break);
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct BreakTag;

impl BreakTag {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TagReflection for BreakTag {
    fn tag(&self) -> &'static str {
        "break"
    }

    fn description(&self) -> &'static str {
        ""
    }
}

impl ParseTag for BreakTag {
    fn parse(
        &self,
        mut arguments: TagTokenIter,
        _options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        // no arguments should be supplied, trying to supply them is an error
        arguments.expect_nothing()?;
        Ok(Box::new(Break))
    }

    fn reflection(&self) -> &dyn TagReflection {
        self
    }
}

#[derive(Copy, Clone, Debug)]
struct Continue;

impl Renderable for Continue {
    fn render_to(&self, _writer: &mut dyn Write, context: &mut Context) -> Result<()> {
        context.interrupt_mut().set_interrupt(Interrupt::Continue);
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ContinueTag;

impl ContinueTag {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TagReflection for ContinueTag {
    fn tag(&self) -> &'static str {
        "continue"
    }

    fn description(&self) -> &'static str {
        ""
    }
}

impl ParseTag for ContinueTag {
    fn parse(
        &self,
        mut arguments: TagTokenIter,
        _options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        // no arguments should be supplied, trying to supply them is an error
        arguments.expect_nothing()?;
        Ok(Box::new(Continue))
    }

    fn reflection(&self) -> &dyn TagReflection {
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use compiler;
    use interpreter;
    use tags;

    fn options() -> Language {
        let mut options = Language::default();
        options.tags.register("break", BreakTag.into());
        options.tags.register("continue", ContinueTag.into());
        options.blocks.register("for", tags::ForBlock.into());
        options.blocks.register("if", tags::IfBlock.into());
        options
    }

    #[test]
    fn test_simple_break() {
        let text = concat!(
            "{% for i in (0..10) %}",
            "enter-{{i}};",
            "{% if i == 2 %}break-{{i}}\n{% break %}{% endif %}",
            "exit-{{i}}\n",
            "{% endfor %}"
        );
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut ctx = Context::new();
        let output = template.render(&mut ctx).unwrap();
        assert_eq!(
            output,
            concat!("enter-0;exit-0\n", "enter-1;exit-1\n", "enter-2;break-2\n")
        );
    }

    #[test]
    fn test_nested_break() {
        // assert that a {% break %} only breaks out of the innermost loop
        let text = concat!(
            "{% for outer in (0..3) %}",
            "enter-{{outer}}; ",
            "{% for inner in (6..10) %}",
            "{% if inner == 8 %}break, {% break %}{% endif %}",
            "{{ inner }}, ",
            "{% endfor %}",
            "exit-{{outer}}\n",
            "{% endfor %}"
        );
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut ctx = Context::new();
        let output = template.render(&mut ctx).unwrap();
        assert_eq!(
            output,
            concat!(
                "enter-0; 6, 7, break, exit-0\n",
                "enter-1; 6, 7, break, exit-1\n",
                "enter-2; 6, 7, break, exit-2\n",
                "enter-3; 6, 7, break, exit-3\n",
            )
        );
    }

    #[test]
    fn test_simple_continue() {
        let text = concat!(
            "{% for i in (0..5) %}",
            "enter-{{i}};",
            "{% if i == 2 %}continue-{{i}}\n{% continue %}{% endif %}",
            "exit-{{i}}\n",
            "{% endfor %}"
        );
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut ctx = Context::new();
        let output = template.render(&mut ctx).unwrap();
        assert_eq!(
            output,
            concat!(
                "enter-0;exit-0\n",
                "enter-1;exit-1\n",
                "enter-2;continue-2\n",
                "enter-3;exit-3\n",
                "enter-4;exit-4\n",
                "enter-5;exit-5\n",
            )
        );
    }

    #[test]
    fn test_nested_continue() {
        // assert that a {% continue %} only jumps out of the innermost loop
        let text = concat!(
            "{% for outer in (0..3) %}",
            "enter-{{outer}}; ",
            "{% for inner in (6..10) %}",
            "{% if inner == 8 %}continue, {% continue %}{% endif %}",
            "{{ inner }}, ",
            "{% endfor %}",
            "exit-{{outer}}\n",
            "{% endfor %}"
        );
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut ctx = Context::new();
        let output = template.render(&mut ctx).unwrap();
        assert_eq!(
            output,
            concat!(
                "enter-0; 6, 7, continue, 9, 10, exit-0\n",
                "enter-1; 6, 7, continue, 9, 10, exit-1\n",
                "enter-2; 6, 7, continue, 9, 10, exit-2\n",
                "enter-3; 6, 7, continue, 9, 10, exit-3\n",
            )
        );
    }
}
