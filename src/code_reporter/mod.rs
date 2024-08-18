use std::{collections::HashMap, fmt::{self, write, Display}, marker::{self, PhantomData}};

use itertools::Itertools;
use owo_colors::{AnsiColors::Black, DynColors, OwoColorize, Style, Styled };
use painter::Painter;

use crate::span::Span;

mod painter;

/**
 * let mutable x = 5555 && false;
 *     ^^^     -    ^   ~~ ^^^^^ Label 0   (This is the main marker line and it has a label if there is more than one in the code line)
 * ____|_______|____|   |                  (This is the sub main marker line, and it has no labels)
 * ____|_______|    |   Label 1            (This is a sub marker line, and it has a label)
 *     |       |    Label 2                (~)
 *     |       Label 3                     (~)
 *     |       Label 4                     (The label belongs to the same marker)
 *     Label 5                             (This is a sub marker line, and it has a label)
 */

struct CodeReporter<'a> {
    /// Map lines indecies and main depth line on them
    lines_to_report: HashMap<usize, CodeLine<'a>>,
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
                .or_insert( CodeLine::default() )
                .mark_as_one_line(start_col, end_col, sign, style, labels);

            return self;
        }


        self.lines_to_report.entry(start_line)
            .or_insert( CodeLine::default() )
            .mark_as_multi_line_start(start_col, sign, style);

        self.lines_to_report.entry(end_line)
            .or_insert( CodeLine::default() )
            .mark_as_multi_line_end(end_col, sign, style, labels);


        return self;
    }
}

#[derive(Default)]
struct CodeLine<'a> {
    /// Map column indecies to markers on them
    markers: HashMap<usize, (Marker<'a>, MarkerType<'a>)>,
}

impl<'a> CodeLine<'a> {
    
    fn mark_as_one_line(&mut self, start_col: usize, end_col: usize, sign: char, style: Style, labels: &'a [&'a str]) {
        let marker = Marker { sign: MarkerSign::Char(sign), style: style };
        self.markers.insert(
            start_col,
            (marker, MarkerType::OneLineStart { end_col: end_col, labels: labels } )
        );
    }
    
    fn mark_as_multi_line_start(&mut self, col: usize, sign: char, style: Style) {
        let marker = Marker { sign: MarkerSign::Char(sign), style: style };
        self.markers.insert(
            col,
            (marker, MarkerType::MultiLine(MultiLineMarkerType::Start) )
        );
    }
    
    fn mark_as_multi_line_end(&mut self, col: usize, sign: char, style: Style, labels: &'a [&'a str]) {
        let marker = Marker { sign: MarkerSign::Char(sign), style: style };
        self.markers.insert(
            col,
            (marker, MarkerType::MultiLine(MultiLineMarkerType::End { labels: labels }) )
        );
    }

}


impl<'a> Display for CodeLine<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let mut painter = Painter::new(
            Marker {sign: MarkerSign::Char(' '), style: Style::new()} // Default is space
        );

        let mut max_labels_depth = 0;
        let mut max_multline_depth = 0;

        for col in self.markers.keys().sorted().rev() {

            painter.move_to_zero();

            let (marker, marker_typ) = &self.markers[col];


            match marker_typ {
                MarkerType::OneLineStart { end_col, labels } => {
                    painter.move_right_by(*end_col);
                    let brush_pos = painter.current_brush_pos();
                    for _ in 0 .. end_col - col {
                        painter.move_left().paint(marker.clone());
                    }
                    if !labels.is_empty(){
                        if max_labels_depth < max_multline_depth {
                            max_labels_depth = max_multline_depth; // Labels of one-line markers and multilines may have the same depth
                        }
                        if max_labels_depth == 0 {
                            painter.move_to(brush_pos);
                        }
                        else {
                            
                            for _depth in 0..max_labels_depth {
                                painter.move_down().paint(marker.clone_with_char('|'));
                            }
                            painter.move_down();
                        }
                        
                        painter.paint(
                            marker.clone_with_str(labels[0])
                        );
                        max_labels_depth += 1;
                        
                        for i in 1..labels.len() {
                            painter.move_down().paint(
                                marker.clone_with_str(labels[i])
                            );
                            max_labels_depth += 1;
                        }
                    }
                },
                MarkerType::MultiLine(MultiLineMarkerType::End { labels }) => {
                    painter.move_right_by(*col);
                    
                    let brush_pos = painter.current_brush_pos();

                    painter.move_left().paint(marker.clone());

                    if !labels.is_empty(){
                        // Note it increase the labels depth if they're equal
                        // as labels of multiline end markers have at least one depth greater than it's depth
                        if max_labels_depth <= max_multline_depth && max_multline_depth != 0 { // This happen if it is not the first multiline
                            max_labels_depth = max_multline_depth + 1; // Labels should be below
                        }
                        if max_labels_depth == 0 {
                            painter.move_to(brush_pos);
                        }
                        else {
                            
                            for _depth in 0..max_labels_depth {
                                painter.move_down().paint(marker.clone_with_char('|'));
                            }
                            painter.move_down();
                        }
                        
                        painter.paint(
                            marker.clone_with_str(labels[0])
                        );
                        max_labels_depth += 1;
                        for i in 1..labels.len() {
                            painter.move_down().paint(
                                marker.clone_with_str(labels[i])
                            );
                            max_labels_depth += 1;
                        }
                    }

                    painter.move_to(brush_pos).move_down_by(max_multline_depth).move_left();

                    for _ in 0 .. *col {
                        painter.move_left().paint(marker.clone_with_char('_'));
                    }

                    max_multline_depth += 1;

                },
                MarkerType::MultiLine(MultiLineMarkerType::Start) => {
                    painter.move_right_by(*col).paint(
                        marker.clone()
                    );

                    for _depth in 0..max_multline_depth {
                        painter.move_down().paint(marker.clone_with_char('|'));
                    }

                    for _ in 0 .. *col {
                        painter.move_left().paint(marker.clone_with_char('_'));
                    }

                    max_multline_depth += 1;
                },
            }

        }

        write!(f, "{}", painter);

        Ok(())
    }
}

