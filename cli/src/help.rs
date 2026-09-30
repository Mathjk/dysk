use {
    crate::args::*,
    clap::CommandFactory,
    std::io::{
        self,
        Write,
    },
    termimad::{
        FmtText,
        minimad::TextTemplate,
    },
};

static INTRO_TEMPLATE: &str = "
**dysk** displays filesystem information in a pretty table.
Complete documentation at https://dystroy.org/dysk

";

static EXAMPLES_TEMPLATE: &str = "
**Examples:**

${examples
**${example-number})** ${example-title}: `${example-cmd}`
${example-comments}
}
";

static EXAMPLES: &[Example] = &[
    Example::new("Standard overview of your usual disks", "dysk", ""),
    Example::new("List all filesystems", "dysk -a", ""),
    Example::new("Display inodes information", "dysk -c +inodes", ""),
    Example::new(
        "Add columns of your choice",
        "dysk -c label+dev+",
        "You may add columns before, after, or at any other place. \
        You can change the column order too. \
        See https://dystroy.org/dysk/table#columns\n",
    ),
    Example::new("See the disk of the current directory", "dysk .", ""),
    Example::new(
        "Filter for low space",
        "dysk -f 'use > 65% | free < 50G'",
        "",
    ),
    Example::new("Filter to exclude SSD disks", "dysk -f 'disk <> SSD'", ""),
    Example::new(
        "Complex filter",
        "dysk -f '(type=xfs & remote=no) | size > 5T'",
        "",
    ),
    Example::new("Export as JSON", "dysk -j", ""),
    Example::new(
        "Sort by free size",
        "dysk -s free",
        "Add `-desc` to the column name to sort in reverse.",
    ),
];

/// Write the help on `w`.
///
/// `clap_help::Printer::print_help` writes to stdout with `println!` and
/// panics when the pipe is closed. The content-width layout is reproduced
/// here through termimad's writer API so that a broken pipe produces a
/// normal `io::Error` handled by the caller instead of a panic.
///
/// # Errors
/// Returns an `io::Error` if writing to `w` fails
pub fn write<W: Write>(
    w: &mut W,
    ascii: bool,
) -> io::Result<()> {
    let mut printer = clap_help::Printer::new(Args::command())
        .with("introduction", INTRO_TEMPLATE)
        .without("author");
    printer.template_keys_mut().push("examples");
    printer.set_template("examples", EXAMPLES_TEMPLATE);
    if ascii {
        printer.skin_mut().limit_to_ascii();
    }
    for (i, example) in EXAMPLES.iter().enumerate() {
        printer
            .expander_mut()
            .sub("examples")
            .set("example-number", i + 1)
            .set("example-title", example.title)
            .set("example-cmd", example.cmd)
            .set_md("example-comments", example.comments);
    }
    // Mirrors `Printer::print_help_content_width` (clap-help 1.6): its
    // templates map isn't exposed, so the templates configured above are
    // listed again here in rendering order
    let skin = printer.skin_mut().clone();
    let mut width = termimad::terminal_size().0 as usize;
    if let Some(max_width) = printer.max_width {
        width = width.min(max_width);
    }
    let expander = printer.expander_mut();
    let mut texts = [
        clap_help::TEMPLATE_TITLE,
        INTRO_TEMPLATE,
        clap_help::TEMPLATE_USAGE,
        clap_help::TEMPLATE_POSITIONALS,
        clap_help::TEMPLATE_OPTIONS,
        EXAMPLES_TEMPLATE,
    ]
    .iter()
    .map(|template| {
        let template = TextTemplate::from(*template);
        FmtText::from_text(&skin, expander.expand(&template), Some(width))
    })
    .collect::<Vec<_>>();
    let content_width = texts
        .iter()
        .fold(0, |cw, text| cw.max(text.content_width()));
    for text in &mut texts {
        text.set_rendering_width(content_width);
        writeln!(w, "{text}")?;
    }
    Ok(())
}

struct Example {
    title: &'static str,
    cmd: &'static str,
    comments: &'static str,
}

impl Example {
    pub const fn new(
        title: &'static str,
        cmd: &'static str,
        comments: &'static str,
    ) -> Self {
        Self {
            title,
            cmd,
            comments,
        }
    }
}
