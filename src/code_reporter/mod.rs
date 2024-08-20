use std::{cell::Cell, collections::HashMap, fmt::{self, Display}, rc::Rc};

use itertools::Itertools;
use owo_colors::{OwoColorize, Style };
use painter::Painter;

use crate::span::Span;

mod painter;

struct CodeReporter<'a> {
    /// Map lines indecies and main depth line on them
    code_lines: HashMap<usize, CodeLine<'a>>,
    /// Lines to read from
    files_lines: &'a [&'a str],
}

impl<'a> CodeReporter<'a> {

    fn new(files_lines: &'a [&'a str]) -> Self {
        Self {
            code_lines: HashMap::new(),
            files_lines: files_lines,
        }
    }

    fn report(mut self, span: Span, sign: char, style: Style, labels: &'a [&'a str]) -> Self {

        let start_line = span.start.line;
        let start_col = span.start.col;
        let end_line = span.end.line;
        let end_col = span.end.col;

        if start_line == end_line {

            self.code_lines.entry(start_line)
                .or_insert( CodeLine::default() )
                .mark_as_one_line(start_col, end_col, sign, style, labels);

            return self;
        }

        let connection_margin = Rc::default(); // It will be updated later

        self.code_lines.entry(start_line)
            .or_insert( CodeLine::default() )
            .mark_as_multi_line_start(start_col, sign, style, Rc::clone(&connection_margin));

        self.code_lines.entry(end_line)
            .or_insert( CodeLine::default() )
            .mark_as_multi_line_end(end_col, sign, style, labels, connection_margin);

        for line in start_line + 1 .. end_line {
            // Add lines in between to display them or to modify them later if markers were added to them
            self.code_lines.entry(line).or_insert( CodeLine::default() );
        }

        return self;
    }

    
}

