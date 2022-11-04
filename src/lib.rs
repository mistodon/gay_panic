//! A Rust panic handler, but make it gay.

use owo_colors::{colors, AnsiColors, OwoColorize, Rgb};
use sashimi::{LineBasedRules, Parser};

use std::{
    backtrace::*,
    collections::HashMap,
    panic::{self, PanicInfo},
    path::PathBuf,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn parse_int(parser: &mut Parser<LineBasedRules>) -> Result<usize> {
    let n = parser.skip_matching(|ch| b"0123456789".contains(&ch));
    let s = std::str::from_utf8(n)?;
    Ok(s.parse()?)
}

fn show_info(info: &PanicInfo) -> Result<()> {
    let thread = std::thread::current()
        .name()
        .map(str::to_owned)
        .unwrap_or_else(|| "unknown".to_owned());

    let message = info.payload().downcast_ref::<&str>();
    let loc = info.location().unwrap();
    let at_sep = if message.is_some() { " at " } else { "" };
    eprintln!(
        "\nthread '{thread}' panicked{at_sep}'{message}', {fname}:{line}:{col}:\n",
        thread = thread.fg::<colors::BrightRed>(),
        at_sep = at_sep,
        message = message.unwrap_or(&"").fg::<colors::BrightGreen>(),
        fname = loc.file().fg::<colors::BrightCyan>(),
        line = loc.line().fg::<colors::BrightCyan>(),
        col = loc.column().fg::<colors::Yellow>(),
    );

    Ok(())
}

fn show_backtrace(info: &PanicInfo, crate_name: String, backtrace: &Backtrace) -> Result<()> {
    show_info(info)?;

    let source = backtrace.to_string();
    if source.starts_with("disabled backtrace") {
        eprintln!("{}", source.fg::<colors::BrightMagenta>());
        return Ok(());
    }

    let frame_count = source.lines().count() / 2;

    let parser = &mut Parser::<LineBasedRules>::new(&source);
    parser.skip_whitespace();

    let crate_colors = [
        AnsiColors::BrightGreen,
        AnsiColors::BrightCyan,
        AnsiColors::Cyan,
        AnsiColors::BrightBlue,
        AnsiColors::BrightMagenta,
    ];
    let mut crate_count = 0;
    let mut crate_colormap = HashMap::new();
    crate_colormap.insert("std".to_owned(), AnsiColors::BrightRed);
    crate_colormap.insert("rust_begin_unwind".to_owned(), AnsiColors::BrightRed);
    crate_colormap.insert("core".to_owned(), AnsiColors::Blue);
    crate_colormap.insert(crate_name, AnsiColors::BrightYellow);

    while !parser.finished() {
        let frame_number = parse_int(parser)?;
        parser.expect(b":")?;
        let callpath = parse_path(parser)?;

        let frame_col = {
            use color_space::{Hsv, Rgb};

            let hue = frame_number as f64 / frame_count as f64;
            let frame_col = Hsv::new(hue * 360., 0.9, 1.);
            Rgb::from(frame_col)
        };

        let frame_col = Rgb(frame_col.r as u8, frame_col.g as u8, frame_col.b as u8);

        if parser.finished() {
            eprintln!(
                "  {frame_number: >2}. {crate_name}",
                frame_number = frame_number.color(frame_col),
                crate_name = callpath.first.color(frame_col),
            );
            break;
        }

        parser.expect(b"at")?;

        let filename =
            unsafe { std::str::from_utf8_unchecked(parser.skip_matching(|ch| ch != b':')) };
        let path = PathBuf::from(&filename);
        let components = path.components().collect::<Vec<_>>();
        let fname = components
            .last()
            .unwrap()
            .as_os_str()
            .to_string_lossy()
            .into_owned();
        let modname = if components.len() > 1 && fname == "mod.rs" {
            components.get(components.len() - 2)
        } else {
            None
        };
        let fsep = if modname.is_some() { "/" } else { "" };
        let modname = modname
            .map(|x| x.as_os_str().to_string_lossy().into_owned())
            .unwrap_or_default();

        parser.expect(b":")?;
        let line_num = parse_int(parser)?;
        parser.expect(b":")?;
        let col_num = parse_int(parser)?;

        let [crate_name, sep1, scope, sep2, fn_name] = callpath.parts();

        let crate_col = crate_colormap
            .entry(crate_name.to_owned())
            .or_insert_with(|| {
                let col = crate_colors[crate_count];
                crate_count = (crate_count + 1) % crate_colors.len();
                col
            });

        eprintln!("  {frame_number: >2}. {crate_name}{sep1}{scope}{sep2}{fn_name}\n        at {modname}{fsep}{fname}:{line}:{col}\n",
            frame_number = frame_number.color(frame_col),
            crate_name = crate_name.color(*crate_col),
            sep1 = sep1,
            scope = scope.fg::<colors::BrightGreen>(),
            sep2 = sep2,
            fn_name = fn_name.fg::<colors::BrightGreen>(),
            modname = modname.fg::<colors::Yellow>(),
            fsep = fsep.fg::<colors::Yellow>(),
            fname = fname.fg::<colors::BrightCyan>(),
            line = line_num.fg::<colors::BrightCyan>(),
            col = col_num.fg::<colors::Yellow>(),
        );

        parser.expect(b"\n")?;
    }

    Ok(())
}

struct CallPath {
    first: String,
    scope: Option<String>,
    last: Option<String>,
}

impl CallPath {
    fn from_components(components: Vec<String>) -> Self {
        let first = components.first().unwrap().to_owned();
        let mut scope = None;
        let mut last = None;

        let mut weird_fn_name = false;

        if components.len() > 1 {
            let name = components.last().unwrap().to_owned();
            weird_fn_name = name.starts_with('{') || name.starts_with('<');
            last = Some(name);
        }

        if components.len() > 2 {
            let name = components.get(components.len() - 2).unwrap().to_owned();
            let in_struct = name.chars().next().unwrap().is_uppercase();

            if in_struct || weird_fn_name {
                scope = Some(name);
            }
        }

        CallPath { first, scope, last }
    }

    fn parts(&self) -> [&str; 5] {
        [
            &self.first,
            if self.scope.is_some() || self.last.is_some() {
                " :: "
            } else {
                ""
            },
            self.scope.as_deref().unwrap_or(""),
            if self.scope.is_some() { "::" } else { "" },
            self.last.as_deref().unwrap_or(""),
        ]
    }
}

fn parse_path(parser: &mut Parser<LineBasedRules>) -> Result<CallPath> {
    fn s(b: &[u8]) -> String {
        String::from_utf8_lossy(b).into_owned()
    }

    let mut parts = vec![];
    let mut first = true;
    while first || parser.skip(b"::").is_some() {
        if let Some(bytes) = parser.skip(b"{{closure}}") {
            parts.push(s(bytes));
        } else if parser.check(b"<").is_some() {
            parts.push(s(parser.skip_around(b'<')?));
        } else {
            let mut ident = s(parser.expect_ident()?);
            if parser.check(b"<").is_some() {
                let generics = parser.skip_around(b'<')?;
                ident.push_str(&s(generics));
            }
            if parser.check(b"{").is_some() {
                let shim = parser.skip_around(b'{')?;
                ident.push_str(&s(shim));
            }
            parts.push(ident);
        }

        first = false;
    }

    parser.expect(b"\n")?;

    Ok(CallPath::from_components(parts))
}

/// Configuration for the panic handler.
///
/// # Examples
/// ```rust
/// use gay_panic::Config;
///
/// assert_eq!(
///     Config::default(),
///     Config {
///         call_previous_hook: false,
///         force_capture_backtrace: false,
///     }
/// );
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Config {
    /// If `true`, the existing/default panic handler will be called after `gay_panic`'s
    /// executes.
    pub call_previous_hook: bool,

    /// If `true`, a backtrace will be displayed regardless of environment variables set.
    pub force_capture_backtrace: bool,
}

/// Replaces the current panic hook with a more colorful one. Allows some configuration.
///
/// # Examples
///
/// ```rust
/// use gay_panic::Config;
///
/// gay_panic::init_with(Config {
///     call_previous_hook: false,
///     force_capture_backtrace: true,
/// });
/// ```
pub fn init_with(config: Config) {
    let crate_name = PathBuf::from(module_path!())
        .iter()
        .next()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let backtrace = if config.force_capture_backtrace {
            Backtrace::force_capture()
        } else {
            Backtrace::capture()
        };
        match show_backtrace(info, crate_name.clone(), &backtrace) {
            Ok(()) => (),
            Err(e) => eprintln!("Error formatting backtrace: {e}"),
        }

        if config.call_previous_hook {
            hook(info);
        }
    }));
}

/// Replaces the current panic hook with a more colorful one. Uses default `Config`.
///
/// # Examples
///
/// ```rust
/// gay_panic::init();
/// ```
pub fn init() {
    init_with(Config::default());
}