#[cfg(test)]
mod test_code_line{
    use std::{io::{self, Write}, process::Command};

    use owo_colors::Style;

    use super::{painter::Painter, CodeLine, Marker, MarkerSign};

    fn print_rtl(){
        let output = Command::new("printf").arg(r#""\e[2 k""#).output().unwrap();
        io::stdout().write_all(&output.stdout[1..output.stdout.len()-1]).unwrap();
    }

    #[test]
    fn test(){

        let mut painter = Painter::new(
            Marker {sign: MarkerSign::Char(' '), style: Style::new()} // Default is space
        );
        let marker = Marker { sign: MarkerSign::Char('^'), style: Style::new().bold().red()};
        painter.move_right_by(3).paint(marker.clone())
        .move_down().paint(marker.clone_with_char('|'))
        .move_down().paint(marker.clone_with_char('|'))
        .move_down().paint(marker.clone_with_char('|'))
        .move_down().paint(marker.clone_with_char('|'))
        .move_left().paint(marker.clone_with_char('_'))
        .move_left().paint(marker.clone_with_char('_'))
        .move_left().paint(marker.clone_with_char('_'))
        .move_left().paint(marker.clone_with_char('_'))
        .move_down().paint(marker.clone_with_char('|'))
        .move_down().paint(marker.clone_with_char('|'))
        .move_down().paint(marker.clone_with_char('|'))
        .move_down().paint(marker.clone_with_char('|'))
        .move_right().paint(marker.clone_with_char('_'))
        .move_right().paint(marker.clone_with_char('_'))
        .move_right().paint(marker.clone_with_char('_'))
        .move_right().paint(marker.clone_with_char('_'))
        .move_right_by(5).paint(marker.clone_with_char('أ'))
        .move_right_by(5).paint(marker.clone_with_char('أ'))
        .move_right_by(5).paint(marker.clone_with_char('أ'))
        .move_to_zero()
        .move_right_by(5).paint(marker.clone_with_char('أ'))
        .move_right_by(5).paint(marker.clone_with_char('أ'))
        .move_right_by(5).paint(marker.clone_with_char('أ'));

        println!("{}", painter);

    }

    #[test]
    fn test_one_line(){
        print_rtl();
        let line = "احجز متغير س = 555؛";
        let mut code_line = CodeLine::default();
        code_line.mark_as_one_line(
            5, 
            10, 
            '^', 
            Style::new().bold().yellow(),
            &["من الممكن عدم جعل القيمة متغيرة"]
        );
        code_line.mark_as_one_line(
            0,
            4,
            '^', 
            Style::new().bold().red(),
            &["من الممكن عدم جعل القيمة متغيرة"]
        );
        code_line.mark_as_one_line(
            15,
            18,
            '-', 
            Style::new().bold().blue(),
            &["من الممكن عدم جعل القيمة متغيرة"]
        );
        code_line.mark_as_multi_line_end(
            14,
            '~', 
            Style::new().bold().green(),
            &["من الممكن عدم جعل القيمة متغيرة"]
        );
        code_line.mark_as_multi_line_end(
            19,
            '~', 
            Style::new().bold().magenta(),
            &["من الممكن عدم جعل القيمة متغيرة"]
        );
        println!("{}\n{}", line, code_line)
    }

    #[test]
    fn test_multiline_start(){
        print_rtl();
        let line = "احجز متغير س = 555؛";
        let mut code_line = CodeLine::default();
        code_line.mark_as_multi_line_start(
            15, 
            '-', 
            Style::new().bold().bright_cyan(),
        );
        code_line.mark_as_multi_line_start(
            5, 
            '-', 
            Style::new().bold().blue(),
        );
        code_line.mark_as_multi_line_start(
            11, 
            '-', 
            Style::new().bold().purple(),
        );
        code_line.mark_as_one_line(
            0,
            4,
            '^', 
            Style::new().bold().red(),
            &["من الممكن عدم جعل القيمة متغيرة"]
        );
        println!("{}\n{}", line, code_line)
    }

    #[test]
    fn test_multiline_end(){
        print_rtl();
        let line = "احجز متغير س = 555؛";
        let mut code_line = CodeLine::default();
        code_line.mark_as_multi_line_end(
            2, 
            '^', 
            Style::new().bold().white(),
            &["من الممكن عدم جعل القيمة متغيرة"]
        );
        code_line.mark_as_multi_line_end(
            4, 
            '^', 
            Style::new().bold().blue(),
            &["من الممكن عدم جعل القيمة متغيرة"]
        );
        code_line.mark_as_multi_line_end(
            10, 
            '^', 
            Style::new().bold().red(),
            &["من الممكن عدم جعل القيمة متغيرة"]
        );
        code_line.mark_as_multi_line_end(
            19, 
            '^', 
            Style::new().bold().purple(),
            &["من الممكن عدم جعل القيمة متغيرة"]
        );
        code_line.mark_as_one_line(
            15,
            18,
            '-', 
            Style::new().bold().green(),
            &["من الممكن عدم جعل القيمة متغيرة"]
        );
        code_line.mark_as_one_line(
            5,
            6,
            '~', 
            Style::new().bold().green(),
            &["من الممكن عدم جعل القيمة متغيرة"]
        );
        code_line.mark_as_one_line(
            7,
            8,
            '-', 
            Style::new().bold().bright_cyan(),
            &["من الممكن عدم جعل القيمة متغيرة"]
        );
        code_line.mark_as_multi_line_end(
            14, 
            '~', 
            Style::new().bold().yellow(),
            &["من الممكن عدم جعل القيمة متغيرة"]
        );
        code_line.mark_as_multi_line_start(
            11, 
            '~', 
            Style::new().bold().white(),
        );
        println!("{}\n{}", line, code_line)
    }
}

#[derive(Clone)]
struct Marker<'a> {
    sign: MarkerSign<'a>,
    style: Style,
}