impl<'a> Display for CodeReporter<'a> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut free_connection_margins = vec![];
        let mut connections_painter = Painter::new(
            Marker {sign: MarkerSign::Char(' '), style: Style::new() } // Default is space
        );
        let mut big_sheet = vec![];
        let mut lines_indecies = self.code_lines.keys().sorted();

        let mut num_of_displayed_lines = 0;

        for line_index in lines_indecies.clone() {
            let code_line = &self.code_lines[line_index];

            let file_line = self.files_lines[*line_index];

            big_sheet.push(vec![vec![
                Marker {
                    sign: MarkerSign::CodeLine(file_line),
                    style: Style::new()
                }
            ]]);

            num_of_displayed_lines += 1;

            // This will always align the connections sheet with other sheets
            connections_painter.move_to_zero().move_down_by(num_of_displayed_lines);
        
            let painter_opt = code_line
                .update_connection_margins(
                    &mut free_connection_margins,
                    &mut connections_painter,
                    file_line,
            );

            match painter_opt {
                Some(painter) => {
                    let small_sheet = painter.get_sheet();
                    num_of_displayed_lines += small_sheet.len();
                    big_sheet.push(small_sheet);
                },
                None => {},
            }
            
        }

        let connections_sheet = connections_painter.get_sheet();
        let mut connections = connections_sheet.iter();
        let max_margin = free_connection_margins.len()*2;

        let max_line_num = self.code_lines.keys().max().unwrap() + 1; // Add one to the maximum index
        let max_line_num_indent = max_line_num.to_string().len();

        writeln!(f, "{} {}", " ".repeat(max_line_num_indent), '|'.blue().bold());
        for line_of_markers in big_sheet.iter().flatten() {

            if line_of_markers.len() == 1 && matches!(line_of_markers[0].sign, MarkerSign::CodeLine(_)){
                let line_num_str = (lines_indecies.next().unwrap() + 1).to_string();
                write!(f, "{}{} {}", line_num_str.blue().bold(), " ".repeat(max_line_num_indent-line_num_str.len()), '|'.blue().bold());
            }
            else {
                write!(f, "{} {}", " ".repeat(max_line_num_indent), '|'.blue().bold());
            }

            if let (Some(connection_line), true) = (connections.next(), max_margin > 0) {
                write!(f, "{}", " ".repeat(max_margin-connection_line.len()+1));
                for c in connection_line.iter().rev() {
                    write!(f, "{c}");
                }
            }
            else {
                write!(f, "{}", " ".repeat(max_margin));
            }
            for marker in line_of_markers {
                write!(f, "{marker}");
            }
            writeln!(f);
        
        }

        write!(f, "{} {}", " ".repeat(max_line_num_indent), '|'.blue().bold());

        Ok(())
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
    
    fn mark_as_multi_line_start(&mut self, col: usize, sign: char, style: Style, connection_margin: Rc<Cell<(usize, usize)>>) {
        let marker = Marker { sign: MarkerSign::Char(sign), style: style };
        self.markers.insert(
            col,
            (marker, MarkerType::StartOfMultiLine { connection_margin: connection_margin } )
        );
    }
    
    fn mark_as_multi_line_end(&mut self, col: usize, sign: char, style: Style, labels: &'a [&'a str], connection_margin: Rc<Cell<(usize, usize)>>) {
        let marker = Marker { sign: MarkerSign::Char(sign), style: style };
        self.markers.insert(
            col,
            (marker, MarkerType::EndOfMultiLine { connection_margin: connection_margin, labels: labels })
        );
    }

    fn update_connection_margins(
        &self,
        free_connection_margins: &mut Vec<bool>,
        connections_painter: &mut Painter<Marker<'a>>,
        file_line: &'a str,
    ) -> Option<Painter<Marker<'a>>> {
        
        let mut painter = Painter::new(
            Marker { sign: MarkerSign::Char(' '), style: Style::new() } // Default is space
        );

        let painter_local_zero = painter.current_brush_pos();

        let connections_painter_local_zero = connections_painter.current_brush_pos();
        
        // This is a special case when the multiline marker starts after spaces
        // It will make the marker starts with `/` from connections sheet not from the normal sheet
        /*
         * /      Code
         * | Code Code
         * |_________^
         */

        let min_col_opt = self.markers.keys().min();

        if min_col_opt.is_none() { // No markers
            return None; // No markers to draw in the main sheet
        }

        let min_col = min_col_opt.unwrap();

        let mut min_col_is_after_spaces = false;

        if let
            (MarkerType::StartOfMultiLine { connection_margin }, true)
            = 
            (&self.markers[min_col].1 , file_line.starts_with(&" ".repeat(*min_col)))
        {
            min_col_is_after_spaces = true;
            
            connections_painter.move_up(); // To align with the code line    

            let brush_pos = connections_painter.current_brush_pos();

            let mut is_free_margin_found = false;
            let mut found_margin = 0;
            for (margin, is_free) in free_connection_margins.iter().enumerate() {
                if *is_free {
                    free_connection_margins[margin] = false;
                    is_free_margin_found = true;
                    found_margin = margin;
                    break;
                }
            }

            if !is_free_margin_found {
                found_margin = free_connection_margins.len();
                free_connection_margins.push(false);
            }

            (*connection_margin).set((brush_pos.0, found_margin));

            connections_painter.move_right_by(2*found_margin+1).paint(
                self.markers[min_col].0.clone_with_char('/')
            );

            if self.markers.len() == 1 {
                connections_painter.move_down(); // To reset the line 208
                return None; // No markers to draw in the main sheet
            }
        }


        // The number of bars (`|`) between the code and the next label (of one-line markers and multiline end markers)
        let mut next_labels_margin = 0;

        // The number of bars (`|`) between the code and the repeated underscores (`_`) of multiline marker
        let mut next_multline_margin = 0;

        for col in self.markers.keys().sorted().rev() {

            painter.move_to(painter_local_zero);
            connections_painter.move_to(connections_painter_local_zero);

            let (marker, marker_typ) = &self.markers[col];

            match marker_typ {
                MarkerType::OneLineStart { end_col, labels } => {
                    painter.move_right_by(*end_col);
                    let brush_pos = painter.current_brush_pos();
                    for _ in 0 .. *end_col - col {
                        painter.move_left().paint(marker.clone());
                    }
                    if !labels.is_empty(){
                        // Check if this label margin will be less than the next multiline marker margin
                        // This will prevent labels to be above the next multiline margin
                        // But they may have the same margins
                        /*
                         * Code Code Code
                         *      ^    ^^^^
                         * _____|    |
                         *           Label
                         */
                        if next_labels_margin < next_multline_margin {
                            // The margin of this label should equal the next multiline margin
                            next_labels_margin = next_multline_margin;
                        }
                        if next_labels_margin == 0 {
                            painter.move_to(brush_pos);
                        }
                        else {
                            for _depth in 0..next_labels_margin {
                                painter.move_down().paint(marker.clone_with_char('|'));
                            }
                            painter.move_down();
                        }

                        // Increase the labels margin by number of labels and if it's greater than one subtract one
                        /*
                         *
                         * احجز متغير س = 555؛
                            ^^^^_^^^^^___~ ---من الممكن عدم جعل القيمة متغيرة
                            |    |       |    من الممكن عدم جعل القيمة متغيرة  (remove extra one margin if they're more than one and we are on the first label in reverse)
                            |    |       من الممكن عدم جعل القيمة متغيرة
                            |    من الممكن عدم جعل القيمة متغيرة
                            من الممكن عدم جعل القيمة متغيرة
 
                         */
                        next_labels_margin += labels.len() - (labels.len() > 1 && next_labels_margin == 0) as usize;

                        for (i, label) in labels.iter().enumerate() {
                            if i != 0 {
                                painter.move_down();
                            }
                            painter.paint(marker.clone_with_str(label));
                        }
                    }
                    else if next_labels_margin == 0{
                        // Increase the labels margin if we are on the first marker from reverse and there is no labels found
                        next_labels_margin += 1;
                    }
                },
                MarkerType::EndOfMultiLine { connection_margin, labels } => {

                    painter.move_right_by(*col);
                    
                    let brush_pos = painter.current_brush_pos();

                    painter.move_left().paint(marker.clone());

                    if !labels.is_empty(){
                        // Note it increase the labels depth if they're equal
                        // as labels of multiline end markers have at least one depth greater than it's depth
                        if next_labels_margin <= next_multline_margin && next_multline_margin != 0 { // This happen if it is not the first multiline
                            next_labels_margin = next_multline_margin + 1; // Labels should be below
                        }
                        if next_labels_margin == 0 {
                            painter.move_to(brush_pos);
                        }
                        else {
                            for _depth in 0..next_labels_margin {
                                painter.move_down().paint(marker.clone_with_char('|'));
                            }
                            painter.move_down();
                        }
                        
                        // Increase the labels margin by number of labels and if it's greater than one subtract one
                        next_labels_margin += labels.len() - (labels.len() > 1 && next_labels_margin == 0) as usize;

                        for (i, label) in labels.iter().enumerate() {
                            if i != 0 {
                                painter.move_down();
                            }
                            painter.paint(marker.clone_with_str(label));
                        }
                    }

                    painter.move_to(brush_pos).move_down_by(next_multline_margin).move_left();

                    for _ in 0 .. *col {
                        painter.move_left().paint(marker.clone_with_char('_'));
                    }


                    connections_painter.move_down_by(next_multline_margin);

                    let margin = connection_margin.get().1;
                    
                    free_connection_margins[margin] = true; // free this margin

                    let brush_pos = connections_painter.current_brush_pos();

                    for _ in 0..=margin*2 {
                        connections_painter.paint(
                            marker.clone_with_char('_')
                        ).move_right();
                    }

                    for _ in connection_margin.get().0..brush_pos.0 {
                        connections_painter.paint(
                            marker.clone_with_char('|')
                        ).move_up();
                    }

                    next_multline_margin += 1;
                },
                MarkerType::StartOfMultiLine { connection_margin } => {

                    // This is a special case when the multiline marker starts after spaces
                    // It will make the marker starts with `/` from connections sheet not from the normal sheet
                    /*
                    * /      Code
                    * | Code Code
                    * |_________^
                    */
                    if col == min_col && min_col_is_after_spaces {
                        break; // No more iterations as it is the last column
                    }

                    painter.move_right_by(*col).paint(
                        marker.clone()
                    );

                    for _depth in 0..next_multline_margin {
                        painter.move_down().paint(marker.clone_with_char('|'));
                    }

                    for _ in 0 .. *col {
                        painter.move_left().paint(marker.clone_with_char('_'));
                    }

                    connections_painter.move_down_by(next_multline_margin);

                    let brush_pos = connections_painter.current_brush_pos();

                    let mut is_free_margin_found = false;
                    let mut found_margin = 0;
                    for (margin, is_free) in free_connection_margins.iter().enumerate() {
                        if *is_free {
                            free_connection_margins[margin] = false;
                            is_free_margin_found = true;
                            found_margin = margin;
                            break;
                        }
                    }

                    if !is_free_margin_found {
                        found_margin = free_connection_margins.len();
                        free_connection_margins.push(false);
                    }

                    (*connection_margin).set((brush_pos.0, found_margin));

                    for _ in 0..=found_margin*2 {
                        connections_painter.paint(
                            marker.clone_with_char('_')
                        ).move_right();
                    }

                    next_multline_margin += 1;

                },
            }

        }

        return Some(painter);

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
            MarkerSign::CodeLine(l) => write!(f, "{}", l.style(self.style)),
        }
    }
}

