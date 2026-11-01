// SPDX-License-Identifier: AGPL-3.0-only
//! Dependency-free, deterministic SVG writer.
//!
//! The writer escapes text and attribute values, formats numbers with at most
//! two decimals (trailing zeros trimmed, `-0` normalised to `0`) and tracks
//! open elements so [`SvgWriter::finish`] can reject unbalanced output. The
//! same inputs always produce byte-identical output.

use std::fmt::Write as _;

/// Format a number for SVG output: at most 2 decimals, no trailing zeros.
#[must_use]
pub fn num(v: f64) -> String {
    if !v.is_finite() {
        return "0".to_owned();
    }
    let mut s = format!("{:.2}", (v * 100.0).round() / 100.0);
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" {
        s = "0".to_owned();
    }
    s
}

/// Escape text content / attribute values.
pub fn escape_into(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
}

/// Value that can be written as an attribute.
pub trait AttrValue {
    fn write_attr(&self, out: &mut String);
}

impl AttrValue for &str {
    fn write_attr(&self, out: &mut String) {
        escape_into(out, self);
    }
}

impl AttrValue for String {
    fn write_attr(&self, out: &mut String) {
        escape_into(out, self);
    }
}

impl AttrValue for f64 {
    fn write_attr(&self, out: &mut String) {
        out.push_str(&num(*self));
    }
}

impl AttrValue for u32 {
    fn write_attr(&self, out: &mut String) {
        let _ = write!(out, "{self}");
    }
}

impl AttrValue for usize {
    fn write_attr(&self, out: &mut String) {
        let _ = write!(out, "{self}");
    }
}

impl AttrValue for i32 {
    fn write_attr(&self, out: &mut String) {
        let _ = write!(out, "{self}");
    }
}

/// Error returned by [`SvgWriter::finish`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SvgError {
    /// Elements still open at finish.
    Unclosed(Vec<&'static str>),
}

impl std::fmt::Display for SvgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unclosed(tags) => write!(f, "unclosed SVG elements: {}", tags.join(", ")),
        }
    }
}

impl std::error::Error for SvgError {}

/// Streaming SVG writer.
#[derive(Debug, Default)]
pub struct SvgWriter {
    buf: String,
    stack: Vec<&'static str>,
}

impl SvgWriter {
    /// Empty writer (for fragments).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Writer with an open root `<svg>` sized `width`×`height`.
    #[must_use]
    pub fn document(width: f64, height: f64) -> Self {
        let mut w = Self::new();
        w.tag("svg")
            .attr("xmlns", "http://www.w3.org/2000/svg")
            .attr("viewBox", format!("0 0 {} {}", num(width), num(height)))
            .attr("width", width)
            .attr("height", height)
            .open();
        w
    }

    /// Start an element; finish it with [`Tag::end`], [`Tag::open`] or [`Tag::text`].
    pub fn tag(&mut self, name: &'static str) -> Tag<'_> {
        self.buf.push('<');
        self.buf.push_str(name);
        Tag { w: self, name }
    }

    /// Close the innermost open element.
    ///
    /// # Panics
    /// Panics if no element is open (a programming error in a generator).
    pub fn close(&mut self) {
        let name = self
            .stack
            .pop()
            .expect("SvgWriter::close with no open element");
        self.buf.push_str("</");
        self.buf.push_str(name);
        self.buf.push('>');
    }

    /// Append escaped character data.
    pub fn text(&mut self, s: &str) {
        escape_into(&mut self.buf, s);
    }

    /// Append an XML comment (`--` is replaced so the comment stays valid).
    pub fn comment(&mut self, s: &str) {
        self.buf.push_str("<!-- ");
        self.buf.push_str(&s.replace("--", "- -"));
        self.buf.push_str(" -->");
    }

    /// Number of currently open elements.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Output so far (may be unbalanced).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.buf
    }

    /// Close every open element and return the document.
    #[must_use]
    pub fn finish_closing(mut self) -> String {
        while !self.stack.is_empty() {
            self.close();
        }
        self.buf
    }

    /// Return the output, failing if elements are still open.
    ///
    /// # Errors
    /// [`SvgError::Unclosed`] when the element stack is not empty.
    pub fn finish(self) -> Result<String, SvgError> {
        if self.stack.is_empty() {
            Ok(self.buf)
        } else {
            Err(SvgError::Unclosed(self.stack))
        }
    }
}

/// Element under construction.
#[must_use = "finish the element with end(), open() or text()"]
pub struct Tag<'a> {
    w: &'a mut SvgWriter,
    name: &'static str,
}

