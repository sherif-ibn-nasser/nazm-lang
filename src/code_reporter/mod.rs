use std::{collections::HashMap, marker::PhantomData};

use owo_colors::{AnsiColors::Black, DynColors, Style, };

use crate::span::Span;

/**
 * let mutable x = 5555 && false;
 *     ^^^     -    ^      ^^^^^ Label 0  (This is the main marker line and it has a label if there is more than one in the code line)
 * ____|_______|____|                     (This is the sub main marker line, and it has no labels)
 * ____|_______|    Label 1               (This is a sub marker line, and it has a label if )
 *     |       |    Label 1               (This is a sub marker line, and it has a label if )
 *     |       Label 2
 *     Label 3
 */

struct CodeReporter<'a> {
    /// Map lines indecies and main depth line on them
    lines_to_report: HashMap<usize, HorizontalMarkerLine<'a, MainDepth>>,
    /// Lines to read from
    files_lines: &'a [&'a str],
}

impl<'a> CodeReporter<'a> {

    fn new(files_lines: &'a [&'a str]) -> Self {
        Self {
            lines_to_report: HashMap::new(),
            files_lines: files_lines,
        }
    }

    fn report(mut self, span: Span, sign: char, style: Style, labels: &'a [&'a str]) -> Self {

        let start_line = span.start.line;
        let start_col = span.start.col;
        let end_line = span.end.line;
        let end_col = span.end.col;

        if start_line == end_line {

            self.lines_to_report.entry(start_line)
                .or_insert( HorizontalMarkerLine::default() )
                .mark_in_line(start_col, end_col, sign, style, labels);

            return self;
        }


        self.lines_to_report.entry(start_line)
            .or_insert( HorizontalMarkerLine::default() )
            .mark_as_multi_line_start(start_col, sign, style);

        self.lines_to_report.entry(end_line)
            .or_insert( HorizontalMarkerLine::default() )
            .mark_as_multi_line_end(end_col, sign, style, labels);


        return self;
    }
}

#[derive(Default)]
struct MainDepth;

struct SubMainDepth;

struct SubDepth;

#[derive(Default)]
struct HorizontalMarkerLine<'a, Level> {
    markers: Vec<Marker<'a>>,
    phantom_data: PhantomData<Level>,
}

impl<'a> HorizontalMarkerLine<'a, MainDepth> {

    fn check_col_or_resize(&mut self, col: usize){

        if col < self.markers.len() {
            return;
        }

        self.markers.resize_with(
            col,
            || Marker { sign: ' ', style: Style::new(), typ: MarkerType::Reapeted }
        );
    }

    fn mark_in_line(&mut self, start_col: usize, end_col: usize, sign: char, style: Style, labels: &'a [&'a str]) {

        self.check_col_or_resize(end_col); // To expand until the end column

        let mut marker = Marker {
            sign: sign,
            style: style,
            typ: MarkerType::OneLineStart { labels: labels }
        };

        self.markers[start_col] = marker.clone(); // The start marker

        marker.typ = MarkerType::Reapeted; // Make after that repeated
            
        for i in start_col + 1 .. end_col {
            self.markers[i] = marker.clone();
        }

    }

    fn mark_as_multi_line_start(&mut self, col: usize, sign: char, style: Style) {

        self.check_col_or_resize(col); // To expand until that column

        let marker = Marker {
            sign: sign,
            style: style,
            typ: MarkerType::MultiLine(MultiLineMarkerType::Start),
        };

        self.markers[col] = marker;
        
    }

    fn mark_as_multi_line_end(&mut self, col: usize, sign: char, style: Style, labels: &'a [&'a str]) {

        self.check_col_or_resize(col); // To expand until that column

        let marker = Marker {
            sign: sign,
            style: style,
            typ: MarkerType::MultiLine(MultiLineMarkerType::End { labels: labels }),
        };

        self.markers[col] = marker;

    }

}

#[derive(Clone)]
struct Marker<'a> {
    sign: char,
    style: Style,
    typ: MarkerType<'a>,
}

#[derive(Clone)]
enum MarkerType<'a> {
    Reapeted,
    OneLineStart{ labels: &'a [&'a str] },
    MultiLine(MultiLineMarkerType<'a>),
}

#[derive(Clone)]
enum MultiLineMarkerType<'a> {
    Start,
    End{ labels: &'a [&'a str] },
}

#[cfg(test)]
mod tests {

    use std::{io::{self, Write}, process::Command};

    use owo_colors::{OwoColorize, Style};

    use crate::span::Span;

    use super::CodeReporter;

    #[test]
    fn initial_test() {


        // RTL printing
        let output = Command::new("printf").arg(r#""\e[2 k""#).output().unwrap();
        io::stdout().write_all(&output.stdout[1..output.stdout.len()-1]).unwrap();
        
        let reporter = CodeReporter::new(
            &[
                "احجز متغير س = 555؛",
                "احجز متغير ص = 555؛",
                "احجز متغير ع = 555؛",
            ]
        )
        .report(
            Span::new((0,5), (0,10)),
            '^',
            Style::new().red().bold(),
            &["القيمة ليست متغيرة"],
        )
        .report(
            Span::new((2,5), (2,10)),
            '^',
            Style::new().yellow().bold(),
            &["القيمة ليست متغيرة"],
        );

        println!("  {} ", "|".bright_blue()); // Add empty line above
        for (k, v) in reporter.lines_to_report.iter() {
            println!("{} {} {}", k.bright_blue(), "|".bright_blue(), reporter.files_lines[*k]);
            print!("  {} ", "|".bright_blue());
            for marker in v.markers.iter() {
                print!("{}", marker.sign.style(marker.style))
            }
            println!()
        }
    }
}