#[derive(Clone)]
enum MarkerSign<'a> {
    Char(char),
    Str(&'a str),
    CodeLine(&'a str),
}

#[derive(Clone)]
enum MarkerType<'a> {
    OneLineStart { end_col: usize, labels: &'a [&'a str] },
    StartOfMultiLine {
        /// This margin and the end marker counter-part should agree on the same margin to connect between them correctly
        /// 
        /// The first is the current row index of the brush of the connection painter
        /// 
        /// The second is the margin index found in the free connection margins array
        /// 
        /// The end counter-part is responsible to draw the connections from it to the position of the start counter-part
        connection_margin: Rc<Cell<(usize, usize)>>
    },
    EndOfMultiLine {
        /// This margin and the start marker counter-part should agree on the same margin to connect between them correctly
        /// 
        /// The first is the row index of the brush of the connection painter at the start counter-part
        /// 
        /// The second is the margin index found in the free connection margins array
        /// 
        /// The end counter-part is responsible to draw the connections from it to the position of the start counter-part
        connection_margin: Rc<Cell<(usize, usize)>>,
        labels: &'a [&'a str],
    },
}

#[cfg(test)]
mod tests {

    use std::{io::{self, Write}, process::Command};

    use owo_colors::{Style, XtermColors};

    use crate::span::Span;