impl<'a> Marker<'a> {

    #[inline]
    fn new_char(ch: char, style: Style) -> Self {
        Self { sign: MarkerSign::Char(ch), style: style }
    }

    #[inline]
    fn new_str(s: &'a str, style: Style) -> Self {
        Self { sign: MarkerSign::Str(s), style: style }
    }

    #[inline]
    fn clone_with_char(&self, ch: char) -> Self {
        Self::new_char(ch, self.style)
    }

    #[inline]
    fn clone_with_str(&self, s: &'a str) -> Self {
        Self::new_str(s, self.style)
    }

}

impl<'a> Display for Marker<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.sign {
            MarkerSign::Char(c) => write!(f, "{}", c.style(self.style)),
            MarkerSign::Str(s) => write!(f, "{}", s.style(self.style)),
        }
    }
}

#[derive(Clone)]
enum MarkerSign<'a> {
    Char(char),
    Str(&'a str)
}

#[derive(Clone)]
enum MarkerType<'a> {
    OneLineStart { end_col: usize, labels: &'a [&'a str] },
    MultiLine(MultiLineMarkerType<'a>),
}

#[derive(Clone)]
enum MultiLineMarkerType<'a> {
    Start,
    End { labels: &'a [&'a str] },
}

#[cfg(test)]
mod tests {

    use std::{io::{self, Write}, process::Command};

    use itertools::Itertools;
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
        )
        .report(
            Span::new((1,5), (2,4)),
            '^',
            Style::new().yellow().bold(),
            &["علامة طويلة"],
        );


        println!("  {} ", "|".bright_blue()); // Add empty line above

        for k in reporter.lines_to_report.keys().sorted() {
            let marker_line = &reporter.lines_to_report[k];
            println!("{} {} {}", k.bright_blue(), "|".bright_blue(), reporter.files_lines[*k]);
            println!("  {} {}", "|".bright_blue(), marker_line);
        }
    }
}