impl Tag<'_> {
    /// Add an attribute.
    pub fn attr(self, key: &str, value: impl AttrValue) -> Self {
        self.w.buf.push(' ');
        self.w.buf.push_str(key);
        self.w.buf.push_str("=\"");
        value.write_attr(&mut self.w.buf);
        self.w.buf.push('"');
        self
    }

    /// Add an attribute only when `value` is `Some`.
    pub fn attr_opt(self, key: &str, value: Option<impl AttrValue>) -> Self {
        match value {
            Some(v) => self.attr(key, v),
            None => self,
        }
    }

    /// Self-closing element.
    pub fn end(self) {
        self.w.buf.push_str("/>");
    }

    /// Open element; close it later with [`SvgWriter::close`].
    pub fn open(self) {
        self.w.buf.push('>');
        self.w.stack.push(self.name);
    }

    /// Element with escaped text content.
    pub fn text(self, content: &str) {
        self.w.buf.push('>');
        escape_into(&mut self.w.buf, content);
        self.w.buf.push_str("</");
        self.w.buf.push_str(self.name);
        self.w.buf.push('>');
    }
}

/// Path data builder with deterministic number formatting.
#[derive(Debug, Default, Clone)]
pub struct PathData(String);

impl PathData {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn cmd(mut self, c: char, vals: &[f64]) -> Self {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push(c);
        for (i, v) in vals.iter().enumerate() {
            if i > 0 {
                self.0.push(' ');
            }
            self.0.push_str(&num(*v));
        }
        self
    }

    #[must_use]
    pub fn m(self, x: f64, y: f64) -> Self {
        self.cmd('M', &[x, y])
    }
    #[must_use]
    pub fn l(self, x: f64, y: f64) -> Self {
        self.cmd('L', &[x, y])
    }
    #[must_use]
    pub fn h(self, x: f64) -> Self {
        self.cmd('H', &[x])
    }
    #[must_use]
    pub fn v(self, y: f64) -> Self {
        self.cmd('V', &[y])
    }
    #[must_use]
    pub fn q(self, x1: f64, y1: f64, x: f64, y: f64) -> Self {
        self.cmd('Q', &[x1, y1, x, y])
    }
    #[must_use]
    pub fn c(self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) -> Self {
        self.cmd('C', &[x1, y1, x2, y2, x, y])
    }
    /// Elliptical arc with flags.
    #[must_use]
    pub fn a(self, r: f64, large: bool, sweep: bool, x: f64, y: f64) -> Self {
        self.cmd(
            'A',
            &[
                r,
                r,
                0.0,
                f64::from(u8::from(large)),
                f64::from(u8::from(sweep)),
                x,
                y,
            ],
        )
    }
    #[must_use]
    pub fn z(mut self) -> Self {
        self.0.push_str(" Z");
        self
    }
    #[must_use]
    pub fn build(self) -> String {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_formatting() {
        assert_eq!(num(1.0), "1");
        assert_eq!(num(1.5), "1.5");
        assert_eq!(num(1.256), "1.26");
        assert_eq!(num(-0.001), "0");
        assert_eq!(num(-2.10), "-2.1");
        assert_eq!(num(f64::NAN), "0");
        assert_eq!(num(1512.0), "1512");
    }

    #[test]
    fn escaping() {
        let mut w = SvgWriter::new();
        w.tag("text").attr("data-x", "a\"b<c").text("R&D <ok> 'q'");
        assert_eq!(
            w.finish().expect("balanced"),
            "<text data-x=\"a&quot;b&lt;c\">R&amp;D &lt;ok&gt; &#39;q&#39;</text>"
        );
    }

    #[test]
    fn unbalanced_is_rejected() {
        let mut w = SvgWriter::document(10.0, 10.0);
        w.tag("g").open();
        assert_eq!(w.depth(), 2);
        assert_eq!(w.finish(), Err(SvgError::Unclosed(vec!["svg", "g"])));
        let mut w = SvgWriter::document(10.0, 10.0);
        w.tag("g").open();
        let s = w.finish_closing();
        assert!(s.ends_with("</g></svg>"));
    }

    #[test]
    fn path_builder() {
        let d = PathData::new()
            .m(0.0, 1.5)
            .l(2.0, 3.0)
            .h(4.0)
            .v(5.0)
            .q(1.0, 2.0, 3.0, 4.0)
            .a(2.0, false, true, 1.0, 1.0)
            .z()
            .build();
        assert_eq!(d, "M0 1.5 L2 3 H4 V5 Q1 2 3 4 A2 2 0 0 1 1 1 Z");
    }

    #[test]
    fn comment_stays_valid() {
        let mut w = SvgWriter::new();
        w.comment("a -- b");
        assert_eq!(w.as_str(), "<!-- a - - b -->");
    }
}