    use super::CodeReporter;

    #[test]
    fn test_reporting() {


        // RTL printing
        let output = Command::new("printf").arg(r#""\e[2 k""#).output().unwrap();
        io::stdout().write_all(&output.stdout[1..output.stdout.len()-1]).unwrap();
        
        let reporter = CodeReporter::new(
            &[
                "احجز متغير س = 555؛",
                "احجز متغير ص = 555؛",
                "احجز متغير ع = 555؛",
                "     متغير ل = 555؛",
                "احجز متغير م = 555؛",
                "احجز متغير ن = 555؛",
                "احجز متغير هـ = 555؛",
                "احجز متغير و = 555؛",
                "احجز متغير ي = 555؛",
                "احجز متغير ز = 555؛",
            ]
        )
        .report(
            Span::new((0,0), (0,4)),
            '?',
            Style::new().blue().cyan(),
            &["القيمة ليست متغيرة"],
        )
        .report(
            Span::new((0,15), (0,18)),
            '~',
            Style::new().blue().green(),
            &["القيمة ليست متغيرة", "القيمة ليست متغيرة", "القيمة ليست متغيرة"],
        )
        .report(
            Span::new((0,5), (0,10)),
            '-',
            Style::new().blue().bold(),
            &["القيمة ليست متغيرة"],
        )
        .report(
            Span::new((2,5), (2,10)),
            '^',
            Style::new().yellow().bold(),
            &["القيمة ليست متغيرة", "القيمة ليست متغيرة"],
        )
        .report(
            Span::new((1,5), (2,4)),
            '^',
            Style::new().red().bold(),
            &["علامة طويلة", "علامة طويلة", "علامة طويلة", "علامة طويلة", "ما قولتلك يا بني علامة طويلة"],
        )
        .report(
            Span::new((1,15), (2,19)),
            '^',
            Style::new().color(XtermColors::FlushOrange).bold(),
            &["علامة طويلة"],
        )
        .report(
            Span::new((0,13), (2,13)),
            '^',
            Style::new().color(XtermColors::PinkFlamingo).bold(),
            &["علامة طويلة"],
        )
        .report(
            Span::new((1,0), (2,0)),
            '^',
            Style::new().color(XtermColors::Brown).bold(),
            &["علامة طويلة"],
        )
        .report(
            Span::new((0,11), (1,4)),
            '^',
            Style::new().magenta().bold(),
            &["علامة طويلة","علامة طويلة","علامة طويلة"],
        )
        .report(
            Span::new((1,8), (1,10)),
            '^',
            Style::new().color(XtermColors::Bermuda).bold(),
            &["علامة طويلة","علامة طويلة","علامة طويلة"],
        )
        .report(
            Span::new((3,5), (6,10)),
            '^',
            Style::new().color(XtermColors::GreenYellow).bold(),
            &["علامة طويلة"],
        )
        .report(
            Span::new((4,11), (5,5)),
            '^',
            Style::new().color(XtermColors::BayLeaf).bold(),
            &["علامة طويلة"],
        )
        .report(
            Span::new((7,15), (7,19)),
            '^',
            Style::new().color(XtermColors::Dandelion).bold(),
            &["علامة طويلة"],
        )
        .report(
            Span::new((7,0), (9,4)),
            '^',
            Style::new().color(XtermColors::Caramel).bold(),
            &["علامة طويلة"],
        )
        .report(
            Span::new((7,5), (9,9)),
            '^',
            Style::new().color(XtermColors::CanCanPink).bold(),
            &["علامة طويلة"],
        )
        .report(
            Span::new((7,10), (9,15)),
            '^',
            Style::new().color(XtermColors::DarkRose).bold(),
            &["علامة طويلة"],
        )
        .report(
            Span::new((7,12), (9,19)),
            '^',
            Style::new().color(XtermColors::Dandelion).bold(),
            &["علامة طويلة"],
        )
        ;

        println!("{}", reporter);
    }
}